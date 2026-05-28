use std::env;
use std::fs;
use std::path::PathBuf;
use std::process;

use crate::contracts::{CommandError, CommandErrorCode};
use crate::platform::windows_process::run_as_administrator_and_wait;

const ELEVATED_HELPER_FILE_PREFIX: &str = "CodexToolsElevatedHelper";
const ELEVATED_HELPER_FILE_EXTENSION: &str = "exe";
const SUCCESS_PROCESS_EXIT_CODE: u32 = 0;

pub fn run_elevated_helper(
    arguments: &[&str],
    failure_code: CommandErrorCode,
    action_name: &str,
) -> Result<(), CommandError> {
    let helper_executable_path = create_elevated_helper_copy(failure_code, action_name)?;
    let parameters = arguments.join(" ");
    let exit_code =
        run_as_administrator_and_wait(&helper_executable_path, &parameters).map_err(|error| {
            CommandError::new(
                CommandErrorCode::WindowsApiFailed,
                format!("Failed to run elevated {action_name} helper: {error}"),
            )
        })?;
    let _ = fs::remove_file(&helper_executable_path);

    if exit_code == SUCCESS_PROCESS_EXIT_CODE {
        return Ok(());
    }

    Err(CommandError::new(
        failure_code,
        format!("Elevated {action_name} helper failed with exit code {exit_code}."),
    ))
}

fn create_elevated_helper_copy(
    failure_code: CommandErrorCode,
    action_name: &str,
) -> Result<PathBuf, CommandError> {
    let current_executable_path = env::current_exe().map_err(|error| {
        CommandError::new(
            failure_code,
            format!("Failed to resolve current executable: {error}"),
        )
    })?;
    let helper_executable_path = env::temp_dir().join(format!(
        "{ELEVATED_HELPER_FILE_PREFIX}-{action_name}-{}.{}",
        process::id(),
        ELEVATED_HELPER_FILE_EXTENSION
    ));

    fs::copy(&current_executable_path, &helper_executable_path).map_err(|error| {
        CommandError::new(
            failure_code,
            format!("Failed to prepare elevated {action_name} helper: {error}"),
        )
    })?;

    Ok(helper_executable_path)
}
