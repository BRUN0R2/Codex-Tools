use std::ffi::OsStr;
use std::iter::once;
use std::mem::size_of;
use std::os::windows::ffi::OsStrExt;

use windows::Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_SUCCESS, WIN32_ERROR};
use windows::Win32::System::Registry::{
    HKEY, HKEY_LOCAL_MACHINE, KEY_CREATE_SUB_KEY, KEY_READ, KEY_SET_VALUE, KEY_WOW64_64KEY,
    REG_DWORD, REG_OPTION_NON_VOLATILE, REG_VALUE_TYPE, RegCloseKey, RegCreateKeyExW,
    RegDeleteValueW, RegOpenKeyExW, RegQueryValueExW, RegSetValueExW,
};
use windows::core::PCWSTR;

const IFEO_ROOT_SUBKEY: &str =
    r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Image File Execution Options";
const PERF_OPTIONS_SUBKEY_NAME: &str = "PerfOptions";
const CPU_PRIORITY_VALUE_NAME: &str = "CpuPriorityClass";
const CODEX_DESKTOP_EXECUTABLE_NAME: &str = "Codex.exe";
const CODEX_CLI_EXECUTABLE_NAME: &str = "codex.exe";
const HIGH_PRIORITY_CLASS_VALUE: u32 = 3;
const TARGET_EXECUTABLE_NAMES: &[&str] =
    &[CODEX_DESKTOP_EXECUTABLE_NAME, CODEX_CLI_EXECUTABLE_NAME];

pub struct PersistentPriorityStatus {
    pub installed: bool,
    pub targets: Vec<PersistentPriorityTargetStatus>,
}

pub struct PersistentPriorityTargetStatus {
    pub executable_name: &'static str,
    pub installed: bool,
    pub priority_value: Option<u32>,
}

pub fn persistent_high_priority_status() -> Result<PersistentPriorityStatus, String> {
    let targets = TARGET_EXECUTABLE_NAMES
        .iter()
        .map(|executable_name| target_status(executable_name))
        .collect::<Result<Vec<PersistentPriorityTargetStatus>, String>>()?;
    let installed = targets.iter().all(|target| target.installed);

    Ok(PersistentPriorityStatus { installed, targets })
}

pub fn install_persistent_high_priority() -> Result<PersistentPriorityStatus, String> {
    for executable_name in TARGET_EXECUTABLE_NAMES {
        write_high_priority_value(executable_name)?;
    }

    persistent_high_priority_status()
}

pub fn remove_persistent_high_priority() -> Result<PersistentPriorityStatus, String> {
    for executable_name in TARGET_EXECUTABLE_NAMES {
        delete_high_priority_value(executable_name)?;
    }

    persistent_high_priority_status()
}

fn target_status(executable_name: &'static str) -> Result<PersistentPriorityTargetStatus, String> {
    let priority_value = read_priority_value(executable_name)?;

    Ok(PersistentPriorityTargetStatus {
        executable_name,
        installed: priority_value == Some(HIGH_PRIORITY_CLASS_VALUE),
        priority_value,
    })
}

fn write_high_priority_value(executable_name: &str) -> Result<(), String> {
    let key = create_key(&perf_options_subkey_path(executable_name))?;
    let value_name = wide_from_str(CPU_PRIORITY_VALUE_NAME);
    let value_bytes = HIGH_PRIORITY_CLASS_VALUE.to_le_bytes();

    let result = unsafe {
        RegSetValueExW(
            key.raw(),
            PCWSTR::from_raw(value_name.as_ptr()),
            None,
            REG_DWORD,
            Some(&value_bytes),
        )
    };

    ensure_success(
        result,
        &format!("Failed to persist high priority for {executable_name}"),
    )
}

fn read_priority_value(executable_name: &str) -> Result<Option<u32>, String> {
    let Some(key) = open_existing_key(&perf_options_subkey_path(executable_name), KEY_READ)? else {
        return Ok(None);
    };
    let value_name = wide_from_str(CPU_PRIORITY_VALUE_NAME);
    let mut value_type = REG_VALUE_TYPE::default();
    let mut value_bytes = [0_u8; size_of::<u32>()];
    let mut value_size = value_bytes.len() as u32;

    let result = unsafe {
        RegQueryValueExW(
            key.raw(),
            PCWSTR::from_raw(value_name.as_ptr()),
            None,
            Some(&mut value_type),
            Some(value_bytes.as_mut_ptr()),
            Some(&mut value_size),
        )
    };

    if result == ERROR_FILE_NOT_FOUND {
        return Ok(None);
    }

    ensure_success(
        result,
        &format!("Failed to read persisted priority for {executable_name}"),
    )?;

    if value_type != REG_DWORD || value_size != size_of::<u32>() as u32 {
        return Err(format!(
            "Persisted priority for {executable_name} is not a DWORD value."
        ));
    }

    Ok(Some(u32::from_le_bytes(value_bytes)))
}

fn delete_high_priority_value(executable_name: &str) -> Result<(), String> {
    let Some(key) = open_existing_key(
        &perf_options_subkey_path(executable_name),
        KEY_READ | KEY_SET_VALUE,
    )?
    else {
        return Ok(());
    };
    let value_name = wide_from_str(CPU_PRIORITY_VALUE_NAME);
    let result = unsafe { RegDeleteValueW(key.raw(), PCWSTR::from_raw(value_name.as_ptr())) };

    if result == ERROR_FILE_NOT_FOUND {
        return Ok(());
    }

    ensure_success(
        result,
        &format!("Failed to remove persisted priority for {executable_name}"),
    )
}

fn create_key(subkey: &str) -> Result<OwnedRegistryKey, String> {
    let subkey = wide_from_str(subkey);
    let mut key = HKEY::default();
    let result = unsafe {
        RegCreateKeyExW(
            HKEY_LOCAL_MACHINE,
            PCWSTR::from_raw(subkey.as_ptr()),
            None,
            PCWSTR::null(),
            REG_OPTION_NON_VOLATILE,
            KEY_SET_VALUE | KEY_CREATE_SUB_KEY | KEY_WOW64_64KEY,
            None,
            &mut key,
            None,
        )
    };

    ensure_success(result, "Failed to create Registry key")?;
    OwnedRegistryKey::new(key)
}

fn open_existing_key(
    subkey: &str,
    access: windows::Win32::System::Registry::REG_SAM_FLAGS,
) -> Result<Option<OwnedRegistryKey>, String> {
    let subkey = wide_from_str(subkey);
    let mut key = HKEY::default();
    let result = unsafe {
        RegOpenKeyExW(
            HKEY_LOCAL_MACHINE,
            PCWSTR::from_raw(subkey.as_ptr()),
            None,
            access | KEY_WOW64_64KEY,
            &mut key,
        )
    };

    if result == ERROR_FILE_NOT_FOUND {
        return Ok(None);
    }

    ensure_success(result, "Failed to open Registry key")?;
    OwnedRegistryKey::new(key).map(Some)
}

fn ensure_success(result: WIN32_ERROR, context: &str) -> Result<(), String> {
    if result == ERROR_SUCCESS {
        return Ok(());
    }

    Err(format!("{context}: Windows error {}.", result.0))
}

fn perf_options_subkey_path(executable_name: &str) -> String {
    format!("{IFEO_ROOT_SUBKEY}\\{executable_name}\\{PERF_OPTIONS_SUBKEY_NAME}")
}

fn wide_from_str(value: &str) -> Vec<u16> {
    OsStr::new(value).encode_wide().chain(once(0)).collect()
}

struct OwnedRegistryKey {
    key: HKEY,
}

impl OwnedRegistryKey {
    fn new(key: HKEY) -> Result<Self, String> {
        if key.is_invalid() {
            return Err("Registry returned an invalid key handle.".to_string());
        }

        Ok(Self { key })
    }

    fn raw(&self) -> HKEY {
        self.key
    }
}

impl Drop for OwnedRegistryKey {
    fn drop(&mut self) {
        unsafe {
            let _ = RegCloseKey(self.key);
        }
    }
}
