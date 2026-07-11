use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

const CODEX_ELEVATED_TASK_NAME: &str = "OpenAI Codex Elevated";
const POWERSHELL_EXE_NAME: &str = "powershell.exe";
const WINDOWS_POWERSHELL_RELATIVE_PATH: &[&str] =
    &["System32", "WindowsPowerShell", "v1.0", POWERSHELL_EXE_NAME];

pub struct ScheduledTaskLaunch {
    pub task_name: String,
    pub stdout: String,
    pub stderr: String,
}

pub fn launch_codex_desktop_via_elevated_scheduled_task(
    executable_path: &Path,
    launch_argument: &str,
) -> Result<ScheduledTaskLaunch, String> {
    let script = create_launch_script(executable_path, launch_argument);
    let output = Command::new(resolve_windows_powershell())
        .arg("-NoProfile")
        .arg("-ExecutionPolicy")
        .arg("Bypass")
        .arg("-Command")
        .arg(script)
        .output()
        .map_err(|error| format!("Failed to start Windows PowerShell: {error}"))?;

    let launch = ScheduledTaskLaunch {
        task_name: CODEX_ELEVATED_TASK_NAME.to_owned(),
        stdout: String::from_utf8_lossy(&output.stdout).trim().to_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
    };

    if !output.status.success() {
        return Err(format!(
            "PowerShell scheduled task launch failed with status {}. stdout: {} stderr: {}",
            output.status,
            empty_if_blank(&launch.stdout),
            empty_if_blank(&launch.stderr)
        ));
    }

    Ok(launch)
}

fn create_launch_script(executable_path: &Path, launch_argument: &str) -> String {
    let task_name = quote_powershell_single_quoted_string(CODEX_ELEVATED_TASK_NAME);
    let executable_path = quote_powershell_single_quoted_string(&executable_path.to_string_lossy());
    let launch_argument = quote_powershell_single_quoted_string(launch_argument);

    format!(
        r#"
$ErrorActionPreference = 'Stop'
$taskName = {task_name}
$appExe = {executable_path}
$appDirectory = [System.IO.Path]::GetDirectoryName($appExe)
$launchArgument = {launch_argument}
$action = New-ScheduledTaskAction -Execute $appExe -Argument $launchArgument
$identity = [Security.Principal.WindowsIdentity]::GetCurrent()
$principal = New-ScheduledTaskPrincipal -UserId $identity.Name -LogonType Interactive -RunLevel Highest
$settings = New-ScheduledTaskSettingsSet -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries -ExecutionTimeLimit (New-TimeSpan -Hours 12)
$task = New-ScheduledTask -Action $action -Principal $principal -Settings $settings

try {{
    Register-ScheduledTask -TaskName $taskName -InputObject $task -Force | Out-Null
}} catch {{
    $principal = New-ScheduledTaskPrincipal -UserId $identity.User.Value -LogonType Interactive -RunLevel Highest
    $task = New-ScheduledTask -Action $action -Principal $principal -Settings $settings
    Register-ScheduledTask -TaskName $taskName -InputObject $task -Force | Out-Null
}}

Get-Process -Name 'ChatGPT','Codex','codex' -ErrorAction SilentlyContinue |
    Where-Object {{
        $processPath = $_.Path
        $processPath -and (
            [string]::Equals($processPath, $appExe, [StringComparison]::OrdinalIgnoreCase) -or
            $processPath.StartsWith($appDirectory + '\', [StringComparison]::OrdinalIgnoreCase)
        )
    }} |
    Stop-Process -Force
Start-Sleep -Seconds 2
Start-ScheduledTask -TaskName $taskName
"#
    )
}

fn resolve_windows_powershell() -> PathBuf {
    if let Some(windows_directory) = env::var_os("WINDIR").map(PathBuf::from) {
        let powershell_path = WINDOWS_POWERSHELL_RELATIVE_PATH
            .iter()
            .fold(windows_directory, |path, part| path.join(part));

        if powershell_path.is_file() {
            return powershell_path;
        }
    }

    PathBuf::from(POWERSHELL_EXE_NAME)
}

fn quote_powershell_single_quoted_string(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

fn empty_if_blank(value: &str) -> &str {
    if value.is_empty() { "<empty>" } else { value }
}
