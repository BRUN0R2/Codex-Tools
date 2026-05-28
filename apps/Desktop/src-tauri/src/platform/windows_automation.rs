use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use crate::contracts::ProcessPriorityRequest;
use crate::platform::windows_shortcut::{ShortcutRequest, create_shortcut};

const APP_DATA_ENVIRONMENT_VARIABLE: &str = "APPDATA";
const LOCAL_APP_DATA_ENVIRONMENT_VARIABLE: &str = "LOCALAPPDATA";
const AUTOMATION_DIRECTORY_NAME: &str = "CodexTools";
const AUTOMATION_EXECUTABLE_FILE_NAME: &str = "CodexTools.exe";
const AUTOMATION_SHORTCUT_FILE_NAME: &str = "Codex Tools.lnk";
const AUTOMATION_TASK_NAME: &str = "CodexToolsOpenCodex";
const START_MENU_RELATIVE_PATH: &[&str] = &["Microsoft", "Windows", "Start Menu", "Programs"];
const RUN_AUTOMATION_ARGUMENT: &str = "--codex-tools-run-automation";
const PRIORITY_ARGUMENT: &str = "--priority";

pub struct AutomationStatus {
    pub installed: bool,
    pub executable_path: PathBuf,
    pub shortcut_path: PathBuf,
    pub task_name: &'static str,
    pub saved_priority: Option<ProcessPriorityRequest>,
}

pub fn install_automation(priority: ProcessPriorityRequest) -> Result<AutomationStatus, String> {
    let paths = automation_paths()?;
    fs::create_dir_all(&paths.install_directory_path).map_err(|error| error.to_string())?;

    let current_executable_path = env::current_exe().map_err(|error| error.to_string())?;

    if !same_path(&current_executable_path, &paths.executable_path) {
        fs::copy(&current_executable_path, &paths.executable_path)
            .map_err(|error| error.to_string())?;
    }

    create_automation_task(&paths.executable_path, priority)?;
    create_shortcut(ShortcutRequest {
        shortcut_path: &paths.shortcut_path,
        target_path: &paths.executable_path,
        description: "Codex Tools",
    })?;

    automation_status()
}

pub fn remove_automation() -> Result<AutomationStatus, String> {
    let paths = automation_paths()?;

    if task_exists()? {
        run_command(
            "schtasks.exe",
            &["/Delete", "/TN", AUTOMATION_TASK_NAME, "/F"],
        )?;
    }

    if paths.shortcut_path.exists() {
        fs::remove_file(&paths.shortcut_path).map_err(|error| error.to_string())?;
    }

    if paths.executable_path.exists() {
        let current_executable_path = env::current_exe().map_err(|error| error.to_string())?;

        if same_path(&current_executable_path, &paths.executable_path) {
            return Err(
                "Codex Tools cannot remove the installed executable while running from it."
                    .to_string(),
            );
        }

        fs::remove_file(&paths.executable_path).map_err(|error| error.to_string())?;
    }

    if paths.install_directory_path.exists() {
        let _ = fs::remove_dir(&paths.install_directory_path);
    }

    automation_status()
}

pub fn automation_status() -> Result<AutomationStatus, String> {
    let paths = automation_paths()?;
    let task_installed = task_exists()?;
    let saved_priority = if task_installed {
        Some(read_saved_task_priority()?)
    } else {
        None
    };
    let installed =
        paths.executable_path.is_file() && paths.shortcut_path.is_file() && task_installed;

    Ok(AutomationStatus {
        installed,
        executable_path: paths.executable_path,
        shortcut_path: paths.shortcut_path,
        task_name: AUTOMATION_TASK_NAME,
        saved_priority,
    })
}

fn create_automation_task(
    executable_path: &Path,
    priority: ProcessPriorityRequest,
) -> Result<(), String> {
    let task_command = format!(
        "\"{}\" {} {} {}",
        executable_path.display(),
        RUN_AUTOMATION_ARGUMENT,
        PRIORITY_ARGUMENT,
        priority.cli_value()
    );

    run_command(
        "schtasks.exe",
        &[
            "/Create",
            "/TN",
            AUTOMATION_TASK_NAME,
            "/SC",
            "ONLOGON",
            "/RL",
            "HIGHEST",
            "/TR",
            &task_command,
            "/F",
        ],
    )
}

fn run_command(program: &str, arguments: &[&str]) -> Result<(), String> {
    let output = Command::new(program)
        .args(arguments)
        .output()
        .map_err(|error| error.to_string())?;

    if output.status.success() {
        return Ok(());
    }

    Err(command_error_message(program, &output))
}

fn read_saved_task_priority() -> Result<ProcessPriorityRequest, String> {
    let output = Command::new("schtasks.exe")
        .args(["/Query", "/TN", AUTOMATION_TASK_NAME, "/XML"])
        .output()
        .map_err(|error| error.to_string())?;

    if !output.status.success() {
        return Err(command_error_message("schtasks.exe", &output));
    }

    let task_definition = command_output_text(&output.stdout);

    parse_priority_from_task_definition(&task_definition)
        .ok_or_else(|| "Automation task is missing the saved priority.".to_string())
}

fn parse_priority_from_task_definition(task_definition: &str) -> Option<ProcessPriorityRequest> {
    let priority_marker = format!("{PRIORITY_ARGUMENT} ");
    let priority_start = task_definition.find(&priority_marker)? + priority_marker.len();
    let priority_value = task_definition[priority_start..]
        .split(is_priority_value_boundary)
        .next()?;

    ProcessPriorityRequest::parse(priority_value)
}

fn is_priority_value_boundary(character: char) -> bool {
    character.is_whitespace() || character == '<' || character == '"' || character == '&'
}

fn command_error_message(program: &str, output: &Output) -> String {
    let stderr = command_output_text(&output.stderr).trim().to_string();
    let stdout = command_output_text(&output.stdout).trim().to_string();

    if !stderr.is_empty() {
        return stderr;
    }

    if !stdout.is_empty() {
        return stdout;
    }

    format!("{program} exited with status {}", output.status)
}

fn command_output_text(bytes: &[u8]) -> String {
    if is_likely_utf16_little_endian(bytes) {
        let utf16_units = bytes
            .chunks_exact(2)
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
            .collect::<Vec<u16>>();

        return String::from_utf16_lossy(&utf16_units);
    }

    String::from_utf8_lossy(bytes).into_owned()
}

fn is_likely_utf16_little_endian(bytes: &[u8]) -> bool {
    let sample_length = bytes.len().min(64);
    let odd_sample_count = sample_length / 2;

    if odd_sample_count == 0 {
        return false;
    }

    let zero_odd_byte_count = bytes
        .iter()
        .take(sample_length)
        .enumerate()
        .filter(|(index, byte)| index % 2 == 1 && **byte == 0)
        .count();

    zero_odd_byte_count * 2 >= odd_sample_count
}

fn task_exists() -> Result<bool, String> {
    let output = Command::new("schtasks.exe")
        .args(["/Query", "/TN", AUTOMATION_TASK_NAME])
        .output()
        .map_err(|error| error.to_string())?;

    Ok(output.status.success())
}

fn automation_paths() -> Result<AutomationPaths, String> {
    let local_app_data_path = required_environment_path(LOCAL_APP_DATA_ENVIRONMENT_VARIABLE)?;
    let app_data_path = required_environment_path(APP_DATA_ENVIRONMENT_VARIABLE)?;

    let install_directory_path = local_app_data_path.join(AUTOMATION_DIRECTORY_NAME);
    let executable_path = install_directory_path.join(AUTOMATION_EXECUTABLE_FILE_NAME);
    let shortcut_directory_path = START_MENU_RELATIVE_PATH
        .iter()
        .fold(app_data_path, |path, part| path.join(part));
    let shortcut_path = shortcut_directory_path.join(AUTOMATION_SHORTCUT_FILE_NAME);

    Ok(AutomationPaths {
        executable_path,
        install_directory_path,
        shortcut_path,
    })
}

fn required_environment_path(name: &str) -> Result<PathBuf, String> {
    env::var_os(name)
        .map(PathBuf::from)
        .ok_or_else(|| format!("{name} is not available."))
}

fn same_path(left: &Path, right: &Path) -> bool {
    match (left.canonicalize(), right.canonicalize()) {
        (Ok(left), Ok(right)) => left == right,
        _ => left == right,
    }
}

struct AutomationPaths {
    executable_path: PathBuf,
    install_directory_path: PathBuf,
    shortcut_path: PathBuf,
}
