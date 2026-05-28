use std::ffi::OsStr;
use std::iter::once;
use std::mem::size_of;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;

use windows::Win32::Foundation::HWND;
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW, TH32CS_SNAPPROCESS,
};
use windows::Win32::System::Threading::{
    HIGH_PRIORITY_CLASS, NORMAL_PRIORITY_CLASS, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
    PROCESS_SET_INFORMATION, SetPriorityClass, WaitForInputIdle,
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
pub enum WindowsProcessPriority {
    Normal,
    High,
}

pub struct PriorityApplication {
    pub updated_process_ids: Vec<u32>,
}

pub fn launch_as_administrator(executable_path: &Path) -> Result<(), Error> {
    let verb = wide_from_str(RUN_AS_ADMINISTRATOR_VERB);
    let file = wide_from_os_str(executable_path.as_os_str());

    let mut execute_info = SHELLEXECUTEINFOW {
        cbSize: size_of::<SHELLEXECUTEINFOW>() as u32,
        fMask: SEE_MASK_NOCLOSEPROCESS,
        hwnd: HWND::default(),
        lpVerb: PCWSTR::from_raw(verb.as_ptr()),
        lpFile: PCWSTR::from_raw(file.as_ptr()),
        nShow: SW_SHOWNORMAL.0,
        ..Default::default()
    };

    unsafe {
        ShellExecuteExW(&mut execute_info)?;
    }

    if let Some(handle) = OwnedWindowsHandle::new(execute_info.hProcess) {
        unsafe {
            let _ = WaitForInputIdle(handle.raw(), WAIT_FOR_INPUT_IDLE_TIMEOUT_MILLISECONDS);
        }

        drop(handle);
    }

    Ok(())
}

pub fn apply_priority_to_running_codex(
    priority: WindowsProcessPriority,
) -> Result<PriorityApplication, Error> {
    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)? };
    let snapshot_handle = OwnedWindowsHandle::new(snapshot).ok_or_else(Error::from_thread)?;

    let mut process_entry = PROCESSENTRY32W {
        dwSize: size_of::<PROCESSENTRY32W>() as u32,
        ..Default::default()
    };
    let mut updated_process_ids = Vec::new();

    let mut has_process =
        unsafe { Process32FirstW(snapshot_handle.raw(), &mut process_entry) }.is_ok();

    while has_process {
        if is_codex_process_name(&process_name_from_entry(&process_entry)) {
            apply_priority_to_process(process_entry.th32ProcessID, priority)?;
            updated_process_ids.push(process_entry.th32ProcessID);
        }

        has_process = unsafe { Process32NextW(snapshot_handle.raw(), &mut process_entry) }.is_ok();
    }

    Ok(PriorityApplication {
        updated_process_ids,
    })
}

fn apply_priority_to_process(
    process_id: u32,
    priority: WindowsProcessPriority,
) -> Result<(), Error> {
    let process_handle = unsafe {
        OpenProcess(
            PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SET_INFORMATION,
            false,
            process_id,
        )?
    };
    let process_handle = OwnedWindowsHandle::new(process_handle).ok_or_else(Error::from_thread)?;

    unsafe {
        SetPriorityClass(process_handle.raw(), priority_class(priority))?;
    }

    Ok(())
}

fn priority_class(
    priority: WindowsProcessPriority,
) -> windows::Win32::System::Threading::PROCESS_CREATION_FLAGS {
    match priority {
        WindowsProcessPriority::Normal => NORMAL_PRIORITY_CLASS,
        WindowsProcessPriority::High => HIGH_PRIORITY_CLASS,
    }
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

fn wide_from_os_str(value: &OsStr) -> Vec<u16> {
    value.encode_wide().chain(once(0)).collect()
}

fn wide_from_str(value: &str) -> Vec<u16> {
    OsStr::new(value).encode_wide().chain(once(0)).collect()
}
