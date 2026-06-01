use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const CODEX_DESKTOP_EXECUTABLE_FILE_NAME: &str = "Codex.exe";
const CODEX_EXECUTABLE_FILE_NAME: &str = "codex.exe";
const CODEX_ALIAS_FILE_NAME: &str = "codex";
const LOCAL_APP_DATA_ENVIRONMENT_VARIABLE: &str = "LOCALAPPDATA";
const PATH_ENVIRONMENT_VARIABLE: &str = "PATH";
const PROGRAM_FILES_ENVIRONMENT_VARIABLE: &str = "ProgramFiles";
const USER_PROFILE_ENVIRONMENT_VARIABLE: &str = "USERPROFILE";
const WINDOWS_APPS_DIRECTORY_NAME: &str = "WindowsApps";
const CODEX_HOME_DIRECTORY_NAME: &str = ".codex";
const CODEX_PACKAGE_DIRECTORY_PREFIX: &str = "OpenAI.Codex_";
const CODEX_PACKAGE_DIRECTORY_SUFFIX: &str = "__2p2nqsd0c76g0";
const CODEX_DESKTOP_RELATIVE_PATH: &[&str] = &["app", CODEX_DESKTOP_EXECUTABLE_FILE_NAME];
const CODEX_RESOURCE_RELATIVE_PATH: &[&str] = &["app", "resources", CODEX_EXECUTABLE_FILE_NAME];

const LOCAL_CODEX_RELATIVE_PATH: &[&str] = &["OpenAI", "Codex", "bin", CODEX_EXECUTABLE_FILE_NAME];
const PACKAGE_CODEX_RELATIVE_PATH: &[&str] = &[
    "Packages",
    "OpenAI.Codex_2p2nqsd0c76g0",
    "LocalCache",
    "Local",
    "OpenAI",
    "Codex",
    "bin",
    CODEX_EXECUTABLE_FILE_NAME,
];

pub struct CodexInstallation {
    executable_path: Option<PathBuf>,
    checked_paths: Vec<PathBuf>,
}

pub struct CodexExecutableInventory {
    executable_paths: Vec<PathBuf>,
    scanned_directories: Vec<PathBuf>,
}

impl CodexInstallation {
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

impl CodexExecutableInventory {
    pub fn executable_paths(&self) -> &[PathBuf] {
        &self.executable_paths
    }

    pub fn executable_paths_as_strings(&self) -> Vec<String> {
        self.executable_paths
            .iter()
            .map(|path| path.to_string_lossy().into_owned())
            .collect()
    }

    pub fn scanned_directories_as_strings(&self) -> Vec<String> {
        self.scanned_directories
            .iter()
            .map(|path| path.to_string_lossy().into_owned())
            .collect()
    }
}

pub fn locate_codex_installation() -> CodexInstallation {
    let checked_paths = collect_codex_candidate_paths();
    let executable_path = checked_paths.iter().find(|path| path.is_file()).cloned();

    CodexInstallation {
        executable_path,
        checked_paths,
    }
}

pub fn collect_codex_executable_inventory() -> CodexExecutableInventory {
    let mut executable_paths = Vec::new();
    let mut scanned_directories = Vec::new();

    collect_codex_installation_directories(&mut scanned_directories);

    for directory in &scanned_directories {
        collect_executables_in_directory(directory, &mut executable_paths);
    }

    for candidate_path in collect_codex_candidate_paths() {
        if candidate_path.is_file() && is_windows_executable(&candidate_path) {
            push_candidate(&mut executable_paths, candidate_path);
        }
    }

    CodexExecutableInventory {
        executable_paths,
        scanned_directories,
    }
}

fn collect_codex_candidate_paths() -> Vec<PathBuf> {
    let mut candidate_paths = Vec::new();

    push_windows_apps_candidates(&mut candidate_paths);

    if let Some(local_app_data_path) =
        env::var_os(LOCAL_APP_DATA_ENVIRONMENT_VARIABLE).map(PathBuf::from)
    {
        push_relative_candidate(
            &mut candidate_paths,
            &local_app_data_path,
            LOCAL_CODEX_RELATIVE_PATH,
        );
        push_relative_candidate(
            &mut candidate_paths,
            &local_app_data_path,
            PACKAGE_CODEX_RELATIVE_PATH,
        );
    }

    if let Some(path_value) = env::var_os(PATH_ENVIRONMENT_VARIABLE) {
        for directory in env::split_paths(&path_value) {
            push_candidate(
                &mut candidate_paths,
                directory.join(CODEX_EXECUTABLE_FILE_NAME),
            );
            push_candidate(&mut candidate_paths, directory.join(CODEX_ALIAS_FILE_NAME));
        }
    }

    candidate_paths
}

fn collect_codex_installation_directories(directories: &mut Vec<PathBuf>) {
    push_windows_apps_directories(directories);

    if let Some(local_app_data_path) =
        env::var_os(LOCAL_APP_DATA_ENVIRONMENT_VARIABLE).map(PathBuf::from)
    {
        push_directory_candidate(
            directories,
            build_relative_path(&local_app_data_path, &["OpenAI", "Codex"]),
        );
        push_directory_candidate(
            directories,
            build_relative_path(
                &local_app_data_path,
                &[
                    "Packages",
                    "OpenAI.Codex_2p2nqsd0c76g0",
                    "LocalCache",
                    "Local",
                    "OpenAI",
                    "Codex",
                ],
            ),
        );
    }

    if let Some(user_profile_path) =
        env::var_os(USER_PROFILE_ENVIRONMENT_VARIABLE).map(PathBuf::from)
    {
        push_directory_candidate(
            directories,
            user_profile_path.join(CODEX_HOME_DIRECTORY_NAME),
        );
    }
}

fn push_windows_apps_candidates(candidate_paths: &mut Vec<PathBuf>) {
    let Some(program_files_path) =
        env::var_os(PROGRAM_FILES_ENVIRONMENT_VARIABLE).map(PathBuf::from)
    else {
        return;
    };

    let windows_apps_path = program_files_path.join(WINDOWS_APPS_DIRECTORY_NAME);
    let Ok(package_entries) = fs::read_dir(windows_apps_path) else {
        return;
    };

    for package_entry_result in package_entries {
        let Ok(package_entry) = package_entry_result else {
            continue;
        };

        let package_path = package_entry.path();

        if !is_codex_package_directory(&package_path) {
            continue;
        }

        push_relative_candidate(candidate_paths, &package_path, CODEX_DESKTOP_RELATIVE_PATH);
        push_relative_candidate(candidate_paths, &package_path, CODEX_RESOURCE_RELATIVE_PATH);
    }
}

fn push_windows_apps_directories(directories: &mut Vec<PathBuf>) {
    let Some(program_files_path) =
        env::var_os(PROGRAM_FILES_ENVIRONMENT_VARIABLE).map(PathBuf::from)
    else {
        return;
    };

    let windows_apps_path = program_files_path.join(WINDOWS_APPS_DIRECTORY_NAME);
    let Ok(package_entries) = fs::read_dir(windows_apps_path) else {
        return;
    };

    for package_entry_result in package_entries {
        let Ok(package_entry) = package_entry_result else {
            continue;
        };

        let package_path = package_entry.path();
        if !is_codex_package_directory(&package_path) {
            continue;
        }

        push_directory_candidate(directories, package_path);
    }
}

fn is_codex_package_directory(path: &Path) -> bool {
    let Some(directory_name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };

    directory_name.starts_with(CODEX_PACKAGE_DIRECTORY_PREFIX)
        && directory_name.ends_with(CODEX_PACKAGE_DIRECTORY_SUFFIX)
}

fn collect_executables_in_directory(directory: &Path, executable_paths: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(directory) else {
        return;
    };

    for entry_result in entries {
        let Ok(entry) = entry_result else {
            continue;
        };

        let path = entry.path();
        if path.is_dir() {
            collect_executables_in_directory(&path, executable_paths);
        } else if path.is_file() && is_windows_executable(&path) {
            push_candidate(executable_paths, path);
        }
    }
}

fn is_windows_executable(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("exe"))
}

fn push_relative_candidate(
    candidate_paths: &mut Vec<PathBuf>,
    base_path: &Path,
    relative_parts: &[&str],
) {
    let path = build_relative_path(base_path, relative_parts);
    push_candidate(candidate_paths, path);
}

fn build_relative_path(base_path: &Path, relative_parts: &[&str]) -> PathBuf {
    relative_parts
        .iter()
        .fold(base_path.to_path_buf(), |mut current_path, part| {
            current_path.push(part);
            current_path
        })
}

fn push_directory_candidate(candidate_paths: &mut Vec<PathBuf>, path: PathBuf) {
    if !path.is_dir() {
        return;
    }

    push_candidate(candidate_paths, path);
}

fn push_candidate(candidate_paths: &mut Vec<PathBuf>, path: PathBuf) {
    if candidate_paths
        .iter()
        .any(|candidate_path| candidate_path == &path)
    {
        return;
    }

    candidate_paths.push(path);
}
