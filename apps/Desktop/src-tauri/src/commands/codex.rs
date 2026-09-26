use std::env;
use std::fs;
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

use serde::Serialize;
use tauri::State;
use windows::Win32::System::Threading::CREATE_NEW_CONSOLE;

use crate::contracts::{CommandError, CommandErrorCode};
use crate::platform::codex_cleanup::{
    CodexCleanupReport, clean_codex_workspace as clean_codex_workspace_data,
};
use crate::platform::codex_uninstall::{
    CodexUninstallReport, uninstall_codex_product as uninstall_codex_product_data,
};
use crate::platform::windows_codex::{
    codex_desktop_application_user_model_id, locate_codex_installation,
};
use crate::platform::windows_codex_cli::locate_codex_cli_installation;
use crate::platform::windows_package_activation::launch_codex_desktop_as_administrator;
use crate::platform::windows_package_capability::codex_package_allows_elevation;
use crate::platform::windows_process::{
    CodexProcessInspection, WindowsProcessElevationState, WindowsProcessPriorityState,
    find_elevated_codex_desktop_process, inspect_running_codex_processes,
};
use crate::priority::stabilization::{
    PriorityStabilizationSnapshot, PriorityStabilizationStore,
    start_cli_high_priority_stabilization, start_high_priority_stabilization,
};

const LOCAL_APP_DATA_ENVIRONMENT_VARIABLE: &str = "LOCALAPPDATA";
const CODEX_TOOLS_DATA_DIRECTORY: &str = "CodexTools";
const ADMINISTRATOR_PROFILE_DIRECTORY: &str = "CodexAdminProfile";
const ELEVATED_DESKTOP_STARTUP_TIMEOUT: Duration = Duration::from_secs(45);
const ELEVATED_DESKTOP_POLL_INTERVAL: Duration = Duration::from_millis(250);
const USER_PROFILE_ENVIRONMENT_VARIABLE: &str = "USERPROFILE";

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexStatusResponse {
    pub found: bool,
    pub executable_path: Option<String>,
    pub checked_paths: Vec<String>,
    pub processes: Vec<CodexProcessResponse>,
    pub priority_stabilization: PriorityStabilizationSnapshot,
    pub package_allows_elevation: Option<bool>,
    pub elevation_diagnostic: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexCliStatusResponse {
    pub found: bool,
    pub executable_path: Option<String>,
    pub checked_paths: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexProcessResponse {
    pub process_id: u32,
    pub process_name: String,
    pub executable_path: Option<String>,
    pub priority: CodexProcessPriorityResponse,
    pub elevation: CodexProcessElevationResponse,
}

#[derive(Serialize)]
pub enum CodexProcessPriorityResponse {
    Idle,
    BelowNormal,
    Normal,
    AboveNormal,
    High,
    Realtime,
    Unknown,
}

#[derive(Serialize)]
pub enum CodexProcessElevationResponse {
    Elevated,
    NotElevated,
    Unavailable,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexLaunchResponse {
    pub executable_path: String,
    pub process_id: u32,
    pub priority: CodexLaunchPriorityResponse,
    pub priority_stabilization_started: bool,
}

pub type CodexCleanupResponse = CodexCleanupReport;
pub type CodexUninstallResponse = CodexUninstallReport;

#[derive(Serialize)]
pub enum CodexLaunchPriorityResponse {
    High,
}

#[tauri::command]
pub fn get_codex_status(
    priority_stabilization: State<'_, PriorityStabilizationStore>,
) -> Result<CodexStatusResponse, CommandError> {
    let installation = locate_codex_installation();
    let checked_paths = installation.checked_paths_as_strings();
    let executable_path = installation.executable_path_as_string();
    let (package_allows_elevation, elevation_diagnostic) = match installation.executable_path() {
        Some(path) => match codex_package_allows_elevation(path) {
            Ok(allowed) => (Some(allowed), None),
            Err(error) => (None, Some(error)),
        },
        None => (None, None),
    };
    let priority_stabilization = priority_stabilization.snapshot().map_err(|error| {
        CommandError::new(
            CommandErrorCode::InvalidState,
            format!("Failed to inspect priority stabilization status: {error}"),
        )
    })?;
    let processes = inspect_running_codex_processes()
        .map_err(|error| {
            CommandError::new(
                CommandErrorCode::WindowsApiFailed,
                format!("Failed to inspect Codex processes: {error}"),
            )
        })?
        .into_iter()
        .map(CodexProcessResponse::from)
        .collect();

    if installation.found() && executable_path.is_none() {
        return Err(CommandError::new(
            CommandErrorCode::InvalidState,
            "Codex detection found an executable without a valid path.",
        ));
    }

    Ok(CodexStatusResponse {
        found: installation.found(),
        executable_path,
        checked_paths,
        processes,
        priority_stabilization,
        package_allows_elevation,
        elevation_diagnostic,
    })
}

#[tauri::command]
pub async fn open_codex(
    priority_stabilization: State<'_, PriorityStabilizationStore>,
) -> Result<CodexLaunchResponse, CommandError> {
    let priority_stabilization = priority_stabilization.inner().clone();

    tauri::async_runtime::spawn_blocking(move || {
        launch_codex_with_high_priority(priority_stabilization)
    })
    .await
    .map_err(|error| {
        CommandError::new(
            CommandErrorCode::InvalidState,
            format!("Failed to join the Codex launch task: {error}"),
        )
    })?
}

#[tauri::command]
pub fn get_codex_cli_status() -> CodexCliStatusResponse {
    let installation = locate_codex_cli_installation();
    CodexCliStatusResponse {
        found: installation.found(),
        executable_path: installation.executable_path_as_string(),
        checked_paths: installation.checked_paths_as_strings(),
    }
}

#[tauri::command]
pub async fn open_codex_cli(
    priority_stabilization: State<'_, PriorityStabilizationStore>,
) -> Result<CodexLaunchResponse, CommandError> {
    let priority_stabilization = priority_stabilization.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        launch_codex_cli_with_high_priority(priority_stabilization)
    })
    .await
    .map_err(|error| {
        CommandError::new(
            CommandErrorCode::InvalidState,
            format!("Failed to join the Codex CLI launch task: {error}"),
        )
    })?
}

#[tauri::command]
pub fn clean_codex_workspace() -> Result<CodexCleanupResponse, CommandError> {
    let running_processes = inspect_running_codex_processes().map_err(|error| {
        CommandError::new(
            CommandErrorCode::WindowsApiFailed,
            format!("Nao foi possivel verificar processos Codex antes da limpeza: {error}"),
        )
    })?;

    if !running_processes.is_empty() {
        return Err(CommandError::new(
            CommandErrorCode::CleanupBlocked,
            "Feche o Codex antes de limpar chats e cache.",
        ));
    }

    clean_codex_workspace_data().map_err(|error| {
        CommandError::new(
            CommandErrorCode::CleanupFailed,
            format!("Falha ao limpar dados locais do Codex: {error}"),
        )
    })
}

#[tauri::command]
pub fn uninstall_codex_product() -> Result<CodexUninstallResponse, CommandError> {
    let running_processes = inspect_running_codex_processes().map_err(|error| {
        CommandError::new(
            CommandErrorCode::WindowsApiFailed,
            format!("Nao foi possivel verificar processos Codex antes da desinstalacao: {error}"),
        )
    })?;

    if !running_processes.is_empty() {
        return Err(CommandError::new(
            CommandErrorCode::UninstallBlocked,
            "Feche o Codex e o ChatGPT Desktop antes de desinstalar e apagar todos os dados.",
        ));
    }

    uninstall_codex_product_data().map_err(|error| {
        CommandError::new(
            CommandErrorCode::UninstallFailed,
            format!("Falha ao desinstalar e apagar dados do Codex: {error}"),
        )
    })
}

fn launch_codex_with_high_priority(
    priority_stabilization: PriorityStabilizationStore,
) -> Result<CodexLaunchResponse, CommandError> {
    let installation = locate_codex_installation();
    let executable_path = installation.executable_path().ok_or_else(|| {
        CommandError::new(
            CommandErrorCode::CodexNotFound,
            "Codex executable was not found.",
        )
    })?;
    let application_user_model_id = codex_desktop_application_user_model_id(executable_path)
        .map_err(|error| CommandError::new(CommandErrorCode::InvalidState, error))?;

    let profile_directory = administrator_profile_directory()
        .map_err(|error| CommandError::new(CommandErrorCode::InvalidState, error))?;

    launch_codex_desktop_as_administrator(&application_user_model_id, &profile_directory)
        .map_err(|error| CommandError::new(CommandErrorCode::WindowsApiFailed, error))?;

    let process_id = wait_for_elevated_codex_desktop_process()
        .map_err(|error| CommandError::new(CommandErrorCode::WindowsApiFailed, error))?;

    start_high_priority_stabilization(priority_stabilization).map_err(|error| {
        CommandError::new(
            CommandErrorCode::InvalidState,
            format!("Failed to start priority stabilization: {error}"),
        )
    })?;

    Ok(CodexLaunchResponse {
        executable_path: executable_path.to_string_lossy().into_owned(),
        process_id,
        priority: CodexLaunchPriorityResponse::High,
        priority_stabilization_started: true,
    })
}

fn launch_codex_cli_with_high_priority(
    priority_stabilization: PriorityStabilizationStore,
) -> Result<CodexLaunchResponse, CommandError> {
    let installation = locate_codex_cli_installation();
    let executable_path = installation.executable_path().ok_or_else(|| {
        CommandError::new(
            CommandErrorCode::CodexNotFound,
            "Codex CLI executable was not found.",
        )
    })?;
    let working_directory = cli_working_directory(executable_path);
    let child = Command::new(executable_path)
        .current_dir(working_directory)
        .creation_flags(CREATE_NEW_CONSOLE.0)
        .spawn()
        .map_err(|error| {
            CommandError::new(
                CommandErrorCode::WindowsApiFailed,
                format!("Failed to open the Codex CLI in a new terminal: {error}"),
            )
        })?;
    let process_id = child.id();

    start_cli_high_priority_stabilization(priority_stabilization, process_id).map_err(|error| {
        CommandError::new(
            CommandErrorCode::InvalidState,
            format!("Failed to start Codex CLI priority stabilization: {error}"),
        )
    })?;

    Ok(CodexLaunchResponse {
        executable_path: executable_path.to_string_lossy().into_owned(),
        process_id,
        priority: CodexLaunchPriorityResponse::High,
        priority_stabilization_started: true,
    })
}

fn cli_working_directory(executable_path: &std::path::Path) -> PathBuf {
    env::var_os(USER_PROFILE_ENVIRONMENT_VARIABLE)
        .map(PathBuf::from)
        .filter(|path| path.is_dir())
        .or_else(|| executable_path.parent().map(PathBuf::from))
        .unwrap_or_else(|| env::current_dir().unwrap_or_default())
}

fn administrator_profile_directory() -> Result<PathBuf, String> {
    let local_app_data = env::var_os(LOCAL_APP_DATA_ENVIRONMENT_VARIABLE)
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .ok_or_else(|| "LOCALAPPDATA is unavailable or is not an absolute path.".to_owned())?;
    let profile_directory = local_app_data
        .join(CODEX_TOOLS_DATA_DIRECTORY)
        .join(ADMINISTRATOR_PROFILE_DIRECTORY);

    fs::create_dir_all(&profile_directory).map_err(|error| {
        format!(
            "Failed to create the Codex administrator profile {}: {error}",
            profile_directory.display()
        )
    })?;

    Ok(profile_directory)
}

fn wait_for_elevated_codex_desktop_process() -> Result<u32, String> {
    let started_at = Instant::now();

    loop {
        if let Some(process_id) = find_elevated_codex_desktop_process()
            .map_err(|error| format!("Failed to inspect elevated Codex processes: {error}"))?
        {
            return Ok(process_id);
        }

        if started_at.elapsed() >= ELEVATED_DESKTOP_STARTUP_TIMEOUT {
            return Err(format!(
                "Codex did not start an elevated app-server within {} seconds.",
                ELEVATED_DESKTOP_STARTUP_TIMEOUT.as_secs()
            ));
        }

        thread::sleep(ELEVATED_DESKTOP_POLL_INTERVAL);
    }
}

impl From<CodexProcessInspection> for CodexProcessResponse {
    fn from(process: CodexProcessInspection) -> Self {
        Self {
            process_id: process.process_id,
            process_name: process.process_name,
            executable_path: process.executable_path,
            priority: CodexProcessPriorityResponse::from(process.priority),
            elevation: CodexProcessElevationResponse::from(process.elevation),
        }
    }
}

impl From<WindowsProcessPriorityState> for CodexProcessPriorityResponse {
    fn from(priority: WindowsProcessPriorityState) -> Self {
        match priority {
            WindowsProcessPriorityState::Idle => Self::Idle,
            WindowsProcessPriorityState::BelowNormal => Self::BelowNormal,
            WindowsProcessPriorityState::Normal => Self::Normal,
            WindowsProcessPriorityState::AboveNormal => Self::AboveNormal,
            WindowsProcessPriorityState::High => Self::High,
            WindowsProcessPriorityState::Realtime => Self::Realtime,
            WindowsProcessPriorityState::Unknown => Self::Unknown,
        }
    }
}

impl From<WindowsProcessElevationState> for CodexProcessElevationResponse {
    fn from(elevation: WindowsProcessElevationState) -> Self {
        match elevation {
            WindowsProcessElevationState::Elevated => Self::Elevated,
            WindowsProcessElevationState::NotElevated => Self::NotElevated,
            WindowsProcessElevationState::Unavailable => Self::Unavailable,
        }
    }
}
