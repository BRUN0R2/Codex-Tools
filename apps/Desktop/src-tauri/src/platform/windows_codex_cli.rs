use std::env;
use std::ffi::{OsStr, OsString};
use std::iter::once;
use std::mem::size_of;
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use windows::Win32::System::Threading::{
    CREATE_NEW_CONSOLE, CreateProcessW, HIGH_PRIORITY_CLASS, PROCESS_INFORMATION, STARTUPINFOW,
};
use windows::core::{Error, PCWSTR, PWSTR};

use crate::platform::windows_handle::OwnedWindowsHandle;

const LOCAL_APP_DATA_ENVIRONMENT_VARIABLE: &str = "LOCALAPPDATA";
const PATH_ENVIRONMENT_VARIABLE: &str = "PATH";
const CODEX_CLI_RELATIVE_PATH: &[&str] = &["Programs", "OpenAI", "Codex", "bin", "codex.exe"];
const CODEX_CLI_EXECUTABLE_NAME: &str = "codex.exe";

pub struct CodexCliInstallation {
    executable_path: Option<PathBuf>,
    checked_paths: Vec<PathBuf>,
}

impl CodexCliInstallation {
    pub fn found(&self) -> bool {
        self.executable_path.is_some()
    }

    pub fn executable_path(&self) -> Option<&Path> {
        self.executable_path.as_deref()
    }

    pub fn executable_path_as_string(&self) -> Option<String> {
        self.executable_path
            .as_ref()
            .map(|path| path.to_string_lossy().into_owned())
    }

    pub fn checked_paths_as_strings(&self) -> Vec<String> {
        self.checked_paths
            .iter()
            .map(|path| path.to_string_lossy().into_owned())
            .collect()
    }
}

pub fn locate_codex_cli_installation() -> CodexCliInstallation {
    let checked_paths = collect_codex_cli_candidate_paths();
    let executable_path = checked_paths.iter().find(|path| path.is_file()).cloned();
    CodexCliInstallation {
        executable_path,
        checked_paths,
    }
}

pub fn launch_codex_cli_in_console(
    executable_path: &Path,
    working_directory: &Path,
) -> Result<u32, Error> {
    let executable = wide_from_os_str(executable_path.as_os_str());
    let command_line = quoted_executable_command_line(executable_path);
    let mut command_line = wide_from_os_str(&command_line);
    let directory = wide_from_os_str(working_directory.as_os_str());
    let startup = STARTUPINFOW {
        cb: size_of::<STARTUPINFOW>() as u32,
        ..Default::default()
    };
    let mut process = PROCESS_INFORMATION::default();

    // Do not pass STARTF_USESTDHANDLES: the GUI parent has no console streams to inherit.
    unsafe {
        CreateProcessW(
            PCWSTR::from_raw(executable.as_ptr()),
            Some(PWSTR::from_raw(command_line.as_mut_ptr())),
            None,
            None,
            false,
            CREATE_NEW_CONSOLE | HIGH_PRIORITY_CLASS,
            None,
            PCWSTR::from_raw(directory.as_ptr()),
            &startup,
            &mut process,
        )?;
    }

    let process_handle = OwnedWindowsHandle::new(process.hProcess);
    let thread_handle = OwnedWindowsHandle::new(process.hThread);
    if process_handle.is_none() || thread_handle.is_none() {
        return Err(Error::from_thread());
    }

    Ok(process.dwProcessId)
}

fn quoted_executable_command_line(executable_path: &Path) -> OsString {
    let mut command_line = OsString::from("\"");
    command_line.push(executable_path.as_os_str());
    command_line.push("\"");
    command_line
}

fn wide_from_os_str(value: &OsStr) -> Vec<u16> {
    value.encode_wide().chain(once(0)).collect()
}

fn collect_codex_cli_candidate_paths() -> Vec<PathBuf> {
    let mut candidate_paths = Vec::new();

    if let Some(local_app_data) = env::var_os(LOCAL_APP_DATA_ENVIRONMENT_VARIABLE) {
        let standard_installation =
            CODEX_CLI_RELATIVE_PATH
                .iter()
                .fold(PathBuf::from(local_app_data), |mut path, part| {
                    path.push(part);
                    path
                });
        push_candidate(&mut candidate_paths, standard_installation);
    }

    if let Some(path_environment) = env::var_os(PATH_ENVIRONMENT_VARIABLE) {
        for directory in env::split_paths(&path_environment).filter(|path| path.is_absolute()) {
            push_candidate(
                &mut candidate_paths,
                directory.join(CODEX_CLI_EXECUTABLE_NAME),
            );
        }
    }

    candidate_paths
}

fn push_candidate(candidate_paths: &mut Vec<PathBuf>, candidate: PathBuf) {
    if candidate_paths.iter().all(|existing| {
        !existing
            .to_string_lossy()
            .eq_ignore_ascii_case(&candidate.to_string_lossy())
    }) {
        candidate_paths.push(candidate);
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CODEX_CLI_EXECUTABLE_NAME, CODEX_CLI_RELATIVE_PATH, push_candidate,
        quoted_executable_command_line,
    };
    use std::path::PathBuf;

    #[test]
    fn expected_user_install_path_ends_in_the_cli_executable() {
        let path = CODEX_CLI_RELATIVE_PATH.iter().fold(
            PathBuf::from(r"C:\Users\Example\AppData\Local"),
            |mut path, part| {
                path.push(part);
                path
            },
        );

        assert!(path.ends_with(CODEX_CLI_EXECUTABLE_NAME));
        assert!(path.ends_with(r"OpenAI\Codex\bin\codex.exe"));
    }

    #[test]
    fn candidate_paths_are_deduplicated_case_insensitively() {
        let mut paths = vec![PathBuf::from(r"C:\Tools\Codex.exe")];
        push_candidate(&mut paths, PathBuf::from(r"c:\tools\codex.exe"));
        assert_eq!(paths.len(), 1);
    }

    #[test]
    fn cli_command_line_quotes_paths_with_spaces() {
        let path = PathBuf::from(r"C:\Program Files\OpenAI\Codex\bin\codex.exe");
        assert_eq!(
            quoted_executable_command_line(&path),
            r#""C:\Program Files\OpenAI\Codex\bin\codex.exe""#
        );
    }
}
