use std::cmp::Ordering;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const CODEX_DESKTOP_EXECUTABLE_FILE_NAMES: &[&str] = &["ChatGPT.exe", "Codex.exe"];
const CODEX_EXECUTABLE_FILE_NAME: &str = "codex.exe";
pub const CODEX_DO_NOT_DE_ELEVATE_ARGUMENT: &str = "--do-not-de-elevate";
const LOCAL_APP_DATA_ENVIRONMENT_VARIABLE: &str = "LOCALAPPDATA";
const PATH_ENVIRONMENT_VARIABLE: &str = "PATH";
const PROGRAM_FILES_ENVIRONMENT_VARIABLE: &str = "ProgramFiles";
const WINDOWS_APPS_DIRECTORY_NAME: &str = "WindowsApps";
const CODEX_PACKAGE_DIRECTORY_PREFIX: &str = "OpenAI.Codex_";
const CODEX_PACKAGE_DIRECTORY_SUFFIX: &str = "__2p2nqsd0c76g0";
const CODEX_PACKAGE_FAMILY_DIRECTORY_NAME: &str = "OpenAI.Codex_2p2nqsd0c76g0";
const CODEX_RESOURCE_RELATIVE_PATH: &[&str] = &["app", "resources", CODEX_EXECUTABLE_FILE_NAME];
const LOCAL_CODEX_BIN_RELATIVE_PATH: &[&str] = &["OpenAI", "Codex", "bin"];
const PACKAGE_CODEX_BIN_RELATIVE_PATH: &[&str] = &[
    "Packages",
    CODEX_PACKAGE_FAMILY_DIRECTORY_NAME,
    "LocalCache",
    "Local",
    "OpenAI",
    "Codex",
    "bin",
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
    let checked_paths = collect_codex_desktop_candidate_paths();
    let executable_path = checked_paths.iter().find(|path| path.is_file()).cloned();

    CodexInstallation {
        executable_path,
        checked_paths,
    }
}

pub fn collect_codex_executable_inventory() -> CodexExecutableInventory {
    let mut executable_paths = Vec::new();
    let mut scanned_directories = Vec::new();

    for package_directory in collect_windows_apps_package_directories() {
        push_directory_candidate(&mut scanned_directories, package_directory.clone());
        push_existing_codex_desktop_candidates(&mut executable_paths, &package_directory);
        push_existing_relative_candidate(
            &mut executable_paths,
            &package_directory,
            CODEX_RESOURCE_RELATIVE_PATH,
        );
    }

    if let Some(local_app_data_path) =
        env::var_os(LOCAL_APP_DATA_ENVIRONMENT_VARIABLE).map(PathBuf::from)
    {
        collect_runtime_bin_executables(
            &build_relative_path(&local_app_data_path, LOCAL_CODEX_BIN_RELATIVE_PATH),
            &mut scanned_directories,
            &mut executable_paths,
        );
        collect_runtime_bin_executables(
            &build_relative_path(&local_app_data_path, PACKAGE_CODEX_BIN_RELATIVE_PATH),
            &mut scanned_directories,
            &mut executable_paths,
        );
    }

    push_path_codex_executables(&mut executable_paths);

    CodexExecutableInventory {
        executable_paths,
        scanned_directories,
    }
}

pub fn is_codex_app_server_executable_path(executable_path: &str) -> bool {
    let normalized_path = executable_path.replace('/', "\\").to_ascii_lowercase();

    normalized_path.ends_with("\\app\\resources\\codex.exe")
        || (normalized_path.contains("\\openai\\codex\\bin\\")
            && normalized_path.ends_with("\\codex.exe"))
}

pub fn is_codex_desktop_executable_path(executable_path: &str) -> bool {
    let normalized_path = executable_path.replace('/', "\\").to_ascii_lowercase();

    normalized_path.contains("\\windowsapps\\openai.codex_")
        && (normalized_path.ends_with("\\app\\chatgpt.exe")
            || normalized_path.ends_with("\\app\\codex.exe"))
}

fn collect_codex_desktop_candidate_paths() -> Vec<PathBuf> {
    let mut candidate_paths = Vec::new();

    for package_directory in collect_windows_apps_package_directories() {
        push_codex_desktop_candidates(&mut candidate_paths, &package_directory);
    }

    candidate_paths
}

fn push_existing_codex_desktop_candidates(candidate_paths: &mut Vec<PathBuf>, base_path: &Path) {
    for executable_file_name in CODEX_DESKTOP_EXECUTABLE_FILE_NAMES {
        push_existing_relative_candidate(
            candidate_paths,
            base_path,
            &["app", executable_file_name],
        );
    }
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

fn collect_runtime_bin_executables(
    bin_directory: &Path,
    scanned_directories: &mut Vec<PathBuf>,
    executable_paths: &mut Vec<PathBuf>,
) {
    if !bin_directory.is_dir() {
        return;
    }

    push_directory_candidate(scanned_directories, bin_directory.to_path_buf());
    push_existing_runtime_executable(bin_directory, executable_paths);

    let Ok(entries) = fs::read_dir(bin_directory) else {
        return;
    };

    for nested_directory in entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
    {
        push_existing_runtime_executable(&nested_directory, executable_paths);
    }
}

fn push_existing_runtime_executable(directory: &Path, executable_paths: &mut Vec<PathBuf>) {
    let candidate_path = directory.join(CODEX_EXECUTABLE_FILE_NAME);
    if candidate_path.is_file() {
        push_candidate(executable_paths, candidate_path);
    }
}

fn push_path_codex_executables(executable_paths: &mut Vec<PathBuf>) {
    let Some(path_value) = env::var_os(PATH_ENVIRONMENT_VARIABLE) else {
        return;
    };

    for directory in env::split_paths(&path_value) {
        let candidate_path = directory.join(CODEX_EXECUTABLE_FILE_NAME);
        if candidate_path.is_file() {
            push_candidate(executable_paths, candidate_path);
        }
    }
}

fn push_existing_relative_candidate(
    candidate_paths: &mut Vec<PathBuf>,
    base_path: &Path,
    relative_parts: &[&str],
) {
    let candidate_path = build_relative_path(base_path, relative_parts);
    if candidate_path.is_file() {
        push_candidate(candidate_paths, candidate_path);
    }
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

fn push_directory_candidate(candidate_paths: &mut Vec<PathBuf>, path: PathBuf) {
    if path.is_dir() {
        push_candidate(candidate_paths, path);
    }
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
        codex_package_version, compare_codex_package_directories,
        is_codex_app_server_executable_path, is_codex_desktop_executable_path,
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
}
