use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use std::slice;

use windows::Win32::Foundation::ERROR_SUCCESS;
use windows::Win32::System::Registry::{
    HKEY, HKEY_CURRENT_USER, KEY_SET_VALUE, REG_OPTION_NON_VOLATILE, REG_SZ, RegCloseKey,
    RegCreateKeyExW, RegSetValueExW,
};
use windows::core::PCWSTR;

const APP_COMPAT_LAYERS_SUBKEY: &str =
    r"Software\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Layers";
const RUN_AS_ADMINISTRATOR_VALUE: &str = "~ RUNASADMIN";

pub fn register_run_as_administrator(executable_path: &Path) -> Result<(), String> {
    let registry_key = RegistryKey::create_current_user_app_compat_layers()?;
    registry_key.set_string_value(executable_path, RUN_AS_ADMINISTRATOR_VALUE)
}

struct RegistryKey {
    handle: HKEY,
}

impl RegistryKey {
    fn create_current_user_app_compat_layers() -> Result<Self, String> {
        let subkey = wide_from_str(APP_COMPAT_LAYERS_SUBKEY);
        let mut handle = HKEY::default();

        let status = unsafe {
            RegCreateKeyExW(
                HKEY_CURRENT_USER,
                PCWSTR::from_raw(subkey.as_ptr()),
                None,
                PCWSTR::null(),
                REG_OPTION_NON_VOLATILE,
                KEY_SET_VALUE,
                None,
                &mut handle,
                None,
            )
        };
        if status != ERROR_SUCCESS {
            return Err(format!(
                "Failed to create AppCompat registry key. Win32 error: {}.",
                status.0
            ));
        }

        Ok(Self { handle })
    }

    fn set_string_value(&self, value_name: &Path, value: &str) -> Result<(), String> {
        let value_name = wide_from_os_str(value_name.as_os_str());
        let value = wide_from_str(value);
        let status = unsafe {
            RegSetValueExW(
                self.handle,
                PCWSTR::from_raw(value_name.as_ptr()),
                None,
                REG_SZ,
                Some(wide_bytes(&value)),
            )
        };
        if status != ERROR_SUCCESS {
            return Err(format!(
                "Failed to write AppCompat registry value. Win32 error: {}.",
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

fn wide_from_os_str(value: &OsStr) -> Vec<u16> {
    value.encode_wide().chain(once(0)).collect()
}

fn wide_from_str(value: &str) -> Vec<u16> {
    OsStr::new(value).encode_wide().chain(once(0)).collect()
}

fn wide_bytes(value: &[u16]) -> &[u8] {
    unsafe { slice::from_raw_parts(value.as_ptr().cast::<u8>(), std::mem::size_of_val(value)) }
}
