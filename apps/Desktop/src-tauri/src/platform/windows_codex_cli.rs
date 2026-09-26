use std::env;
use std::ffi::{OsStr, OsString};
use std::iter::once;
use std::mem::size_of;
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::slice;
use windows::Win32::System::Environment::{FreeEnvironmentStringsW, GetEnvironmentStringsW};
use windows::Win32::System::Threading::{
    CREATE_NEW_CONSOLE, CREATE_UNICODE_ENVIRONMENT, CreateProcessW, HIGH_PRIORITY_CLASS,
    PROCESS_INFORMATION, STARTUPINFOW,
};
use windows::core::{Error, PCWSTR, PWSTR};

use crate::platform::windows_handle::OwnedWindowsHandle;

const LOCAL_APP_DATA_ENVIRONMENT_VARIABLE: &str = "LOCALAPPDATA";
const PATH_ENVIRONMENT_VARIABLE: &str = "PATH";
const CODEX_CLI_RELATIVE_PATH: &[&str] = &["Programs", "OpenAI", "Codex", "bin", "codex.exe"];
const CODEX_CLI_EXECUTABLE_NAME: &str = "codex.exe";
const NO_DAEMON_ARGUMENT: &str = "--no-daemon";
const TERM_ENVIRONMENT_VARIABLE: &str = "TERM";
const DUMB_TERMINAL_VALUE: &str = "dumb";

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
    let command_line = cli_command_line(executable_path);
    let mut command_line = wide_from_os_str(&command_line);
    let directory = wide_from_os_str(working_directory.as_os_str());
    let environment = cli_environment_block()?;
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
            CREATE_NEW_CONSOLE | CREATE_UNICODE_ENVIRONMENT | HIGH_PRIORITY_CLASS,
            Some(environment.as_ptr().cast()),
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

fn cli_command_line(executable_path: &Path) -> OsString {
    // Elevated CLI sessions cannot connect to the shared background daemon.
    let mut command_line = OsString::from("\"");
    command_line.push(executable_path.as_os_str());
    command_line.push("\" ");
    command_line.push(NO_DAEMON_ARGUMENT);
    command_line
}

fn cli_environment_block() -> Result<Vec<u16>, Error> {
    // The GUI may inherit TERM=dumb from a Codex-launched shell environment.
    let source = unsafe { GetEnvironmentStringsW() };
    if source.is_null() {
        return Err(Error::from_thread());
    }

    let source_length = unsafe { environment_block_length(source.as_ptr()) };
    let source_block = unsafe { slice::from_raw_parts(source.as_ptr(), source_length) };
    let environment = without_dumb_terminal(source_block);
    unsafe { FreeEnvironmentStringsW(PCWSTR::from_raw(source.as_ptr())) }?;
    Ok(environment)
}

unsafe fn environment_block_length(source: *const u16) -> usize {
    let mut index = 0;
    loop {
        if unsafe { *source.add(index) == 0 && *source.add(index + 1) == 0 } {
            return index + 2;
        }
        index += 1;
    }
}

fn without_dumb_terminal(source: &[u16]) -> Vec<u16> {
    let mut environment = Vec::with_capacity(source.len());
    for entry in source.split(|character| *character == 0) {
        if entry.is_empty() {
            break;
        }
        if is_dumb_terminal_entry(entry) {
            continue;
        }
        environment.extend_from_slice(entry);
        environment.push(0);
    }
    environment.push(0);
    if environment.len() == 1 {
        environment.push(0);
    }
    environment
}

fn is_dumb_terminal_entry(entry: &[u16]) -> bool {
    let Some(separator) = entry
        .iter()
        .position(|character| *character == u16::from(b'='))
    else {
        return false;
    };
    String::from_utf16_lossy(&entry[..separator]).eq_ignore_ascii_case(TERM_ENVIRONMENT_VARIABLE)
        && String::from_utf16_lossy(&entry[separator + 1..])
            .eq_ignore_ascii_case(DUMB_TERMINAL_VALUE)
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
        CODEX_CLI_EXECUTABLE_NAME, CODEX_CLI_RELATIVE_PATH, cli_command_line, push_candidate,
        without_dumb_terminal,
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
    fn cli_command_line_quotes_paths_with_spaces_and_disables_the_daemon() {
        let path = PathBuf::from(r"C:\Program Files\OpenAI\Codex\bin\codex.exe");
        assert_eq!(
            cli_command_line(&path),
            r#""C:\Program Files\OpenAI\Codex\bin\codex.exe" --no-daemon"#
        );
    }

    #[test]
    fn cli_environment_removes_only_dumb_term_and_keeps_drive_state() {
        let source: Vec<u16> =
            "=C:=C:\\Users\\Example\0Path=C:\\Tools\0TERM=dumb\0USERPROFILE=C:\\Users\\Example\0\0"
                .encode_utf16()
                .collect();
        let expected: Vec<u16> =
            "=C:=C:\\Users\\Example\0Path=C:\\Tools\0USERPROFILE=C:\\Users\\Example\0\0"
                .encode_utf16()
                .collect();

        assert_eq!(without_dumb_terminal(&source), expected);
    }

    #[test]
    fn cli_environment_keeps_a_supported_term() {
        let source: Vec<u16> = "TERM=xterm-256color\0USERPROFILE=C:\\Users\\Example\0\0"
            .encode_utf16()
            .collect();

        assert_eq!(without_dumb_terminal(&source), source);
    }
}
