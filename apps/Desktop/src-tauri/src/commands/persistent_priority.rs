use std::env;

use serde::Serialize;

use crate::contracts::{CommandError, CommandErrorCode};
use crate::platform::windows_elevated_helper::run_elevated_helper;
use crate::platform::windows_persistent_priority::{
    PersistentPriorityStatus, PersistentPriorityTargetStatus,
    install_persistent_high_priority as install_persistent_high_priority_registry,
    persistent_high_priority_status,
    remove_persistent_high_priority as remove_persistent_high_priority_registry,
};
use crate::platform::windows_process::{WindowsProcessPriority, apply_priority_to_running_codex};

const INSTALL_HIGH_PRIORITY_ARGUMENT: &str = "--codex-tools-install-high-priority";
const REMOVE_HIGH_PRIORITY_ARGUMENT: &str = "--codex-tools-remove-high-priority";
const PERSISTENT_PRIORITY_HELPER_ACTION_NAME: &str = "persistent-priority";
const FAILURE_EXIT_CODE: i32 = 1;
const SUCCESS_EXIT_CODE: i32 = 0;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PersistentPriorityStatusResponse {
    pub installed: bool,
    pub targets: Vec<PersistentPriorityTargetResponse>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PersistentPriorityTargetResponse {
    pub executable_name: String,
    pub installed: bool,
    pub priority_value: Option<u32>,
}

#[tauri::command]
pub fn get_persistent_priority_status() -> Result<PersistentPriorityStatusResponse, CommandError> {
    persistent_high_priority_status()
        .map(PersistentPriorityStatusResponse::from)
        .map_err(persistent_priority_error)
}

#[tauri::command]
pub fn install_persistent_high_priority() -> Result<PersistentPriorityStatusResponse, CommandError>
{
    run_elevated_helper(
        &[INSTALL_HIGH_PRIORITY_ARGUMENT],
        CommandErrorCode::PersistentPriorityFailed,
        PERSISTENT_PRIORITY_HELPER_ACTION_NAME,
    )?;

    get_persistent_priority_status()
}

#[tauri::command]
pub fn remove_persistent_high_priority() -> Result<PersistentPriorityStatusResponse, CommandError> {
    run_elevated_helper(
        &[REMOVE_HIGH_PRIORITY_ARGUMENT],
        CommandErrorCode::PersistentPriorityFailed,
        PERSISTENT_PRIORITY_HELPER_ACTION_NAME,
    )?;

    get_persistent_priority_status()
}

pub fn run_cli_request() -> Option<i32> {
    let arguments: Vec<String> = env::args().skip(1).collect();

    if arguments
        .iter()
        .any(|argument| argument == INSTALL_HIGH_PRIORITY_ARGUMENT)
    {
        return Some(exit_code_from_result(install_high_priority_cli_request()));
    }

    if arguments
        .iter()
        .any(|argument| argument == REMOVE_HIGH_PRIORITY_ARGUMENT)
    {
        return Some(exit_code_from_result(remove_high_priority_cli_request()));
    }

    None
}

fn install_high_priority_cli_request() -> Result<(), String> {
    install_persistent_high_priority_registry()?;
    apply_priority_to_running_codex(WindowsProcessPriority::High)
        .map(|_| ())
        .map_err(|error| {
            format!("Failed to apply high priority to running Codex processes: {error}")
        })
}

fn remove_high_priority_cli_request() -> Result<(), String> {
    remove_persistent_high_priority_registry()?;
    apply_priority_to_running_codex(WindowsProcessPriority::Normal)
        .map(|_| ())
        .map_err(|error| {
            format!("Failed to restore normal priority for running Codex processes: {error}")
        })
}

fn persistent_priority_error(message: String) -> CommandError {
    CommandError::new(CommandErrorCode::PersistentPriorityFailed, message)
}

fn exit_code_from_result(result: Result<(), String>) -> i32 {
    match result {
        Ok(()) => SUCCESS_EXIT_CODE,
        Err(_) => FAILURE_EXIT_CODE,
    }
}

impl From<PersistentPriorityStatus> for PersistentPriorityStatusResponse {
    fn from(status: PersistentPriorityStatus) -> Self {
        Self {
            installed: status.installed,
            targets: status
                .targets
                .into_iter()
                .map(PersistentPriorityTargetResponse::from)
                .collect(),
        }
    }
}

impl From<PersistentPriorityTargetStatus> for PersistentPriorityTargetResponse {
    fn from(status: PersistentPriorityTargetStatus) -> Self {
        Self {
            executable_name: status.executable_name.to_string(),
            installed: status.installed,
            priority_value: status.priority_value,
        }
    }
}
