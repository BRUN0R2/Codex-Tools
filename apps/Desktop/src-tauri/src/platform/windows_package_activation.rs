use std::ffi::OsStr;
use std::iter::once;
use std::mem::size_of;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use std::sync::mpsc;
use std::thread;

use windows::Win32::Foundation::HWND;
use windows::Win32::System::Com::{COINIT_APARTMENTTHREADED, CoInitializeEx, CoUninitialize};
use windows::Win32::UI::Shell::{SHELLEXECUTEINFOW, ShellExecuteExW};
use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
use windows::core::PCWSTR;

use crate::platform::windows_shell::quote_windows_argument;

const ACTIVATION_THREAD_NAME: &str = "codex-package-activation";
const APPS_FOLDER_PREFIX: &str = "shell:AppsFolder\\";
const RUN_AS_ADMINISTRATOR_VERB: &str = "runas";
const KEEP_ELEVATION_ARGUMENT: &str = "--do-not-de-elevate";
const USER_DATA_DIRECTORY_ARGUMENT: &str = "--user-data-dir=";

pub fn launch_codex_desktop_as_administrator(
    application_user_model_id: &str,
    profile_directory: &Path,
) -> Result<(), String> {
    let application_user_model_id = application_user_model_id.to_owned();
    let profile_directory = profile_directory.to_path_buf();
    let (sender, receiver) = mpsc::channel();

    thread::Builder::new()
        .name(ACTIVATION_THREAD_NAME.to_owned())
        .spawn(move || {
            let result = launch_on_sta_thread(&application_user_model_id, &profile_directory);
            let _ = sender.send(result);
        })
        .map_err(|error| format!("Failed to start Codex package activation: {error}"))?;

    receiver
        .recv()
        .map_err(|_| "Codex package activation ended without a result.".to_owned())?
}

fn launch_on_sta_thread(
    application_user_model_id: &str,
    profile_directory: &Path,
) -> Result<(), String> {
    unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED)
            .ok()
            .map_err(|error| format!("Failed to initialize COM for Codex activation: {error}"))?;

        let launch_result = launch_package(application_user_model_id, profile_directory);
        CoUninitialize();
        launch_result
    }
}

fn launch_package(application_user_model_id: &str, profile_directory: &Path) -> Result<(), String> {
    // The AppsFolder item keeps the MSIX identity. Starting ChatGPT.exe directly
    // can produce an elevated token, but the app then fails without package identity.
    let package_item = wide_null(&format!("{APPS_FOLDER_PREFIX}{application_user_model_id}"));
    let verb = wide_null(RUN_AS_ADMINISTRATOR_VERB);
    let user_data_directory = format!(
        "{USER_DATA_DIRECTORY_ARGUMENT}{}",
        profile_directory.display()
    );
    let arguments = wide_null(&format!(
        "{KEEP_ELEVATION_ARGUMENT} {}",
        quote_windows_argument(&user_data_directory)
    ));

    // Shell namespace items do not always return a process handle. Requesting one
    // can report an error even after Windows successfully starts the package.
    let mut execute_info = SHELLEXECUTEINFOW {
        cbSize: size_of::<SHELLEXECUTEINFOW>() as u32,
        hwnd: HWND::default(),
        lpVerb: PCWSTR::from_raw(verb.as_ptr()),
        lpFile: PCWSTR::from_raw(package_item.as_ptr()),
        lpParameters: PCWSTR::from_raw(arguments.as_ptr()),
        nShow: SW_SHOWNORMAL.0,
        ..Default::default()
    };

    unsafe { ShellExecuteExW(&mut execute_info) }.map_err(|error| {
        format!("Windows denied elevated activation of the Codex package: {error}")
    })
}

fn wide_null(value: &str) -> Vec<u16> {
    OsStr::new(value).encode_wide().chain(once(0)).collect()
}
