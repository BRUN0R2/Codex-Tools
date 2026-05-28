use std::env;
use std::ffi::OsStr;
use std::iter::once;
use std::mem::size_of;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;

use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Shell::{SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW, ShellExecuteExW};
use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
use windows::core::{Error, PCWSTR};

use crate::platform::windows_handle::OwnedWindowsHandle;
use crate::platform::windows_process::current_process_is_elevated;

const RUN_AS_ADMINISTRATOR_VERB: &str = "runas";

pub fn relaunch_current_process_as_administrator_if_needed() -> Result<bool, String> {
    if current_process_is_elevated()
        .map_err(|error| format!("Failed to inspect Codex Tools elevation: {error}"))?
    {
        return Ok(false);
    }

    let executable_path = env::current_exe()
        .map_err(|error| format!("Failed to resolve Codex Tools executable path: {error}"))?;
    let parameters = current_process_parameters();
    let parameters = (!parameters.is_empty()).then_some(parameters.as_str());

    shell_execute(
        &executable_path,
        parameters,
        Some(RUN_AS_ADMINISTRATOR_VERB),
    )
    .map_err(|error| format!("Failed to relaunch Codex Tools as administrator: {error}"))?;

    Ok(true)
}

pub fn launch_as_administrator(executable_path: &Path) -> Result<(), Error> {
    shell_execute(executable_path, None, Some(RUN_AS_ADMINISTRATOR_VERB))
}

fn shell_execute(
    executable_path: &Path,
    parameters: Option<&str>,
    verb: Option<&str>,
) -> Result<(), Error> {
    let verb = verb.map(wide_from_str);
    let file = wide_from_os_str(executable_path.as_os_str());
    let parameters = parameters.map(wide_from_str);

    let mut execute_info = SHELLEXECUTEINFOW {
        cbSize: size_of::<SHELLEXECUTEINFOW>() as u32,
        fMask: SEE_MASK_NOCLOSEPROCESS,
        hwnd: HWND::default(),
        lpVerb: verb
            .as_ref()
            .map_or(PCWSTR::null(), |value| PCWSTR::from_raw(value.as_ptr())),
        lpFile: PCWSTR::from_raw(file.as_ptr()),
        lpParameters: parameters
            .as_ref()
            .map_or(PCWSTR::null(), |value| PCWSTR::from_raw(value.as_ptr())),
        nShow: SW_SHOWNORMAL.0,
        ..Default::default()
    };

    unsafe {
        ShellExecuteExW(&mut execute_info)?;
    }

    let _process_handle =
        OwnedWindowsHandle::new(execute_info.hProcess).ok_or_else(Error::from_thread)?;

    Ok(())
}

fn current_process_parameters() -> String {
    env::args_os()
        .skip(1)
        .map(|argument| quote_windows_argument(&argument.to_string_lossy()))
        .collect::<Vec<String>>()
        .join(" ")
}

fn quote_windows_argument(value: &str) -> String {
    if !value.is_empty()
        && !value
            .chars()
            .any(|character| character.is_whitespace() || character == '"')
    {
        return value.to_owned();
    }

    let mut quoted_argument = String::from("\"");
    let mut pending_backslash_count = 0usize;

    for character in value.chars() {
        match character {
            '\\' => {
                pending_backslash_count += 1;
            }
            '"' => {
                quoted_argument.push_str(&"\\".repeat((pending_backslash_count * 2) + 1));
                quoted_argument.push('"');
                pending_backslash_count = 0;
            }
            _ => {
                quoted_argument.push_str(&"\\".repeat(pending_backslash_count));
                quoted_argument.push(character);
                pending_backslash_count = 0;
            }
        }
    }

    quoted_argument.push_str(&"\\".repeat(pending_backslash_count * 2));
    quoted_argument.push('"');
    quoted_argument
}

fn wide_from_os_str(value: &OsStr) -> Vec<u16> {
    value.encode_wide().chain(once(0)).collect()
}

fn wide_from_str(value: &str) -> Vec<u16> {
    OsStr::new(value).encode_wide().chain(once(0)).collect()
}
