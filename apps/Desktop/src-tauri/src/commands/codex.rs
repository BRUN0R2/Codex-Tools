use std::thread;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::contracts::{CommandError, CommandErrorCode};
use crate::platform::windows_codex::locate_codex_installation;
use crate::platform::windows_process::{
    WindowsProcessPriority, apply_priority_to_running_codex, launch_as_administrator,
};

const PRIORITY_APPLICATION_ATTEMPTS: usize = 6;
const PRIORITY_APPLICATION_RETRY_DELAY: Duration = Duration::from_millis(500);

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexStatusResponse {
    pub found: bool,
    pub executable_path: Option<String>,
    pub checked_paths: Vec<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexLaunchRequest {
    pub priority: ProcessPriorityRequest,
}

#[derive(Deserialize, Serialize, Clone, Copy)]
pub enum ProcessPriorityRequest {
    Normal,
    High,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexLaunchResponse {
    pub executable_path: String,
    pub priority: ProcessPriorityRequest,
    pub updated_process_count: usize,
}

#[tauri::command]
pub fn get_codex_status() -> Result<CodexStatusResponse, CommandError> {
    let installation = locate_codex_installation();
    let checked_paths = installation.checked_paths_as_strings();
    let executable_path = installation.executable_path_as_string();

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
    })
}

#[tauri::command]
pub fn open_codex(request: CodexLaunchRequest) -> Result<CodexLaunchResponse, CommandError> {
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

    let priority_application =
        apply_priority_with_retry(WindowsProcessPriority::from(request.priority))?;

    if priority_application.updated_process_ids.is_empty() {
        return Err(CommandError::new(
            CommandErrorCode::InvalidState,
            "Codex was launched, but no Codex process was available for priority update.",
        ));
    }

    Ok(CodexLaunchResponse {
        executable_path: executable_path.to_string_lossy().into_owned(),
        priority: request.priority,
        updated_process_count: priority_application.updated_process_ids.len(),
    })
}

impl From<ProcessPriorityRequest> for WindowsProcessPriority {
    fn from(priority: ProcessPriorityRequest) -> Self {
        match priority {
            ProcessPriorityRequest::Normal => Self::Normal,
            ProcessPriorityRequest::High => Self::High,
        }
    }
}

fn apply_priority_with_retry(
    priority: WindowsProcessPriority,
) -> Result<crate::platform::windows_process::PriorityApplication, CommandError> {
    for attempt_index in 0..PRIORITY_APPLICATION_ATTEMPTS {
        let priority_application = apply_priority_to_running_codex(priority).map_err(|error| {
            CommandError::new(
                CommandErrorCode::WindowsApiFailed,
                format!("Failed to apply Codex priority: {error}"),
            )
        })?;

        if !priority_application.updated_process_ids.is_empty()
            || attempt_index + 1 == PRIORITY_APPLICATION_ATTEMPTS
        {
            return Ok(priority_application);
        }

        thread::sleep(PRIORITY_APPLICATION_RETRY_DELAY);
    }

    Err(CommandError::new(
        CommandErrorCode::InvalidState,
        "Codex priority update finished without a result.",
    ))
}
