use std::env;
use std::io;
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::process::{Command, Output};

use windows::Win32::System::Threading::CREATE_NO_WINDOW;

const POWERSHELL_EXE_NAME: &str = "powershell.exe";
const WINDOWS_POWERSHELL_RELATIVE_PATH: &[&str] =
    &["System32", "WindowsPowerShell", "v1.0", POWERSHELL_EXE_NAME];

pub fn run_hidden_powershell_script(script: &str) -> io::Result<Output> {
    Command::new(resolve_windows_powershell())
        .arg("-NoLogo")
        .arg("-NoProfile")
        .arg("-NonInteractive")
        .arg("-ExecutionPolicy")
        .arg("Bypass")
        .arg("-Command")
        .arg(script)
        .creation_flags(CREATE_NO_WINDOW.0)
        .output()
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
