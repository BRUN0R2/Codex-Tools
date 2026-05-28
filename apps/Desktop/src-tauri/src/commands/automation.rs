use std::env;

use serde::{Deserialize, Serialize};

use crate::commands::codex::{CodexLaunchElevation, launch_codex_with_priority};
use crate::contracts::{CommandError, CommandErrorCode, ProcessPriorityRequest};
use crate::platform::windows_automation::{
    AutomationStatus, automation_status, install_automation, remove_automation,
};
use crate::platform::windows_elevated_helper::run_elevated_helper;

const INSTALL_AUTOMATION_ARGUMENT: &str = "--codex-tools-install-automation";
const REMOVE_AUTOMATION_ARGUMENT: &str = "--codex-tools-remove-automation";
const RUN_AUTOMATION_ARGUMENT: &str = "--codex-tools-run-automation";
const PRIORITY_ARGUMENT: &str = "--priority";
const AUTOMATION_HELPER_ACTION_NAME: &str = "automation";
const FAILURE_EXIT_CODE: i32 = 1;
const SUCCESS_EXIT_CODE: i32 = 0;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutomationInstallRequest {
    pub priority: ProcessPriorityRequest,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AutomationStatusResponse {
    pub installed: bool,
    pub executable_path: String,
    pub shortcut_path: String,
    pub task_name: String,
    pub saved_priority: Option<ProcessPriorityRequest>,
}

#[tauri::command]
pub fn get_automation_status() -> Result<AutomationStatusResponse, CommandError> {
    automation_status()
        .map(AutomationStatusResponse::from)
        .map_err(automation_error)
}

#[tauri::command]
pub fn install_codex_automation(
    request: AutomationInstallRequest,
) -> Result<AutomationStatusResponse, CommandError> {
    run_elevated_helper(
        &[
            INSTALL_AUTOMATION_ARGUMENT,
            PRIORITY_ARGUMENT,
            request.priority.cli_value(),
        ],
        CommandErrorCode::AutomationFailed,
        AUTOMATION_HELPER_ACTION_NAME,
    )?;

    get_automation_status()
}

#[tauri::command]
pub fn remove_codex_automation() -> Result<AutomationStatusResponse, CommandError> {
    run_elevated_helper(
        &[REMOVE_AUTOMATION_ARGUMENT],
        CommandErrorCode::AutomationFailed,
        AUTOMATION_HELPER_ACTION_NAME,
    )?;

    get_automation_status()
}

pub fn run_cli_request() -> Option<i32> {
    let arguments: Vec<String> = env::args().skip(1).collect();

    if arguments
        .iter()
        .any(|argument| argument == INSTALL_AUTOMATION_ARGUMENT)
    {
        return Some(exit_code_from_result(run_install_cli_request(&arguments)));
    }

    if arguments
        .iter()
        .any(|argument| argument == REMOVE_AUTOMATION_ARGUMENT)
    {
        return Some(exit_code_from_result(remove_automation().map(|_| ())));
    }

    if arguments
        .iter()
        .any(|argument| argument == RUN_AUTOMATION_ARGUMENT)
    {
        return Some(exit_code_from_result(run_codex_automation_cli_request(
            &arguments,
        )));
    }

    None
}

fn run_install_cli_request(arguments: &[String]) -> Result<(), String> {
    let priority = parse_priority_argument(arguments)?;
    install_automation(priority).map(|_| ())
}

fn run_codex_automation_cli_request(arguments: &[String]) -> Result<(), String> {
    let priority = parse_priority_argument(arguments)?;
    launch_codex_with_priority(priority, CodexLaunchElevation::CurrentToken)
        .map(|_| ())
        .map_err(|error| error.message)
}

fn parse_priority_argument(arguments: &[String]) -> Result<ProcessPriorityRequest, String> {
    let priority_argument_index = arguments
        .iter()
        .position(|argument| argument == PRIORITY_ARGUMENT)
        .ok_or_else(|| "Priority argument is missing.".to_string())?;
    let priority_value_index = priority_argument_index + 1;
    let priority_value = arguments
        .get(priority_value_index)
        .ok_or_else(|| "Priority argument value is missing.".to_string())?;

    ProcessPriorityRequest::parse(priority_value)
        .ok_or_else(|| format!("Unsupported priority: {priority_value}"))
}

fn automation_error(message: String) -> CommandError {
    CommandError::new(CommandErrorCode::AutomationFailed, message)
}

fn exit_code_from_result(result: Result<(), String>) -> i32 {
    match result {
        Ok(()) => SUCCESS_EXIT_CODE,
        Err(_) => FAILURE_EXIT_CODE,
    }
}

impl From<AutomationStatus> for AutomationStatusResponse {
    fn from(status: AutomationStatus) -> Self {
        Self {
            installed: status.installed,
            executable_path: status.executable_path.to_string_lossy().into_owned(),
            shortcut_path: status.shortcut_path.to_string_lossy().into_owned(),
            task_name: status.task_name.to_string(),
            saved_priority: status.saved_priority,
        }
    }
}
