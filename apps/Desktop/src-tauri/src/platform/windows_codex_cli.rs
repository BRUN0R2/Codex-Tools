use std::env;
use std::path::{Path, PathBuf};

const LOCAL_APP_DATA_ENVIRONMENT_VARIABLE: &str = "LOCALAPPDATA";
const PATH_ENVIRONMENT_VARIABLE: &str = "PATH";
const CODEX_CLI_RELATIVE_PATH: &[&str] = &["Programs", "OpenAI", "Codex", "bin", "codex.exe"];
const CODEX_CLI_EXECUTABLE_NAME: &str = "codex.exe";

pub struct CodexCliInstallation {
    executable_path: Option<PathBuf>,
    checked_paths: Vec<PathBuf>,
}

impl CodexCliInstallation {
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

pub fn locate_codex_cli_installation() -> CodexCliInstallation {
    let checked_paths = collect_codex_cli_candidate_paths();
    let executable_path = checked_paths.iter().find(|path| path.is_file()).cloned();
    CodexCliInstallation {
        executable_path,
        checked_paths,
    }
}

fn collect_codex_cli_candidate_paths() -> Vec<PathBuf> {
    let mut candidate_paths = Vec::new();

    if let Some(local_app_data) = env::var_os(LOCAL_APP_DATA_ENVIRONMENT_VARIABLE) {
        let standard_installation =
            CODEX_CLI_RELATIVE_PATH
                .iter()
                .fold(PathBuf::from(local_app_data), |mut path, part| {
                    path.push(part);
                    path
                });
        push_candidate(&mut candidate_paths, standard_installation);
    }

    if let Some(path_environment) = env::var_os(PATH_ENVIRONMENT_VARIABLE) {
        for directory in env::split_paths(&path_environment).filter(|path| path.is_absolute()) {
            push_candidate(
                &mut candidate_paths,
                directory.join(CODEX_CLI_EXECUTABLE_NAME),
            );
        }
    }

    candidate_paths
}

fn push_candidate(candidate_paths: &mut Vec<PathBuf>, candidate: PathBuf) {
    if candidate_paths.iter().all(|existing| {
        !existing
            .to_string_lossy()
            .eq_ignore_ascii_case(&candidate.to_string_lossy())
    }) {
        candidate_paths.push(candidate);
    }
}

#[cfg(test)]
mod tests {
    use super::{CODEX_CLI_EXECUTABLE_NAME, CODEX_CLI_RELATIVE_PATH, push_candidate};
    use std::path::PathBuf;

    #[test]
    fn expected_user_install_path_ends_in_the_cli_executable() {
        let path = CODEX_CLI_RELATIVE_PATH.iter().fold(
            PathBuf::from(r"C:\Users\Example\AppData\Local"),
            |mut path, part| {
                path.push(part);
                path
            },
        );

        assert!(path.ends_with(CODEX_CLI_EXECUTABLE_NAME));
        assert!(path.ends_with(r"OpenAI\Codex\bin\codex.exe"));
    }

    #[test]
    fn candidate_paths_are_deduplicated_case_insensitively() {
        let mut paths = vec![PathBuf::from(r"C:\Tools\Codex.exe")];
        push_candidate(&mut paths, PathBuf::from(r"c:\tools\codex.exe"));
        assert_eq!(paths.len(), 1);
    }
}
