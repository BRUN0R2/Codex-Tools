use std::collections::BTreeSet;
use std::thread;
use std::time::Duration;

use serde::Serialize;

use crate::contracts::{CommandError, CommandErrorCode};
use crate::platform::windows_codex::locate_codex_installation;
use crate::platform::windows_process::{
    CodexProcessInspection, PriorityApplication, WindowsProcessElevationState,
    WindowsProcessPriorityState, apply_high_priority_to_running_codex,
    inspect_running_codex_processes, launch_as_administrator,
};

const PRIORITY_APPLICATION_MAX_ATTEMPTS: usize = 40;
const PRIORITY_APPLICATION_MIN_ATTEMPTS: usize = 24;
const PRIORITY_APPLICATION_RETRY_DELAY: Duration = Duration::from_millis(500);
const REQUIRED_STABLE_HIGH_PRIORITY_ATTEMPTS: usize = 3;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexStatusResponse {
    pub found: bool,
    pub executable_path: Option<String>,
    pub checked_paths: Vec<String>,
    pub processes: Vec<CodexProcessResponse>,
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
    pub updated_process_count: usize,
}

#[derive(Serialize)]
pub enum CodexLaunchPriorityResponse {
    High,
}

#[tauri::command]
pub fn get_codex_status() -> Result<CodexStatusResponse, CommandError> {
    let installation = locate_codex_installation();
    let checked_paths = installation.checked_paths_as_strings();
    let executable_path = installation.executable_path_as_string();
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
    })
}

#[tauri::command]
pub fn open_codex() -> Result<CodexLaunchResponse, CommandError> {
    launch_codex_with_high_priority()
}

fn launch_codex_with_high_priority() -> Result<CodexLaunchResponse, CommandError> {
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

    let priority_application = apply_high_priority_with_retry()?;

    if priority_application.updated_process_ids.is_empty() {
        return Err(CommandError::new(
            CommandErrorCode::InvalidState,
            "Codex was launched, but no Codex process was available for priority update.",
        ));
    }

    Ok(CodexLaunchResponse {
        executable_path: executable_path.to_string_lossy().into_owned(),
        priority: CodexLaunchPriorityResponse::High,
        updated_process_count: priority_application.updated_process_ids.len(),
    })
}

fn apply_high_priority_with_retry() -> Result<PriorityApplication, CommandError> {
    let mut updated_process_ids = BTreeSet::new();
    let mut stable_high_priority_attempts = 0usize;

    for attempt_index in 0..PRIORITY_APPLICATION_MAX_ATTEMPTS {
        let priority_application = apply_high_priority_to_running_codex().map_err(|error| {
            CommandError::new(
                CommandErrorCode::WindowsApiFailed,
                format!("Failed to apply Codex high priority: {error}"),
            )
        })?;

        updated_process_ids.extend(priority_application.updated_process_ids);

        let processes = inspect_running_codex_processes().map_err(|error| {
            CommandError::new(
                CommandErrorCode::WindowsApiFailed,
                format!("Failed to inspect Codex priority after update: {error}"),
            )
        })?;

        if codex_processes_are_high_priority(&processes) {
            stable_high_priority_attempts += 1;
        } else {
            stable_high_priority_attempts = 0;
        }

        let minimum_attempts_completed = attempt_index + 1 >= PRIORITY_APPLICATION_MIN_ATTEMPTS;
        let priority_is_stable =
            stable_high_priority_attempts >= REQUIRED_STABLE_HIGH_PRIORITY_ATTEMPTS;

        if minimum_attempts_completed && priority_is_stable {
            break;
        }

        if attempt_index + 1 < PRIORITY_APPLICATION_MAX_ATTEMPTS {
            thread::sleep(PRIORITY_APPLICATION_RETRY_DELAY);
        }
    }

    Ok(PriorityApplication {
        updated_process_ids: updated_process_ids.into_iter().collect(),
    })
}

fn codex_processes_are_high_priority(processes: &[CodexProcessInspection]) -> bool {
    !processes.is_empty()
        && processes
            .iter()
            .all(|process| matches!(process.priority, WindowsProcessPriorityState::High))
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
