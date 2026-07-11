use std::thread::sleep;
use std::time::Duration;

use serde::Serialize;
use tauri::State;

use crate::contracts::{CommandError, CommandErrorCode};
use crate::platform::codex_cleanup::{
    CodexCleanupReport, clean_codex_workspace as clean_codex_workspace_data,
};
use crate::platform::windows_app_compat::register_run_as_administrator;
use crate::platform::windows_codex::{
    CODEX_DO_NOT_DE_ELEVATE_ARGUMENT, collect_codex_executable_inventory,
    is_codex_app_server_executable_path, locate_codex_installation,
};
use crate::platform::windows_process::{
    CodexProcessInspection, WindowsProcessElevationState, WindowsProcessPriorityState,
    inspect_running_codex_processes,
};
use crate::platform::windows_scheduled_task::launch_codex_desktop_via_elevated_scheduled_task;
use crate::platform::windows_shell::launch_as_administrator_with_parameters;
use crate::priority::stabilization::{
    PriorityStabilizationSnapshot, PriorityStabilizationStore, start_high_priority_stabilization,
};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexStatusResponse {
    pub found: bool,
    pub executable_path: Option<String>,
    pub checked_paths: Vec<String>,
    pub processes: Vec<CodexProcessResponse>,
    pub priority_stabilization: PriorityStabilizationSnapshot,
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
    pub launch_method: CodexLaunchMethodResponse,
    pub app_server_elevation_observed: bool,
    pub fallback_used: bool,
    pub diagnostic_message: Option<String>,
    pub priority: CodexLaunchPriorityResponse,
    pub priority_stabilization_started: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexRunAsAdministratorRegistrationResponse {
    pub registered_executable_paths: Vec<String>,
    pub scanned_directories: Vec<String>,
}

pub type CodexCleanupResponse = CodexCleanupReport;

#[derive(Serialize)]
pub enum CodexLaunchPriorityResponse {
    High,
}

#[derive(Serialize)]
pub enum CodexLaunchMethodResponse {
    ElevatedScheduledTask,
    ShellExecuteRunAs,
}

const CODEX_APP_SERVER_ELEVATION_POLL_ATTEMPTS: usize = 16;
const CODEX_APP_SERVER_ELEVATION_POLL_INTERVAL: Duration = Duration::from_millis(500);

#[tauri::command]
pub fn get_codex_status(
    priority_stabilization: State<'_, PriorityStabilizationStore>,
) -> Result<CodexStatusResponse, CommandError> {
    let installation = locate_codex_installation();
    let checked_paths = installation.checked_paths_as_strings();
    let executable_path = installation.executable_path_as_string();
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
    })
}

#[tauri::command]
pub fn open_codex(
    priority_stabilization: State<'_, PriorityStabilizationStore>,
) -> Result<CodexLaunchResponse, CommandError> {
    launch_codex_with_high_priority(priority_stabilization.inner().clone())
}

#[tauri::command]
pub fn register_codex_run_as_administrator()
-> Result<CodexRunAsAdministratorRegistrationResponse, CommandError> {
    let inventory = collect_codex_executable_inventory();
    if inventory.executable_paths().is_empty() {
        return Err(CommandError::new(
            CommandErrorCode::CodexNotFound,
            "No Codex executable was found to register as administrator.",
        ));
    }

    for executable_path in inventory.executable_paths() {
        register_run_as_administrator(executable_path).map_err(|error| {
            CommandError::new(
                CommandErrorCode::WindowsApiFailed,
                format!(
                    "Failed to save administrator mode for {}: {error}",
                    executable_path.to_string_lossy()
                ),
            )
        })?;
    }

    Ok(CodexRunAsAdministratorRegistrationResponse {
        registered_executable_paths: inventory.executable_paths_as_strings(),
        scanned_directories: inventory.scanned_directories_as_strings(),
    })
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

    let launch_result = launch_codex_desktop_via_elevated_scheduled_task(
        executable_path,
        CODEX_DO_NOT_DE_ELEVATE_ARGUMENT,
    );
    let (launch_method, fallback_used, diagnostic_message) = match launch_result {
        Ok(launch) => {
            let message = if launch.stderr.is_empty() {
                None
            } else {
                Some(format!(
                    "Elevated scheduled task '{}' completed with stderr: {}",
                    launch.task_name, launch.stderr
                ))
            };

            (
                CodexLaunchMethodResponse::ElevatedScheduledTask,
                false,
                message,
            )
        }
        Err(scheduled_task_error) => {
            launch_as_administrator_with_parameters(executable_path, CODEX_DO_NOT_DE_ELEVATE_ARGUMENT)
                .map_err(|error| {
                    CommandError::new(
                        CommandErrorCode::WindowsApiFailed,
                        format!(
                            "Failed to launch Codex through scheduled task ({scheduled_task_error}) and ShellExecute fallback ({error})."
                        ),
                    )
                })?;

            (
                CodexLaunchMethodResponse::ShellExecuteRunAs,
                true,
                Some(format!(
                    "Elevated scheduled task failed; ShellExecute runas fallback was used. Scheduled task error: {scheduled_task_error}"
                )),
            )
        }
    };

    let app_server_elevation_observed = wait_for_elevated_codex_app_server().map_err(|error| {
        CommandError::new(
            CommandErrorCode::WindowsApiFailed,
            format!("Failed to verify Codex app-server elevation: {error}"),
        )
    })?;

    start_high_priority_stabilization(priority_stabilization).map_err(|error| {
        CommandError::new(
            CommandErrorCode::InvalidState,
            format!("Failed to start priority stabilization: {error}"),
        )
    })?;

    Ok(CodexLaunchResponse {
        executable_path: executable_path.to_string_lossy().into_owned(),
        launch_method,
        app_server_elevation_observed,
        fallback_used,
        diagnostic_message,
        priority: CodexLaunchPriorityResponse::High,
        priority_stabilization_started: true,
    })
}

fn wait_for_elevated_codex_app_server() -> Result<bool, windows::core::Error> {
    for _ in 0..CODEX_APP_SERVER_ELEVATION_POLL_ATTEMPTS {
        sleep(CODEX_APP_SERVER_ELEVATION_POLL_INTERVAL);

        let app_server_is_elevated =
            inspect_running_codex_processes()?
                .into_iter()
                .any(|process| {
                    process.elevation == WindowsProcessElevationState::Elevated
                        && process
                            .executable_path
                            .as_deref()
                            .is_some_and(is_codex_app_server_executable_path)
                });

        if app_server_is_elevated {
            return Ok(true);
        }
    }

    Ok(false)
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
