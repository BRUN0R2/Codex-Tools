use std::cmp::Ordering;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const CODEX_DESKTOP_EXECUTABLE_FILE_NAMES: &[&str] = &["ChatGPT.exe", "Codex.exe"];
const CODEX_DESKTOP_APPLICATION_ID: &str = "App";
const PROGRAM_FILES_ENVIRONMENT_VARIABLE: &str = "ProgramFiles";
const WINDOWS_APPS_DIRECTORY_NAME: &str = "WindowsApps";
const CODEX_PACKAGE_DIRECTORY_PREFIX: &str = "OpenAI.Codex_";
const CODEX_PACKAGE_DIRECTORY_SUFFIX: &str = "__2p2nqsd0c76g0";

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
    let checked_paths = collect_codex_desktop_candidate_paths();
    let executable_path = checked_paths.iter().find(|path| path.is_file()).cloned();

    CodexInstallation {
        executable_path,
        checked_paths,
    }
}

pub fn codex_desktop_application_user_model_id(executable_path: &Path) -> Result<String, String> {
    let package_directory = executable_path
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| "Codex executable is not inside a package directory.".to_owned())?;

    if !is_codex_package_directory(package_directory) {
        return Err(format!(
            "Codex executable is not inside a recognized package directory: {}",
            package_directory.display()
        ));
    }

    Ok(format!(
        "{}!{CODEX_DESKTOP_APPLICATION_ID}",
        codex_package_family_name()
    ))
}

pub fn is_codex_app_server_executable_path(executable_path: &str) -> bool {
    let normalized_path = normalize_windows_path(executable_path);

    normalized_path.ends_with("\\app\\resources\\codex.exe")
        || (normalized_path.contains("\\openai\\codex\\bin\\")
            && normalized_path.ends_with("\\codex.exe"))
}

pub fn is_codex_desktop_executable_path(executable_path: &str) -> bool {
    let normalized_path = normalize_windows_path(executable_path);

    normalized_path.contains("\\windowsapps\\openai.codex_")
        && (normalized_path.ends_with("\\app\\chatgpt.exe")
            || normalized_path.ends_with("\\app\\codex.exe"))
}

fn codex_package_family_name() -> String {
    format!(
        "{CODEX_PACKAGE_DIRECTORY_PREFIX}{}",
        CODEX_PACKAGE_DIRECTORY_SUFFIX.trim_start_matches('_')
    )
}

fn normalize_windows_path(executable_path: &str) -> String {
    executable_path.replace('/', "\\").to_ascii_lowercase()
}

fn collect_codex_desktop_candidate_paths() -> Vec<PathBuf> {
    let mut candidate_paths = Vec::new();

    for package_directory in collect_windows_apps_package_directories() {
        push_codex_desktop_candidates(&mut candidate_paths, &package_directory);
    }

    candidate_paths
}

fn push_codex_desktop_candidates(candidate_paths: &mut Vec<PathBuf>, base_path: &Path) {
    for executable_file_name in CODEX_DESKTOP_EXECUTABLE_FILE_NAMES {
        push_relative_candidate(candidate_paths, base_path, &["app", executable_file_name]);
    }
}

fn collect_windows_apps_package_directories() -> Vec<PathBuf> {
    let Some(program_files_path) =
        env::var_os(PROGRAM_FILES_ENVIRONMENT_VARIABLE).map(PathBuf::from)
    else {
        return Vec::new();
    };

    let windows_apps_path = program_files_path.join(WINDOWS_APPS_DIRECTORY_NAME);
    let Ok(package_entries) = fs::read_dir(windows_apps_path) else {
        return Vec::new();
    };

    let mut package_directories = package_entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_dir() && is_codex_package_directory(path))
        .collect::<Vec<PathBuf>>();

    package_directories
        .sort_by(|left, right| compare_codex_package_directories(left.as_path(), right.as_path()));
    package_directories
}

fn compare_codex_package_directories(left: &Path, right: &Path) -> Ordering {
    match (codex_package_version(left), codex_package_version(right)) {
        (Some(left_version), Some(right_version)) => right_version
            .cmp(&left_version)
            .then_with(|| right.cmp(left)),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => right.cmp(left),
    }
}

fn codex_package_version(path: &Path) -> Option<Vec<u64>> {
    let directory_name = path.file_name()?.to_str()?;
    let package_identity = directory_name
        .strip_prefix(CODEX_PACKAGE_DIRECTORY_PREFIX)?
        .strip_suffix(CODEX_PACKAGE_DIRECTORY_SUFFIX)?;
    let version = package_identity.split('_').next()?;
    let version_parts = version
        .split('.')
        .map(str::parse::<u64>)
        .collect::<Result<Vec<u64>, _>>()
        .ok()?;

    (!version_parts.is_empty()).then_some(version_parts)
}

fn is_codex_package_directory(path: &Path) -> bool {
    codex_package_version(path).is_some()
}

fn push_relative_candidate(
    candidate_paths: &mut Vec<PathBuf>,
    base_path: &Path,
    relative_parts: &[&str],
) {
    push_candidate(
        candidate_paths,
        build_relative_path(base_path, relative_parts),
    );
}

fn build_relative_path(base_path: &Path, relative_parts: &[&str]) -> PathBuf {
    relative_parts
        .iter()
        .fold(base_path.to_path_buf(), |mut current_path, part| {
            current_path.push(part);
            current_path
        })
}

fn push_candidate(candidate_paths: &mut Vec<PathBuf>, path: PathBuf) {
    if candidate_paths
        .iter()
        .any(|candidate_path| paths_match(candidate_path, &path))
    {
        return;
    }

    candidate_paths.push(path);
}

fn paths_match(left: &Path, right: &Path) -> bool {
    left.to_string_lossy()
        .eq_ignore_ascii_case(&right.to_string_lossy())
}

#[cfg(test)]
mod tests {
    use super::{
        codex_desktop_application_user_model_id, codex_package_version,
        compare_codex_package_directories, is_codex_app_server_executable_path,
        is_codex_desktop_executable_path,
    };
    use std::cmp::Ordering;
    use std::path::{Path, PathBuf};

    #[test]
    fn package_versions_are_sorted_numerically_in_descending_order() {
        let older = PathBuf::from("OpenAI.Codex_26.99.0.0_x64__2p2nqsd0c76g0");
        let newer = PathBuf::from("OpenAI.Codex_26.100.0.0_x64__2p2nqsd0c76g0");

        assert_eq!(
            compare_codex_package_directories(&newer, &older),
            Ordering::Less
        );
        assert_eq!(
            codex_package_version(Path::new("OpenAI.Codex_26.527.7698.0_x64__2p2nqsd0c76g0")),
            Some(vec![26, 527, 7698, 0])
        );
    }

    #[test]
    fn app_server_detection_accepts_packaged_and_hashed_runtime_paths() {
        assert!(is_codex_app_server_executable_path(
            r"C:\Program Files\WindowsApps\OpenAI.Codex_26.527.7698.0_x64__2p2nqsd0c76g0\app\resources\codex.exe"
        ));
        assert!(is_codex_app_server_executable_path(
            r"C:\Users\Example\AppData\Local\OpenAI\Codex\bin\716dda49c14d31a0\codex.exe"
        ));
    }

    #[test]
    fn desktop_detection_accepts_current_and_legacy_packaged_executables() {
        assert!(is_codex_desktop_executable_path(
            r"C:\Program Files\WindowsApps\OpenAI.Codex_26.707.3748.0_x64__2p2nqsd0c76g0\app\ChatGPT.exe"
        ));
        assert!(is_codex_desktop_executable_path(
            r"C:\Program Files\WindowsApps\OpenAI.Codex_26.527.7698.0_x64__2p2nqsd0c76g0\app\Codex.exe"
        ));
        assert!(!is_codex_desktop_executable_path(
            r"C:\Program Files\ChatGPT\ChatGPT.exe"
        ));
    }

    #[test]
    fn desktop_activation_uses_the_package_family_application_id() {
        let executable_path = Path::new(
            r"C:\Program Files\WindowsApps\OpenAI.Codex_26.917.9434.0_x64__2p2nqsd0c76g0\app\ChatGPT.exe",
        );

        assert_eq!(
            codex_desktop_application_user_model_id(executable_path).expect("package identity"),
            "OpenAI.Codex_2p2nqsd0c76g0!App"
        );
        assert!(
            codex_desktop_application_user_model_id(Path::new(
                r"C:\Program Files\ChatGPT\ChatGPT.exe"
            ))
            .is_err()
        );
    }
}
