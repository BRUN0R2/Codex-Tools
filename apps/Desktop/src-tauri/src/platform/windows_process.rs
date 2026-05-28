use std::env;
use std::ffi::OsStr;
use std::iter::once;
use std::mem::size_of;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;

use windows::Win32::Foundation::{HANDLE, HWND};
use windows::Win32::Security::{GetTokenInformation, TOKEN_ELEVATION, TOKEN_QUERY, TokenElevation};
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW, TH32CS_SNAPPROCESS,
};
use windows::Win32::System::Threading::{
    GetCurrentProcess, GetPriorityClass, HIGH_PRIORITY_CLASS, NORMAL_PRIORITY_CLASS, OpenProcess,
    OpenProcessToken, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SET_INFORMATION, SetPriorityClass,
    WaitForInputIdle,
};
use windows::Win32::UI::Shell::{SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW, ShellExecuteExW};
use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
use windows::core::{Error, PCWSTR};

use crate::platform::windows_handle::OwnedWindowsHandle;

const RUN_AS_ADMINISTRATOR_VERB: &str = "runas";
const CODEX_DESKTOP_PROCESS_NAME: &str = "Codex.exe";
const CODEX_PROCESS_NAME: &str = "codex.exe";
const WAIT_FOR_INPUT_IDLE_TIMEOUT_MILLISECONDS: u32 = 1_500;

#[derive(Clone, Copy)]
pub enum WindowsProcessPriorityState {
    Normal,
    High,
    Other,
    Unknown,
}

#[derive(Clone, Copy)]
pub enum WindowsProcessElevationState {
    Elevated,
    NotElevated,
    Unavailable,
}

pub struct CodexProcessInspection {
    pub process_id: u32,
    pub process_name: String,
    pub priority: WindowsProcessPriorityState,
    pub elevation: WindowsProcessElevationState,
}

struct CodexProcessEntry {
    process_id: u32,
    process_name: String,
}

pub struct PriorityApplication {
    pub updated_process_ids: Vec<u32>,
}

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

pub fn apply_high_priority_to_running_codex() -> Result<PriorityApplication, Error> {
    let codex_processes = collect_codex_process_entries()?;
    let mut updated_process_ids = Vec::with_capacity(codex_processes.len());

    for codex_process in codex_processes {
        apply_high_priority_to_process(codex_process.process_id)?;
        updated_process_ids.push(codex_process.process_id);
    }

    Ok(PriorityApplication {
        updated_process_ids,
    })
}

pub fn inspect_running_codex_processes() -> Result<Vec<CodexProcessInspection>, Error> {
    Ok(collect_codex_process_entries()?
        .into_iter()
        .map(inspect_codex_process)
        .collect())
}

fn collect_codex_process_entries() -> Result<Vec<CodexProcessEntry>, Error> {
    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)? };
    let snapshot_handle = OwnedWindowsHandle::new(snapshot).ok_or_else(Error::from_thread)?;

    let mut process_entry = PROCESSENTRY32W {
        dwSize: size_of::<PROCESSENTRY32W>() as u32,
        ..Default::default()
    };
    let mut codex_processes = Vec::new();

    let mut has_process =
        unsafe { Process32FirstW(snapshot_handle.raw(), &mut process_entry) }.is_ok();

    while has_process {
        let process_name = process_name_from_entry(&process_entry);

        if is_codex_process_name(&process_name) {
            codex_processes.push(CodexProcessEntry {
                process_id: process_entry.th32ProcessID,
                process_name,
            });
        }

        has_process = unsafe { Process32NextW(snapshot_handle.raw(), &mut process_entry) }.is_ok();
    }

    Ok(codex_processes)
}

fn inspect_codex_process(codex_process: CodexProcessEntry) -> CodexProcessInspection {
    let process_handle = open_process_for_query(codex_process.process_id).ok();

    CodexProcessInspection {
        process_id: codex_process.process_id,
        process_name: codex_process.process_name,
        priority: process_handle
            .as_ref()
            .map_or(WindowsProcessPriorityState::Unknown, |handle| {
                process_priority_from_handle(handle.raw())
            }),
        elevation: process_handle
            .as_ref()
            .map_or(WindowsProcessElevationState::Unavailable, |handle| {
                process_elevation_from_handle(handle.raw())
            }),
    }
}

fn apply_high_priority_to_process(process_id: u32) -> Result<(), Error> {
    let process_handle = open_process_for_priority_update(process_id)?;

    unsafe {
        SetPriorityClass(process_handle.raw(), HIGH_PRIORITY_CLASS)?;
    }

    Ok(())
}

fn open_process_for_priority_update(process_id: u32) -> Result<OwnedWindowsHandle, Error> {
    let process_handle = unsafe {
        OpenProcess(
            PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SET_INFORMATION,
            false,
            process_id,
        )?
    };

    OwnedWindowsHandle::new(process_handle).ok_or_else(Error::from_thread)
}

fn open_process_for_query(process_id: u32) -> Result<OwnedWindowsHandle, Error> {
    let process_handle =
        unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, process_id)? };

    OwnedWindowsHandle::new(process_handle).ok_or_else(Error::from_thread)
}

fn process_priority_from_handle(process_handle: HANDLE) -> WindowsProcessPriorityState {
    let priority_class = unsafe { GetPriorityClass(process_handle) };

    match priority_class {
        0 => WindowsProcessPriorityState::Unknown,
        value if value == NORMAL_PRIORITY_CLASS.0 => WindowsProcessPriorityState::Normal,
        value if value == HIGH_PRIORITY_CLASS.0 => WindowsProcessPriorityState::High,
        _ => WindowsProcessPriorityState::Other,
    }
}

fn current_process_is_elevated() -> Result<bool, Error> {
    let process_handle = unsafe { GetCurrentProcess() };

    process_is_elevated(process_handle)
}

fn process_elevation_from_handle(process_handle: HANDLE) -> WindowsProcessElevationState {
    match process_is_elevated(process_handle) {
        Ok(true) => WindowsProcessElevationState::Elevated,
        Ok(false) => WindowsProcessElevationState::NotElevated,
        Err(_) => WindowsProcessElevationState::Unavailable,
    }
}

fn process_is_elevated(process_handle: HANDLE) -> Result<bool, Error> {
    let mut token_handle = HANDLE::default();

    unsafe {
        OpenProcessToken(process_handle, TOKEN_QUERY, &mut token_handle)?;
    }

    let token_handle = OwnedWindowsHandle::new(token_handle).ok_or_else(Error::from_thread)?;
    let mut token_elevation = TOKEN_ELEVATION::default();
    let mut returned_length = 0;

    unsafe {
        GetTokenInformation(
            token_handle.raw(),
            TokenElevation,
            Some((&mut token_elevation as *mut TOKEN_ELEVATION).cast()),
            size_of::<TOKEN_ELEVATION>() as u32,
            &mut returned_length,
        )?;
    }

    Ok(token_elevation.TokenIsElevated != 0)
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

    let process_handle =
        OwnedWindowsHandle::new(execute_info.hProcess).ok_or_else(Error::from_thread)?;

    unsafe {
        let _ = WaitForInputIdle(
            process_handle.raw(),
            WAIT_FOR_INPUT_IDLE_TIMEOUT_MILLISECONDS,
        );
    }

    Ok(())
}

fn is_codex_process_name(process_name: &str) -> bool {
    process_name.eq_ignore_ascii_case(CODEX_DESKTOP_PROCESS_NAME)
        || process_name.eq_ignore_ascii_case(CODEX_PROCESS_NAME)
}

fn process_name_from_entry(process_entry: &PROCESSENTRY32W) -> String {
    let length = process_entry
        .szExeFile
        .iter()
        .position(|character| *character == 0)
        .unwrap_or(process_entry.szExeFile.len());

    String::from_utf16_lossy(&process_entry.szExeFile[..length])
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
