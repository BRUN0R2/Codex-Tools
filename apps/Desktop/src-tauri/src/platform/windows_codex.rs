use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const CODEX_DESKTOP_EXECUTABLE_FILE_NAME: &str = "Codex.exe";
const CODEX_EXECUTABLE_FILE_NAME: &str = "codex.exe";
const CODEX_ALIAS_FILE_NAME: &str = "codex";
const LOCAL_APP_DATA_ENVIRONMENT_VARIABLE: &str = "LOCALAPPDATA";
const PATH_ENVIRONMENT_VARIABLE: &str = "PATH";
const PROGRAM_FILES_ENVIRONMENT_VARIABLE: &str = "ProgramFiles";
const WINDOWS_APPS_DIRECTORY_NAME: &str = "WindowsApps";
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

pub fn locate_codex_installation() -> CodexInstallation {
    let checked_paths = collect_codex_candidate_paths();
    let executable_path = checked_paths.iter().find(|path| path.is_file()).cloned();

    CodexInstallation {
        executable_path,
        checked_paths,
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

fn is_codex_package_directory(path: &Path) -> bool {
    let Some(directory_name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };

    directory_name.starts_with(CODEX_PACKAGE_DIRECTORY_PREFIX)
        && directory_name.ends_with(CODEX_PACKAGE_DIRECTORY_SUFFIX)
}

fn push_relative_candidate(
    candidate_paths: &mut Vec<PathBuf>,
    base_path: &Path,
    relative_parts: &[&str],
) {
    let path = relative_parts
        .iter()
        .fold(base_path.to_path_buf(), |mut current_path, part| {
            current_path.push(part);
            current_path
        });

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
