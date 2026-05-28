use std::mem::size_of;

use windows::Win32::Foundation::{ERROR_NOT_ALL_ASSIGNED, GetLastError, HANDLE, LUID};
use windows::Win32::Security::{
    AdjustTokenPrivileges, LUID_AND_ATTRIBUTES, LookupPrivilegeValueW, SE_DEBUG_NAME,
    SE_INC_BASE_PRIORITY_NAME, SE_PRIVILEGE_ENABLED, TOKEN_ADJUST_PRIVILEGES, TOKEN_PRIVILEGES,
    TOKEN_QUERY,
};
use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
use windows::core::{Error, PCWSTR};

use crate::platform::windows_handle::OwnedWindowsHandle;

#[derive(Clone, Copy)]
struct TokenPrivilege {
    display_name: &'static str,
    system_name: PCWSTR,
}

const PROCESS_MANAGEMENT_PRIVILEGES: [TokenPrivilege; 2] = [
    TokenPrivilege {
        display_name: "SeDebugPrivilege",
        system_name: SE_DEBUG_NAME,
    },
    TokenPrivilege {
        display_name: "SeIncreaseBasePriorityPrivilege",
        system_name: SE_INC_BASE_PRIORITY_NAME,
    },
];

pub fn enable_process_management_privileges() -> Result<(), String> {
    let token_handle = current_process_token()
        .map_err(|error| format!("Failed to open current process token: {error}"))?;

    for privilege in PROCESS_MANAGEMENT_PRIVILEGES {
        enable_token_privilege(token_handle.raw(), privilege)?;
    }

    Ok(())
}

fn current_process_token() -> Result<OwnedWindowsHandle, Error> {
    let process_handle = unsafe { GetCurrentProcess() };
    let mut token_handle = HANDLE::default();

    unsafe {
        OpenProcessToken(
            process_handle,
            TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY,
            &mut token_handle,
        )?;
    }

    OwnedWindowsHandle::new(token_handle).ok_or_else(Error::from_thread)
}

fn enable_token_privilege(token_handle: HANDLE, privilege: TokenPrivilege) -> Result<(), String> {
    let mut privilege_id = LUID::default();

    unsafe {
        LookupPrivilegeValueW(PCWSTR::null(), privilege.system_name, &mut privilege_id)
            .map_err(|error| format!("Failed to resolve {}: {error}", privilege.display_name))?;
    }

    let token_privileges = TOKEN_PRIVILEGES {
        PrivilegeCount: 1,
        Privileges: [LUID_AND_ATTRIBUTES {
            Luid: privilege_id,
            Attributes: SE_PRIVILEGE_ENABLED,
        }],
    };

    unsafe {
        AdjustTokenPrivileges(
            token_handle,
            false,
            Some(&token_privileges as *const TOKEN_PRIVILEGES),
            size_of::<TOKEN_PRIVILEGES>() as u32,
            None,
            None,
        )
        .map_err(|error| format!("Failed to enable {}: {error}", privilege.display_name))?;

        if GetLastError() == ERROR_NOT_ALL_ASSIGNED {
            return Err(format!(
                "Windows did not assign required privilege {}",
                privilege.display_name
            ));
        }
    }

    Ok(())
}
