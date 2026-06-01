use serde::Serialize;
use tauri::State;

use crate::contracts::{CommandError, CommandErrorCode};
use crate::platform::windows_app_compat::register_run_as_administrator;
use crate::platform::windows_codex::{
    collect_codex_executable_inventory, locate_codex_installation,
};
use crate::platform::windows_process::{
    CodexProcessInspection, WindowsProcessElevationState, WindowsProcessPriorityState,
    inspect_running_codex_processes,
};
use crate::platform::windows_shell::launch_as_administrator;
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
    pub priority: CodexLaunchPriorityResponse,
    pub priority_stabilization_started: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexRunAsAdministratorRegistrationResponse {
    pub registered_executable_paths: Vec<String>,
    pub scanned_directories: Vec<String>,
}

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

    launch_as_administrator(executable_path).map_err(|error| {
        CommandError::new(
            CommandErrorCode::WindowsApiFailed,
            format!("Failed to launch Codex as administrator: {error}"),
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
        priority: CodexLaunchPriorityResponse::High,
        priority_stabilization_started: true,
    })
}

impl From<CodexProcessInspection> for CodexProcessResponse {
    fn from(process: CodexProcessInspection) -> Self {
        Self {
            process_id: process.process_id,
            process_name: process.process_name,
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
