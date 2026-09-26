use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::platform::windows_app_compat::remove_codex_run_as_administrator_entries;
use crate::platform::windows_powershell::run_hidden_powershell_script;
use crate::platform::windows_scheduled_task::CODEX_ELEVATED_TASK_NAMES;

const CODEX_HOME_ENVIRONMENT_VARIABLE: &str = "CODEX_HOME";
const USER_PROFILE_ENVIRONMENT_VARIABLE: &str = "USERPROFILE";
const LOCAL_APP_DATA_ENVIRONMENT_VARIABLE: &str = "LOCALAPPDATA";
const PROGRAM_DATA_ENVIRONMENT_VARIABLE: &str = "ProgramData";
const CODEX_HOME_DIRECTORY_NAME: &str = ".codex";
const CODEX_CACHE_RUNTIME_RELATIVE_PATH: &[&str] = &[".cache", "codex-runtimes"];
const LOCAL_OPENAI_DIRECTORY_NAME: &str = "OpenAI";
const LOCAL_CODEX_DIRECTORY_NAME: &str = "Codex";
const DOCUMENTS_DIRECTORY_NAME: &str = "Documents";
const CODEX_DOCUMENTS_DIRECTORY_NAME: &str = "Codex";
const PACKAGES_DIRECTORY_NAME: &str = "Packages";
const CODEX_PACKAGE_FAMILY_PREFIX: &str = "OpenAI.Codex_";

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexUninstallReport {
    pub removed_file_count: u64,
    pub removed_directory_count: u64,
    pub freed_bytes: u64,
    pub package_removed: bool,
    pub targets: Vec<CodexUninstallTargetReport>,
    pub warnings: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexUninstallTargetReport {
    pub name: String,
    pub removed_file_count: u64,
    pub removed_directory_count: u64,
    pub freed_bytes: u64,
    pub details: String,
}

#[derive(Default)]
struct RemovalStats {
    removed_file_count: u64,
    removed_directory_count: u64,
    freed_bytes: u64,
}

struct UninstallReportBuilder {
    package_removed: bool,
    totals: RemovalStats,
    targets: Vec<CodexUninstallTargetReport>,
    warnings: Vec<String>,
}

impl UninstallReportBuilder {
    fn new() -> Self {
        Self {
            package_removed: false,
            totals: RemovalStats::default(),
            targets: Vec::new(),
            warnings: Vec::new(),
        }
    }

    fn push_target(
        &mut self,
        name: impl Into<String>,
        stats: RemovalStats,
        details: impl Into<String>,
    ) {
        self.totals.removed_file_count += stats.removed_file_count;
        self.totals.removed_directory_count += stats.removed_directory_count;
        self.totals.freed_bytes += stats.freed_bytes;

        self.targets.push(CodexUninstallTargetReport {
            name: name.into(),
            removed_file_count: stats.removed_file_count,
            removed_directory_count: stats.removed_directory_count,
            freed_bytes: stats.freed_bytes,
            details: details.into(),
        });
    }

    fn push_warning(&mut self, warning: impl Into<String>) {
        self.warnings.push(warning.into());
    }

    fn set_package_removed(&mut self, package_removed: bool) {
        self.package_removed = package_removed;
    }

    fn build(self) -> CodexUninstallReport {
        CodexUninstallReport {
            removed_file_count: self.totals.removed_file_count,
            removed_directory_count: self.totals.removed_directory_count,
            freed_bytes: self.totals.freed_bytes,
            package_removed: self.package_removed,
            targets: self.targets,
            warnings: self.warnings,
        }
    }
}

pub fn uninstall_codex_product() -> Result<CodexUninstallReport, String> {
    let mut report = UninstallReportBuilder::new();

    remove_codex_home(&mut report)?;
    remove_codex_runtime_cache(&mut report)?;
    remove_local_openai_codex(&mut report)?;
    remove_msix_package_data(&mut report)?;
    remove_program_data_codex(&mut report)?;
    remove_documents_codex_workspace(&mut report)?;
    remove_elevated_scheduled_task(&mut report);
    remove_app_compat_entries(&mut report);
    remove_appx_package(&mut report);

    Ok(report.build())
}

fn remove_codex_home(report: &mut UninstallReportBuilder) -> Result<(), String> {
    let Some(path) = locate_codex_home_path() else {
        report.push_target(
            "Codex home",
            RemovalStats::default(),
            "Pasta .codex nao encontrada.",
        );
        return Ok(());
    };

    let details = path.to_string_lossy().into_owned();
    let stats = remove_path_tree(&path)?;
    report.push_target("Codex home", stats, details);
    Ok(())
}

fn remove_codex_runtime_cache(report: &mut UninstallReportBuilder) -> Result<(), String> {
    let Some(user_profile) = env::var_os(USER_PROFILE_ENVIRONMENT_VARIABLE).map(PathBuf::from)
    else {
        report.push_warning("USERPROFILE ausente; cache codex-runtimes nao foi verificado.");
        return Ok(());
    };

    let path = build_relative_path(&user_profile, CODEX_CACHE_RUNTIME_RELATIVE_PATH);
    if !path.exists() {
        report.push_target(
            "Cache de runtimes",
            RemovalStats::default(),
            "Pasta .cache\\codex-runtimes nao encontrada.",
        );
        return Ok(());
    }

    let details = path.to_string_lossy().into_owned();
    let stats = remove_path_tree(&path)?;
    report.push_target("Cache de runtimes", stats, details);
    Ok(())
}

fn remove_local_openai_codex(report: &mut UninstallReportBuilder) -> Result<(), String> {
    let Some(local_app_data) = env::var_os(LOCAL_APP_DATA_ENVIRONMENT_VARIABLE).map(PathBuf::from)
    else {
        report.push_warning(
            "LOCALAPPDATA ausente; dados locais OpenAI\\Codex nao foram verificados.",
        );
        return Ok(());
    };

    let codex_path = local_app_data
        .join(LOCAL_OPENAI_DIRECTORY_NAME)
        .join(LOCAL_CODEX_DIRECTORY_NAME);
    if codex_path.exists() {
        let details = codex_path.to_string_lossy().into_owned();
        let stats = remove_path_tree(&codex_path)?;
        report.push_target("LocalAppData OpenAI\\Codex", stats, details);
    } else {
        report.push_target(
            "LocalAppData OpenAI\\Codex",
            RemovalStats::default(),
            "Pasta nao encontrada.",
        );
    }

    let openai_path = local_app_data.join(LOCAL_OPENAI_DIRECTORY_NAME);
    if openai_path.is_dir() {
        match remove_directory_if_empty(&openai_path) {
            Ok(true) => report.push_target(
                "LocalAppData OpenAI",
                RemovalStats {
                    removed_directory_count: 1,
                    ..RemovalStats::default()
                },
                "Pasta OpenAI removida por estar vazia.",
            ),
            Ok(false) => report
                .push_warning("LOCALAPPDATA\\OpenAI ainda contem outros dados e foi preservada."),
            Err(error) => report.push_warning(error),
        }
    }

    Ok(())
}

fn remove_msix_package_data(report: &mut UninstallReportBuilder) -> Result<(), String> {
    let Some(local_app_data) = env::var_os(LOCAL_APP_DATA_ENVIRONMENT_VARIABLE).map(PathBuf::from)
    else {
        return Ok(());
    };

    let packages_path = local_app_data.join(PACKAGES_DIRECTORY_NAME);
    if !packages_path.is_dir() {
        report.push_target(
            "Dados do pacote MSIX",
            RemovalStats::default(),
            "Pasta Packages nao encontrada.",
        );
        return Ok(());
    }

    let mut combined = RemovalStats::default();
    let mut removed_names = Vec::new();

    for entry in fs::read_dir(&packages_path)
        .map_err(|error| format!("Falha ao ler {}: {error}", packages_path.to_string_lossy()))?
    {
        let entry = entry.map_err(|error| {
            format!(
                "Falha ao inspecionar entrada em {}: {error}",
                packages_path.to_string_lossy()
            )
        })?;
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };

        if !name.starts_with(CODEX_PACKAGE_FAMILY_PREFIX) {
            continue;
        }

        let stats = remove_path_tree(&path)?;
        combined.removed_file_count += stats.removed_file_count;
        combined.removed_directory_count += stats.removed_directory_count;
        combined.freed_bytes += stats.freed_bytes;
        removed_names.push(name.to_owned());
    }

    let details = if removed_names.is_empty() {
        "Nenhum pacote OpenAI.Codex_* encontrado em Packages.".to_owned()
    } else {
        removed_names.join(", ")
    };
    report.push_target("Dados do pacote MSIX", combined, details);
    Ok(())
}

fn remove_program_data_codex(report: &mut UninstallReportBuilder) -> Result<(), String> {
    let Some(program_data) = env::var_os(PROGRAM_DATA_ENVIRONMENT_VARIABLE).map(PathBuf::from)
    else {
        report.push_warning("ProgramData ausente; requirements de sistema nao foram verificados.");
        return Ok(());
    };

    let codex_path = program_data
        .join(LOCAL_OPENAI_DIRECTORY_NAME)
        .join(LOCAL_CODEX_DIRECTORY_NAME);
    if codex_path.exists() {
        let details = codex_path.to_string_lossy().into_owned();
        let stats = remove_path_tree(&codex_path)?;
        report.push_target("ProgramData OpenAI\\Codex", stats, details);
    } else {
        report.push_target(
            "ProgramData OpenAI\\Codex",
            RemovalStats::default(),
            "Pasta nao encontrada.",
        );
    }

    let openai_path = program_data.join(LOCAL_OPENAI_DIRECTORY_NAME);
    if openai_path.is_dir() {
        match remove_directory_if_empty(&openai_path) {
            Ok(true) => report.push_target(
                "ProgramData OpenAI",
                RemovalStats {
                    removed_directory_count: 1,
                    ..RemovalStats::default()
                },
                "Pasta OpenAI removida por estar vazia.",
            ),
            Ok(false) => report
                .push_warning("ProgramData\\OpenAI ainda contem outros dados e foi preservada."),
            Err(error) => report.push_warning(error),
        }
    }

    Ok(())
}

fn remove_documents_codex_workspace(report: &mut UninstallReportBuilder) -> Result<(), String> {
    let Some(user_profile) = env::var_os(USER_PROFILE_ENVIRONMENT_VARIABLE).map(PathBuf::from)
    else {
        return Ok(());
    };

    let path = user_profile
        .join(DOCUMENTS_DIRECTORY_NAME)
        .join(CODEX_DOCUMENTS_DIRECTORY_NAME);
    if !path.exists() {
        report.push_target(
            "Workspace Documents\\Codex",
            RemovalStats::default(),
            "Pasta nao encontrada.",
        );
        return Ok(());
    }

    let details = path.to_string_lossy().into_owned();
    let stats = remove_path_tree(&path)?;
    report.push_target("Workspace Documents\\Codex", stats, details);
    Ok(())
}

fn remove_elevated_scheduled_task(report: &mut UninstallReportBuilder) {
    let task_names = CODEX_ELEVATED_TASK_NAMES
        .iter()
        .map(|task_name| quote_powershell_single_quoted_string(task_name))
        .collect::<Vec<String>>()
        .join(", ");
    let script = format!(
        r#"
$ErrorActionPreference = 'Stop'
$taskNames = @({task_names})
$taskNotFoundHresult = -2147024894
$removedTaskCount = 0
$taskService = New-Object -ComObject 'Schedule.Service'
$taskService.Connect()
$rootFolder = $taskService.GetFolder('\')

foreach ($taskName in $taskNames) {{
    try {{
        $rootFolder.DeleteTask($taskName, 0)
        $removedTaskCount++
    }} catch {{
        if ($_.Exception.HResult -ne $taskNotFoundHresult) {{
            throw
        }}
    }}
}}

if ($removedTaskCount -eq 0) {{
    Write-Output 'missing'
    exit 0
}}
Write-Output ("removed:{{0}}" -f $removedTaskCount)
"#
    );

    match run_powershell_script(&script) {
        Ok(output) if output.trim() == "missing" => report.push_target(
            "Tarefa agendada elevada",
            RemovalStats::default(),
            "Nenhuma tarefa elevada do Codex existia.",
        ),
        Ok(output) => {
            let normalized_output = output.trim();
            let removed_task_count = normalized_output
                .strip_prefix("removed:")
                .and_then(|value| value.parse::<usize>().ok());

            if let Some(removed_task_count) = removed_task_count {
                report.push_target(
                    "Tarefa agendada elevada",
                    RemovalStats::default(),
                    format!("{removed_task_count} tarefa(s) elevada(s) do Codex removida(s)."),
                );
            } else {
                report.push_warning(format!(
                    "Remocao da tarefa agendada retornou saida inesperada: {output}"
                ));
            }
        }
        Err(error) => report.push_warning(format!(
            "Nao foi possivel remover a tarefa agendada elevada: {error}"
        )),
    }
}

fn remove_app_compat_entries(report: &mut UninstallReportBuilder) {
    match remove_codex_run_as_administrator_entries() {
        Ok(removed_paths) if removed_paths.is_empty() => report.push_target(
            "AppCompat RUNASADMIN",
            RemovalStats::default(),
            "Nenhuma entrada Codex encontrada no Registry.",
        ),
        Ok(removed_paths) => report.push_target(
            "AppCompat RUNASADMIN",
            RemovalStats::default(),
            format!("{} entrada(s) removida(s).", removed_paths.len()),
        ),
        Err(error) => report.push_warning(format!(
            "Nao foi possivel limpar entradas AppCompat: {error}"
        )),
    }
}

fn remove_appx_package(report: &mut UninstallReportBuilder) {
    let script = r#"
$ErrorActionPreference = 'Stop'
$packages = Get-AppxPackage -Name 'OpenAI.Codex' -ErrorAction SilentlyContinue
if ($null -eq $packages) {
    Write-Output 'missing'
    exit 0
}
$packages | ForEach-Object {
    Remove-AppxPackage -Package $_.PackageFullName
}
Write-Output 'removed'
"#;

    match run_powershell_script(script) {
        Ok(output) if output.trim() == "removed" => {
            report.set_package_removed(true);
            report.push_target(
                "Pacote MSIX OpenAI.Codex",
                RemovalStats::default(),
                "Pacote desinstalado via Remove-AppxPackage.",
            );
        }
        Ok(output) if output.trim() == "missing" => report.push_target(
            "Pacote MSIX OpenAI.Codex",
            RemovalStats::default(),
            "Pacote nao estava instalado para o usuario atual.",
        ),
        Ok(output) => report.push_warning(format!(
            "Desinstalacao do pacote MSIX retornou saida inesperada: {output}"
        )),
        Err(error) => report.push_warning(format!(
            "Nao foi possivel desinstalar o pacote MSIX OpenAI.Codex: {error}"
        )),
    }
}

fn locate_codex_home_path() -> Option<PathBuf> {
    if let Some(path) = env::var_os(CODEX_HOME_ENVIRONMENT_VARIABLE)
        .map(PathBuf::from)
        .filter(|path| path.exists())
    {
        return Some(path);
    }

    env::var_os(USER_PROFILE_ENVIRONMENT_VARIABLE)
        .map(PathBuf::from)
        .map(|profile| profile.join(CODEX_HOME_DIRECTORY_NAME))
        .filter(|path| path.exists())
}

fn remove_path_tree(path: &Path) -> Result<RemovalStats, String> {
    if !path.exists() {
        return Ok(RemovalStats::default());
    }

    let metadata = fs::symlink_metadata(path)
        .map_err(|error| format!("Falha ao inspecionar {}: {error}", path.to_string_lossy()))?;

    if metadata.file_type().is_symlink() {
        let mut stats = RemovalStats::default();
        remove_symlink(path, &mut stats)?;
        return Ok(stats);
    }

    if metadata.is_file() {
        let mut stats = RemovalStats::default();
        stats.freed_bytes += metadata.len();
        fs::remove_file(path)
            .map_err(|error| format!("Falha ao remover {}: {error}", path.to_string_lossy()))?;
        stats.removed_file_count += 1;
        return Ok(stats);
    }

    if !metadata.is_dir() {
        return Err(format!(
            "Tipo de caminho nao suportado em {}.",
            path.to_string_lossy()
        ));
    }

    let mut stats = RemovalStats::default();
    remove_directory_contents_recursive(path, &mut stats)?;
    fs::remove_dir(path)
        .map_err(|error| format!("Falha ao remover pasta {}: {error}", path.to_string_lossy()))?;
    stats.removed_directory_count += 1;
    Ok(stats)
}

fn remove_directory_contents_recursive(
    directory_path: &Path,
    stats: &mut RemovalStats,
) -> Result<(), String> {
    for entry in fs::read_dir(directory_path)
        .map_err(|error| format!("Falha ao ler {}: {error}", directory_path.to_string_lossy()))?
    {
        let entry = entry.map_err(|error| {
            format!(
                "Falha ao inspecionar entrada em {}: {error}",
                directory_path.to_string_lossy()
            )
        })?;
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path)
            .map_err(|error| format!("Falha ao inspecionar {}: {error}", path.to_string_lossy()))?;

        if metadata.file_type().is_symlink() {
            remove_symlink(&path, stats)?;
            continue;
        }

        if metadata.is_dir() {
            remove_directory_contents_recursive(&path, stats)?;
            fs::remove_dir(&path).map_err(|error| {
                format!("Falha ao remover pasta {}: {error}", path.to_string_lossy())
            })?;
            stats.removed_directory_count += 1;
            continue;
        }

        stats.freed_bytes += metadata.len();
        fs::remove_file(&path)
            .map_err(|error| format!("Falha ao remover {}: {error}", path.to_string_lossy()))?;
        stats.removed_file_count += 1;
    }

    Ok(())
}

fn remove_symlink(path: &Path, stats: &mut RemovalStats) -> Result<(), String> {
    match fs::remove_file(path) {
        Ok(()) => {
            stats.removed_file_count += 1;
            Ok(())
        }
        Err(file_error) => match fs::remove_dir(path) {
            Ok(()) => {
                stats.removed_directory_count += 1;
                Ok(())
            }
            Err(directory_error) => Err(format!(
                "Falha ao remover symlink {} como arquivo ({file_error}) ou pasta ({directory_error}).",
                path.to_string_lossy()
            )),
        },
    }
}

fn remove_directory_if_empty(path: &Path) -> Result<bool, String> {
    let mut entries = fs::read_dir(path)
        .map_err(|error| format!("Falha ao ler {}: {error}", path.to_string_lossy()))?;
    if entries.next().is_some() {
        return Ok(false);
    }

    fs::remove_dir(path).map_err(|error| {
        format!(
            "Falha ao remover pasta vazia {}: {error}",
            path.to_string_lossy()
        )
    })?;
    Ok(true)
}

fn run_powershell_script(script: &str) -> Result<String, String> {
    let output = run_hidden_powershell_script(script)
        .map_err(|error| format!("Falha ao iniciar Windows PowerShell: {error}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();

    if !output.status.success() {
        return Err(format!(
            "PowerShell falhou com status {}. stdout: {} stderr: {}",
            output.status,
            empty_if_blank(&stdout),
            empty_if_blank(&stderr)
        ));
    }

    Ok(stdout)
}

fn quote_powershell_single_quoted_string(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

fn empty_if_blank(value: &str) -> &str {
    if value.is_empty() { "<empty>" } else { value }
}

fn build_relative_path(base_path: &Path, relative_parts: &[&str]) -> PathBuf {
    relative_parts
        .iter()
        .fold(base_path.to_path_buf(), |mut current_path, part| {
            current_path.push(part);
            current_path
        })
}
