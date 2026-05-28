use std::ffi::OsStr;
use std::fs;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;

use windows::Win32::System::Com::{
    CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx,
    CoUninitialize, IPersistFile,
};
use windows::Win32::UI::Shell::{IShellLinkW, ShellLink};
use windows::core::{Error, Interface, PCWSTR};

pub struct ShortcutRequest<'a> {
    pub shortcut_path: &'a Path,
    pub target_path: &'a Path,
    pub description: &'a str,
}

pub fn create_shortcut(request: ShortcutRequest<'_>) -> Result<(), String> {
    if let Some(parent_path) = request.shortcut_path.parent() {
        fs::create_dir_all(parent_path).map_err(|error| error.to_string())?;
    }

    let _apartment = ComApartment::initialize().map_err(|error| error.to_string())?;
    let target_path = wide_from_os_str(request.target_path.as_os_str());
    let shortcut_path = wide_from_os_str(request.shortcut_path.as_os_str());
    let description = wide_from_str(request.description);

    let shell_link: IShellLinkW =
        unsafe { CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER) }
            .map_err(|error: Error| error.to_string())?;

    unsafe {
        shell_link
            .SetPath(PCWSTR::from_raw(target_path.as_ptr()))
            .map_err(|error| error.to_string())?;
        shell_link
            .SetDescription(PCWSTR::from_raw(description.as_ptr()))
            .map_err(|error| error.to_string())?;

        if let Some(working_directory) = request.target_path.parent() {
            let working_directory = wide_from_os_str(working_directory.as_os_str());
            shell_link
                .SetWorkingDirectory(PCWSTR::from_raw(working_directory.as_ptr()))
                .map_err(|error| error.to_string())?;
        }

        let persist_file: IPersistFile = shell_link.cast().map_err(|error| error.to_string())?;
        persist_file
            .Save(PCWSTR::from_raw(shortcut_path.as_ptr()), true)
            .map_err(|error| error.to_string())?;
    }

    Ok(())
}

struct ComApartment;

impl ComApartment {
    fn initialize() -> Result<Self, Error> {
        unsafe {
            CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok()?;
        }

        Ok(Self)
    }
}

impl Drop for ComApartment {
    fn drop(&mut self) {
        unsafe {
            CoUninitialize();
        }
    }
}

fn wide_from_os_str(value: &OsStr) -> Vec<u16> {
    value.encode_wide().chain(once(0)).collect()
}

fn wide_from_str(value: &str) -> Vec<u16> {
    OsStr::new(value).encode_wide().chain(once(0)).collect()
}
