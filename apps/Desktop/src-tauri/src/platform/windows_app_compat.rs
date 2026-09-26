use std::ffi::{OsStr, OsString};
use std::iter::once;
use std::os::windows::ffi::{OsStrExt, OsStringExt};

use windows::Win32::Foundation::{ERROR_NO_MORE_ITEMS, ERROR_SUCCESS};
use windows::Win32::System::Registry::{
    HKEY, HKEY_CURRENT_USER, KEY_QUERY_VALUE, KEY_SET_VALUE, RegCloseKey, RegDeleteValueW,
    RegEnumValueW, RegOpenKeyExW,
};
use windows::core::PCWSTR;

const APP_COMPAT_LAYERS_SUBKEY: &str =
    r"Software\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Layers";
const CODEX_APP_COMPAT_PATH_MARKERS: &[&str] = &[
    r"\windowsapps\openai.codex_",
    r"\openai\codex\",
    r"\openai.codex_",
];

pub fn remove_codex_run_as_administrator_entries() -> Result<Vec<String>, String> {
    let registry_key = match RegistryKey::open_current_user_app_compat_layers_for_write() {
        Ok(key) => key,
        Err(error) if error.contains("Win32 error: 2") => return Ok(Vec::new()),
        Err(error) => return Err(error),
    };

    let value_names = registry_key.list_value_names()?;
    let mut removed_paths = Vec::new();

    for value_name in value_names {
        if !is_codex_app_compat_path(&value_name) {
            continue;
        }

        registry_key.delete_value_name(&value_name)?;
        removed_paths.push(value_name);
    }

    Ok(removed_paths)
}

fn is_codex_app_compat_path(value_name: &str) -> bool {
    let normalized = value_name.replace('/', "\\").to_ascii_lowercase();
    CODEX_APP_COMPAT_PATH_MARKERS
        .iter()
        .any(|marker| normalized.contains(marker))
}

struct RegistryKey {
    handle: HKEY,
}

impl RegistryKey {
    fn open_current_user_app_compat_layers_for_write() -> Result<Self, String> {
        let subkey = wide_from_str(APP_COMPAT_LAYERS_SUBKEY);
        let mut handle = HKEY::default();

        let status = unsafe {
            RegOpenKeyExW(
                HKEY_CURRENT_USER,
                PCWSTR::from_raw(subkey.as_ptr()),
                None,
                KEY_QUERY_VALUE | KEY_SET_VALUE,
                &mut handle,
            )
        };
        if status != ERROR_SUCCESS {
            return Err(format!(
                "Failed to open AppCompat registry key. Win32 error: {}.",
                status.0
            ));
        }

        Ok(Self { handle })
    }

    fn list_value_names(&self) -> Result<Vec<String>, String> {
        let mut value_names = Vec::new();
        let mut index = 0u32;

        loop {
            let mut name_buffer = vec![0u16; 32_768];
            let mut name_length = name_buffer.len() as u32;
            let status = unsafe {
                RegEnumValueW(
                    self.handle,
                    index,
                    Some(windows::core::PWSTR(name_buffer.as_mut_ptr())),
                    &mut name_length,
                    None,
                    None,
                    None,
                    None,
                )
            };

            if status == ERROR_NO_MORE_ITEMS {
                break;
            }

            if status != ERROR_SUCCESS {
                return Err(format!(
                    "Failed to enumerate AppCompat registry values. Win32 error: {}.",
                    status.0
                ));
            }

            name_buffer.truncate(name_length as usize);
            value_names.push(
                OsString::from_wide(&name_buffer)
                    .to_string_lossy()
                    .into_owned(),
            );
            index += 1;
        }

        Ok(value_names)
    }

    fn delete_value_name(&self, value_name: &str) -> Result<(), String> {
        let value_name = wide_from_str(value_name);
        let status = unsafe { RegDeleteValueW(self.handle, PCWSTR::from_raw(value_name.as_ptr())) };
        if status != ERROR_SUCCESS {
            return Err(format!(
                "Failed to delete AppCompat registry value. Win32 error: {}.",
                status.0
            ));
        }

        Ok(())
    }
}

impl Drop for RegistryKey {
    fn drop(&mut self) {
        unsafe {
            let _ = RegCloseKey(self.handle);
        }
    }
}

fn wide_from_str(value: &str) -> Vec<u16> {
    OsStr::new(value).encode_wide().chain(once(0)).collect()
}
