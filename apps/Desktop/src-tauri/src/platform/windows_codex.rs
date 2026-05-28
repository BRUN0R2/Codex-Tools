use std::env;
use std::path::{Path, PathBuf};

const CODEX_EXECUTABLE_FILE_NAME: &str = "codex.exe";
const CODEX_ALIAS_FILE_NAME: &str = "codex";
const LOCAL_APP_DATA_ENVIRONMENT_VARIABLE: &str = "LOCALAPPDATA";
const PATH_ENVIRONMENT_VARIABLE: &str = "PATH";

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
