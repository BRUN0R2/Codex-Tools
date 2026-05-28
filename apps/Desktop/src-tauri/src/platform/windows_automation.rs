use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

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
    let installed =
        paths.executable_path.is_file() && paths.shortcut_path.is_file() && task_exists()?;

    Ok(AutomationStatus {
        installed,
        executable_path: paths.executable_path,
        shortcut_path: paths.shortcut_path,
        task_name: AUTOMATION_TASK_NAME,
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

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();

    if !stderr.is_empty() {
        return Err(stderr);
    }

    if !stdout.is_empty() {
        return Err(stdout);
    }

    Err(format!("{program} exited with status {}", output.status))
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
