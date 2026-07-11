use std::collections::HashSet;
use std::env;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::time::Duration;

use rusqlite::Connection;
use serde::Serialize;
use serde_json::Value;

const CODEX_HOME_ENVIRONMENT_VARIABLE: &str = "CODEX_HOME";
const USER_PROFILE_ENVIRONMENT_VARIABLE: &str = "USERPROFILE";
const CODEX_HOME_DIRECTORY_NAME: &str = ".codex";
const SESSION_INDEX_FILE_NAME: &str = "session_index.jsonl";
const ACTIVE_STATE_DATABASE_RELATIVE_PATH: &[&str] = &["sqlite", "state_5.sqlite"];
const GLOBAL_STATE_FILE_NAMES: &[&str] =
    &[".codex-global-state.json", ".codex-global-state.json.bak"];
const ARCHIVED_SESSIONS_DIRECTORY_NAME: &str = "archived_sessions";
const ATTACHMENTS_DIRECTORY_NAME: &str = "attachments";
const SESSIONS_DIRECTORY_NAME: &str = "sessions";
const CACHE_DIRECTORY_NAME: &str = "cache";
const TEMPORARY_DIRECTORY_NAMES: &[&str] = &[".tmp", "tmp"];
const DATABASE_BUSY_TIMEOUT: Duration = Duration::from_secs(5);
const THREAD_ID_LENGTH: usize = 36;
const CODEX_THREAD_ID_PREFIX: &str = "019";

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexCleanupReport {
    pub codex_home_path: String,
    pub removed_thread_count: u64,
    pub removed_global_state_reference_count: u64,
    pub removed_file_count: u64,
    pub removed_directory_count: u64,
    pub freed_bytes: u64,
    pub targets: Vec<CodexCleanupTargetReport>,
    pub warnings: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexCleanupTargetReport {
    pub name: String,
    pub removed_thread_count: u64,
    pub removed_global_state_reference_count: u64,
    pub removed_file_count: u64,
    pub removed_directory_count: u64,
    pub freed_bytes: u64,
}

#[derive(Default)]
struct CleanupStats {
    removed_thread_count: u64,
    removed_global_state_reference_count: u64,
    removed_file_count: u64,
    removed_directory_count: u64,
    freed_bytes: u64,
}

struct CleanupReportBuilder {
    codex_home_path: PathBuf,
    removed_thread_count: u64,
    totals: CleanupStats,
    targets: Vec<CodexCleanupTargetReport>,
    warnings: Vec<String>,
}

impl CleanupReportBuilder {
    fn new(codex_home_path: PathBuf, removed_thread_count: u64) -> Self {
        Self {
            codex_home_path,
            removed_thread_count,
            totals: CleanupStats::default(),
            targets: Vec::new(),
            warnings: Vec::new(),
        }
    }

    fn push_target(&mut self, name: impl Into<String>, stats: CleanupStats) {
        self.totals.removed_global_state_reference_count +=
            stats.removed_global_state_reference_count;
        self.totals.removed_file_count += stats.removed_file_count;
        self.totals.removed_directory_count += stats.removed_directory_count;
        self.totals.freed_bytes += stats.freed_bytes;

        self.targets.push(CodexCleanupTargetReport {
            name: name.into(),
            removed_thread_count: stats.removed_thread_count,
            removed_global_state_reference_count: stats.removed_global_state_reference_count,
            removed_file_count: stats.removed_file_count,
            removed_directory_count: stats.removed_directory_count,
            freed_bytes: stats.freed_bytes,
        });
    }

    fn push_warning(&mut self, warning: impl Into<String>) {
        self.warnings.push(warning.into());
    }

    fn build(self) -> CodexCleanupReport {
        CodexCleanupReport {
            codex_home_path: self.codex_home_path.to_string_lossy().into_owned(),
            removed_thread_count: self.removed_thread_count,
            removed_global_state_reference_count: self.totals.removed_global_state_reference_count,
            removed_file_count: self.totals.removed_file_count,
            removed_directory_count: self.totals.removed_directory_count,
            freed_bytes: self.totals.freed_bytes,
            targets: self.targets,
            warnings: self.warnings,
        }
    }
}

pub fn clean_codex_workspace() -> Result<CodexCleanupReport, String> {
    let codex_home_path = locate_codex_home_path()?;
    let active_database_path =
        build_relative_path(&codex_home_path, ACTIVE_STATE_DATABASE_RELATIVE_PATH);
    let mut stale_thread_ids = collect_stale_thread_ids(&codex_home_path, &active_database_path)?;
    stale_thread_ids.extend(collect_global_state_thread_ids(&codex_home_path)?);
    let removed_thread_count = stale_thread_ids.len() as u64;
    let mut report = CleanupReportBuilder::new(codex_home_path.clone(), removed_thread_count);

    report.push_target("Banco ativo", clean_active_database(&active_database_path)?);
    report.push_target("Indice de sessoes", clean_session_index(&codex_home_path)?);
    report.push_target("Sessoes antigas", clean_session_files(&codex_home_path)?);

    report.push_target(
        "Estado global",
        clean_global_state_files(&codex_home_path, &stale_thread_ids)?,
    );
    report.push_target(
        "Chats arquivados",
        remove_directory_contents(
            &codex_home_path,
            &codex_home_path.join(ARCHIVED_SESSIONS_DIRECTORY_NAME),
        )?,
    );
    report.push_target(
        "Anexos",
        remove_directory_contents(
            &codex_home_path,
            &codex_home_path.join(ATTACHMENTS_DIRECTORY_NAME),
        )?,
    );
    report.push_target(
        "Bancos legados",
        remove_legacy_root_databases(&codex_home_path)?,
    );

    for directory_name in TEMPORARY_DIRECTORY_NAMES {
        report.push_target(
            format!("Temporarios {directory_name}"),
            remove_directory_contents(&codex_home_path, &codex_home_path.join(directory_name))?,
        );
    }

    report.push_target(
        "Cache local",
        remove_directory_contents(
            &codex_home_path,
            &codex_home_path.join(CACHE_DIRECTORY_NAME),
        )?,
    );
    report.push_target(
        "Pastas vazias",
        remove_empty_session_directories(&codex_home_path)?,
    );

    report.push_warning("Cache de plugins preservado para manter ferramentas instaladas.");

    Ok(report.build())
}

fn locate_codex_home_path() -> Result<PathBuf, String> {
    if let Some(path) = env::var_os(CODEX_HOME_ENVIRONMENT_VARIABLE)
        .map(PathBuf::from)
        .filter(|path| path.is_dir())
    {
        return canonicalize_existing_directory(&path);
    }

    let user_profile_path = env::var_os(USER_PROFILE_ENVIRONMENT_VARIABLE)
        .map(PathBuf::from)
        .ok_or_else(|| "USERPROFILE is not defined.".to_owned())?;
    canonicalize_existing_directory(&user_profile_path.join(CODEX_HOME_DIRECTORY_NAME))
}

fn canonicalize_existing_directory(path: &Path) -> Result<PathBuf, String> {
    let canonical_path = path
        .canonicalize()
        .map_err(|error| format!("Failed to resolve {}: {error}", path.to_string_lossy()))?;

    if !canonical_path.is_dir() {
        return Err(format!(
            "Expected a directory at {}.",
            canonical_path.to_string_lossy()
        ));
    }

    Ok(canonical_path)
}

fn collect_stale_thread_ids(
    codex_home_path: &Path,
    active_database_path: &Path,
) -> Result<HashSet<String>, String> {
    let mut thread_ids = HashSet::new();

    thread_ids.extend(collect_session_index_thread_ids(codex_home_path)?);
    thread_ids.extend(collect_database_thread_ids(active_database_path)?);
    thread_ids.extend(collect_session_file_thread_ids(
        &codex_home_path.join(SESSIONS_DIRECTORY_NAME),
    )?);
    thread_ids.extend(collect_session_file_thread_ids(
        &codex_home_path.join(ARCHIVED_SESSIONS_DIRECTORY_NAME),
    )?);

    Ok(thread_ids)
}

fn collect_session_index_thread_ids(codex_home_path: &Path) -> Result<HashSet<String>, String> {
    let session_index_path = codex_home_path.join(SESSION_INDEX_FILE_NAME);
    if !session_index_path.is_file() {
        return Ok(HashSet::new());
    }

    let file = File::open(&session_index_path).map_err(|error| {
        format!(
            "Failed to open {}: {error}",
            session_index_path.to_string_lossy()
        )
    })?;
    let mut thread_ids = HashSet::new();

    for line in BufReader::new(file).lines() {
        let line = line.map_err(|error| {
            format!(
                "Failed to read {}: {error}",
                session_index_path.to_string_lossy()
            )
        })?;
        if line.trim().is_empty() {
            continue;
        }

        let value = serde_json::from_str::<Value>(&line).map_err(|error| {
            format!(
                "Invalid JSON in {}: {error}",
                session_index_path.to_string_lossy()
            )
        })?;
        if let Some(thread_id) = value.get("id").and_then(Value::as_str) {
            thread_ids.insert(thread_id.to_owned());
        }
    }

    Ok(thread_ids)
}

fn collect_database_thread_ids(active_database_path: &Path) -> Result<HashSet<String>, String> {
    if !active_database_path.is_file()
        || active_database_path
            .metadata()
            .map_or(true, |m| m.len() == 0)
    {
        return Ok(HashSet::new());
    }

    let connection = open_database(active_database_path)?;
    let mut statement = connection
        .prepare("SELECT id FROM threads")
        .map_err(|error| {
            format!(
                "Failed to prepare thread query for {}: {error}",
                active_database_path.to_string_lossy()
            )
        })?;
    let rows = statement
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|error| {
            format!(
                "Failed to query threads from {}: {error}",
                active_database_path.to_string_lossy()
            )
        })?;

    let mut thread_ids = HashSet::new();
    for row in rows {
        thread_ids.insert(row.map_err(|error| {
            format!(
                "Failed to read thread id from {}: {error}",
                active_database_path.to_string_lossy()
            )
        })?);
    }

    Ok(thread_ids)
}

fn collect_session_file_thread_ids(directory_path: &Path) -> Result<HashSet<String>, String> {
    if !directory_path.is_dir() {
        return Ok(HashSet::new());
    }

    let mut thread_ids = HashSet::new();
    collect_session_file_thread_ids_recursive(directory_path, &mut thread_ids)?;
    Ok(thread_ids)
}

fn collect_session_file_thread_ids_recursive(
    directory_path: &Path,
    thread_ids: &mut HashSet<String>,
) -> Result<(), String> {
    for entry in fs::read_dir(directory_path).map_err(|error| {
        format!(
            "Failed to read {}: {error}",
            directory_path.to_string_lossy()
        )
    })? {
        let entry = entry.map_err(|error| {
            format!(
                "Failed to inspect an entry in {}: {error}",
                directory_path.to_string_lossy()
            )
        })?;
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path)
            .map_err(|error| format!("Failed to inspect {}: {error}", path.to_string_lossy()))?;

        if metadata.is_dir() && !metadata.file_type().is_symlink() {
            collect_session_file_thread_ids_recursive(&path, thread_ids)?;
            continue;
        }

        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };

        for thread_id in extract_thread_ids(file_name) {
            thread_ids.insert(thread_id);
        }
    }

    Ok(())
}

fn collect_global_state_thread_ids(codex_home_path: &Path) -> Result<HashSet<String>, String> {
    let mut thread_ids = HashSet::new();

    for file_name in GLOBAL_STATE_FILE_NAMES {
        let file_path = codex_home_path.join(file_name);
        if !file_path.is_file() {
            continue;
        }

        let value = read_json_file(&file_path)?;
        collect_thread_ids_from_json(&value, &mut thread_ids);
    }

    Ok(thread_ids)
}

fn clean_active_database(active_database_path: &Path) -> Result<CleanupStats, String> {
    if !active_database_path.is_file()
        || active_database_path
            .metadata()
            .map_or(true, |m| m.len() == 0)
    {
        return Ok(CleanupStats::default());
    }

    let before_bytes = database_family_size(active_database_path);
    let mut connection = open_database(active_database_path)?;
    let transaction = connection.transaction().map_err(|error| {
        format!(
            "Failed to start database cleanup transaction for {}: {error}",
            active_database_path.to_string_lossy()
        )
    })?;

    transaction
        .execute_batch("PRAGMA foreign_keys = ON;")
        .map_err(|error| {
            format!(
                "Failed to enable foreign keys for {}: {error}",
                active_database_path.to_string_lossy()
            )
        })?;
    transaction
        .execute(
            "UPDATE agent_job_items SET assigned_thread_id = NULL WHERE assigned_thread_id IS NOT NULL",
            [],
        )
        .map_err(|error| format!("Failed to detach agent job items: {error}"))?;
    transaction
        .execute("DELETE FROM thread_dynamic_tools", [])
        .map_err(|error| format!("Failed to delete dynamic tool records: {error}"))?;
    transaction
        .execute("DELETE FROM thread_spawn_edges", [])
        .map_err(|error| format!("Failed to delete thread spawn edges: {error}"))?;
    let removed_thread_count = transaction
        .execute("DELETE FROM threads", [])
        .map_err(|error| format!("Failed to delete threads: {error}"))?
        as u64;

    transaction.commit().map_err(|error| {
        format!(
            "Failed to commit database cleanup for {}: {error}",
            active_database_path.to_string_lossy()
        )
    })?;
    connection
        .execute_batch("VACUUM; PRAGMA wal_checkpoint(TRUNCATE);")
        .map_err(|error| {
            format!(
                "Failed to compact {}: {error}",
                active_database_path.to_string_lossy()
            )
        })?;

    let after_bytes = database_family_size(active_database_path);

    Ok(CleanupStats {
        removed_thread_count,
        freed_bytes: before_bytes.saturating_sub(after_bytes),
        ..CleanupStats::default()
    })
}

fn clean_session_index(codex_home_path: &Path) -> Result<CleanupStats, String> {
    let session_index_path = codex_home_path.join(SESSION_INDEX_FILE_NAME);
    if !session_index_path.is_file() {
        return Ok(CleanupStats::default());
    }

    let before_bytes = file_size(&session_index_path);
    let file = File::open(&session_index_path).map_err(|error| {
        format!(
            "Failed to open {}: {error}",
            session_index_path.to_string_lossy()
        )
    })?;
    let mut removed_thread_count = 0u64;

    for line in BufReader::new(file).lines() {
        let line = line.map_err(|error| {
            format!(
                "Failed to read {}: {error}",
                session_index_path.to_string_lossy()
            )
        })?;
        if line.trim().is_empty() {
            continue;
        }

        let value = serde_json::from_str::<Value>(&line).map_err(|error| {
            format!(
                "Invalid JSON in {}: {error}",
                session_index_path.to_string_lossy()
            )
        })?;
        value.get("id").and_then(Value::as_str).ok_or_else(|| {
            format!(
                "Session index entry is missing an id in {}.",
                session_index_path.to_string_lossy()
            )
        })?;

        removed_thread_count += 1;
    }

    File::create(&session_index_path).map_err(|error| {
        format!(
            "Failed to rewrite {}: {error}",
            session_index_path.to_string_lossy()
        )
    })?;

    let after_bytes = file_size(&session_index_path);
    Ok(CleanupStats {
        removed_thread_count,
        freed_bytes: before_bytes.saturating_sub(after_bytes),
        ..CleanupStats::default()
    })
}

fn clean_session_files(codex_home_path: &Path) -> Result<CleanupStats, String> {
    let sessions_path = codex_home_path.join(SESSIONS_DIRECTORY_NAME);
    if !sessions_path.is_dir() {
        return Ok(CleanupStats::default());
    }

    let root = canonicalize_existing_directory(codex_home_path)?;
    let mut stats = CleanupStats::default();
    remove_old_session_files_recursive(&root, &sessions_path, &mut stats)?;
    Ok(stats)
}

fn remove_old_session_files_recursive(
    root: &Path,
    directory_path: &Path,
    stats: &mut CleanupStats,
) -> Result<(), String> {
    for entry in fs::read_dir(directory_path).map_err(|error| {
        format!(
            "Failed to read {}: {error}",
            directory_path.to_string_lossy()
        )
    })? {
        let entry = entry.map_err(|error| {
            format!(
                "Failed to inspect an entry in {}: {error}",
                directory_path.to_string_lossy()
            )
        })?;
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path)
            .map_err(|error| format!("Failed to inspect {}: {error}", path.to_string_lossy()))?;

        if metadata.is_dir() && !metadata.file_type().is_symlink() {
            remove_old_session_files_recursive(root, &path, stats)?;
            continue;
        }

        if !is_jsonl_file(&path) {
            continue;
        }

        delete_path(root, &path, stats)?;
    }

    Ok(())
}

fn clean_global_state_files(
    codex_home_path: &Path,
    stale_thread_ids: &HashSet<String>,
) -> Result<CleanupStats, String> {
    if stale_thread_ids.is_empty() {
        return Ok(CleanupStats::default());
    }

    let mut stats = CleanupStats::default();
    for file_name in GLOBAL_STATE_FILE_NAMES {
        let file_path = codex_home_path.join(file_name);
        if !file_path.is_file() {
            continue;
        }

        let before_bytes = file_size(&file_path);
        let value = read_json_file(&file_path)?;
        let mut removed_references = 0u64;
        let cleaned_value =
            clean_json_thread_references(value, stale_thread_ids, &mut removed_references)
                .unwrap_or(Value::Object(serde_json::Map::new()));
        let serialized = serde_json::to_vec(&cleaned_value).map_err(|error| {
            format!(
                "Failed to serialize {}: {error}",
                file_path.to_string_lossy()
            )
        })?;
        fs::write(&file_path, serialized)
            .map_err(|error| format!("Failed to write {}: {error}", file_path.to_string_lossy()))?;
        let after_bytes = file_size(&file_path);

        stats.removed_global_state_reference_count += removed_references;
        stats.freed_bytes += before_bytes.saturating_sub(after_bytes);
    }

    Ok(stats)
}

fn remove_legacy_root_databases(codex_home_path: &Path) -> Result<CleanupStats, String> {
    let root = canonicalize_existing_directory(codex_home_path)?;
    let mut stats = CleanupStats::default();

    for entry in fs::read_dir(&root)
        .map_err(|error| format!("Failed to read {}: {error}", root.to_string_lossy()))?
    {
        let entry = entry.map_err(|error| {
            format!(
                "Failed to inspect an entry in {}: {error}",
                root.to_string_lossy()
            )
        })?;
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path)
            .map_err(|error| format!("Failed to inspect {}: {error}", path.to_string_lossy()))?;

        if !metadata.is_file() || !is_sqlite_family_file(&path) {
            continue;
        }

        delete_path(&root, &path, &mut stats)?;
    }

    Ok(stats)
}

fn remove_directory_contents(root: &Path, directory_path: &Path) -> Result<CleanupStats, String> {
    if !directory_path.is_dir() {
        return Ok(CleanupStats::default());
    }

    let root = canonicalize_existing_directory(root)?;
    let directory_path = canonicalize_existing_directory(directory_path)?;
    ensure_path_inside(&root, &directory_path)?;

    let mut stats = CleanupStats::default();
    for entry in fs::read_dir(&directory_path).map_err(|error| {
        format!(
            "Failed to read {}: {error}",
            directory_path.to_string_lossy()
        )
    })? {
        let entry = entry.map_err(|error| {
            format!(
                "Failed to inspect an entry in {}: {error}",
                directory_path.to_string_lossy()
            )
        })?;
        delete_path(&root, &entry.path(), &mut stats)?;
    }

    Ok(stats)
}

fn remove_empty_session_directories(codex_home_path: &Path) -> Result<CleanupStats, String> {
    let root = canonicalize_existing_directory(codex_home_path)?;
    let mut stats = CleanupStats::default();

    for directory_name in [SESSIONS_DIRECTORY_NAME, ARCHIVED_SESSIONS_DIRECTORY_NAME] {
        let directory_path = codex_home_path.join(directory_name);
        if directory_path.is_dir() {
            remove_empty_directories_recursive(
                &root,
                &directory_path,
                &directory_path,
                &mut stats,
            )?;
        }
    }

    Ok(stats)
}

fn remove_empty_directories_recursive(
    root: &Path,
    protected_directory_path: &Path,
    directory_path: &Path,
    stats: &mut CleanupStats,
) -> Result<(), String> {
    let entries = fs::read_dir(directory_path).map_err(|error| {
        format!(
            "Failed to read {}: {error}",
            directory_path.to_string_lossy()
        )
    })?;

    for entry in entries {
        let entry = entry.map_err(|error| {
            format!(
                "Failed to inspect an entry in {}: {error}",
                directory_path.to_string_lossy()
            )
        })?;
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path)
            .map_err(|error| format!("Failed to inspect {}: {error}", path.to_string_lossy()))?;

        if metadata.is_dir() && !metadata.file_type().is_symlink() {
            remove_empty_directories_recursive(root, protected_directory_path, &path, stats)?;
        }
    }

    if directory_path == protected_directory_path {
        return Ok(());
    }

    let mut entries = fs::read_dir(directory_path).map_err(|error| {
        format!(
            "Failed to read {}: {error}",
            directory_path.to_string_lossy()
        )
    })?;
    if entries.next().is_none() {
        ensure_path_inside(root, directory_path)?;
        fs::remove_dir(directory_path).map_err(|error| {
            format!(
                "Failed to remove empty directory {}: {error}",
                directory_path.to_string_lossy()
            )
        })?;
        stats.removed_directory_count += 1;
    }

    Ok(())
}

fn open_database(path: &Path) -> Result<Connection, String> {
    let connection = Connection::open(path)
        .map_err(|error| format!("Failed to open {}: {error}", path.to_string_lossy()))?;
    connection
        .busy_timeout(DATABASE_BUSY_TIMEOUT)
        .map_err(|error| format!("Failed to configure database timeout: {error}"))?;
    Ok(connection)
}

fn read_json_file(path: &Path) -> Result<Value, String> {
    let data = fs::read(path)
        .map_err(|error| format!("Failed to read {}: {error}", path.to_string_lossy()))?;
    serde_json::from_slice(&data)
        .map_err(|error| format!("Invalid JSON in {}: {error}", path.to_string_lossy()))
}

fn clean_json_thread_references(
    value: Value,
    stale_thread_ids: &HashSet<String>,
    removed_references: &mut u64,
) -> Option<Value> {
    match value {
        Value::String(text) if stale_thread_ids.contains(&text) => {
            *removed_references += 1;
            None
        }
        Value::Array(items) => Some(Value::Array(
            items
                .into_iter()
                .filter_map(|item| {
                    if json_value_has_stale_thread_marker(&item, stale_thread_ids) {
                        *removed_references += 1;
                        None
                    } else {
                        clean_json_thread_references(item, stale_thread_ids, removed_references)
                    }
                })
                .collect(),
        )),
        Value::Object(object) => {
            let mut cleaned = serde_json::Map::new();

            for (key, item) in object {
                if stale_thread_ids.contains(&key)
                    || json_value_has_stale_thread_marker(&item, stale_thread_ids)
                {
                    *removed_references += 1;
                    continue;
                }

                if let Some(cleaned_item) =
                    clean_json_thread_references(item, stale_thread_ids, removed_references)
                {
                    cleaned.insert(key, cleaned_item);
                }
            }

            Some(Value::Object(cleaned))
        }
        other => Some(other),
    }
}

fn json_value_has_stale_thread_marker(value: &Value, stale_thread_ids: &HashSet<String>) -> bool {
    let Some(object) = value.as_object() else {
        return false;
    };

    ["id", "threadId", "targetThreadId", "conversationId"]
        .iter()
        .filter_map(|key| object.get(*key))
        .filter_map(Value::as_str)
        .any(|thread_id| stale_thread_ids.contains(thread_id))
}

fn collect_thread_ids_from_json(value: &Value, thread_ids: &mut HashSet<String>) {
    match value {
        Value::String(text) => {
            if is_codex_thread_id(text) {
                thread_ids.insert(text.to_owned());
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_thread_ids_from_json(item, thread_ids);
            }
        }
        Value::Object(object) => {
            for (key, item) in object {
                if is_codex_thread_id(key) {
                    thread_ids.insert(key.to_owned());
                }
                collect_thread_ids_from_json(item, thread_ids);
            }
        }
        _ => {}
    }
}

fn extract_thread_ids(text: &str) -> Vec<String> {
    text.split(|character: char| !(character.is_ascii_hexdigit() || character == '-'))
        .filter(|candidate| is_codex_thread_id(candidate))
        .map(str::to_owned)
        .collect()
}

fn is_codex_thread_id(value: &str) -> bool {
    value.len() == THREAD_ID_LENGTH
        && value.starts_with(CODEX_THREAD_ID_PREFIX)
        && value.char_indices().all(|(index, character)| match index {
            8 | 13 | 18 | 23 => character == '-',
            _ => character.is_ascii_hexdigit(),
        })
}

fn delete_path(root: &Path, path: &Path, stats: &mut CleanupStats) -> Result<(), String> {
    ensure_delete_target_inside(root, path)?;

    let metadata = fs::symlink_metadata(path)
        .map_err(|error| format!("Failed to inspect {}: {error}", path.to_string_lossy()))?;
    if metadata.file_type().is_symlink() {
        remove_symlink(path, stats)
    } else if metadata.is_dir() {
        for entry in fs::read_dir(path)
            .map_err(|error| format!("Failed to read {}: {error}", path.to_string_lossy()))?
        {
            let entry = entry.map_err(|error| {
                format!(
                    "Failed to inspect an entry in {}: {error}",
                    path.to_string_lossy()
                )
            })?;
            delete_path(root, &entry.path(), stats)?;
        }
        fs::remove_dir(path).map_err(|error| {
            format!(
                "Failed to remove directory {}: {error}",
                path.to_string_lossy()
            )
        })?;
        stats.removed_directory_count += 1;
        Ok(())
    } else {
        stats.freed_bytes += metadata.len();
        fs::remove_file(path).map_err(|error| {
            format!("Failed to remove file {}: {error}", path.to_string_lossy())
        })?;
        stats.removed_file_count += 1;
        Ok(())
    }
}

fn remove_symlink(path: &Path, stats: &mut CleanupStats) -> Result<(), String> {
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
                "Failed to remove symlink {} as file ({file_error}) or directory ({directory_error}).",
                path.to_string_lossy()
            )),
        },
    }
}

fn ensure_delete_target_inside(root: &Path, path: &Path) -> Result<(), String> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| format!("Failed to inspect {}: {error}", path.to_string_lossy()))?;

    if metadata.file_type().is_symlink() {
        let parent = path
            .parent()
            .ok_or_else(|| format!("Delete target {} has no parent.", path.to_string_lossy()))?;
        ensure_path_inside(
            root,
            &parent.canonicalize().map_err(|error| {
                format!("Failed to resolve {}: {error}", parent.to_string_lossy())
            })?,
        )
    } else {
        ensure_path_inside(
            root,
            &path.canonicalize().map_err(|error| {
                format!("Failed to resolve {}: {error}", path.to_string_lossy())
            })?,
        )
    }
}

fn ensure_path_inside(root: &Path, path: &Path) -> Result<(), String> {
    if path == root || path.starts_with(root) {
        return Ok(());
    }

    Err(format!(
        "Refusing to operate outside {}: {}",
        root.to_string_lossy(),
        path.to_string_lossy()
    ))
}

fn database_family_size(database_path: &Path) -> u64 {
    [
        database_path.to_path_buf(),
        PathBuf::from(format!("{}-wal", database_path.to_string_lossy())),
        PathBuf::from(format!("{}-shm", database_path.to_string_lossy())),
    ]
    .iter()
    .map(|path| file_size(path))
    .sum()
}

fn file_size(path: &Path) -> u64 {
    path.metadata().map_or(0, |metadata| metadata.len())
}

fn is_jsonl_file(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("jsonl"))
}

fn is_sqlite_family_file(path: &Path) -> bool {
    let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };

    file_name.ends_with(".sqlite")
        || file_name.ends_with(".sqlite-wal")
        || file_name.ends_with(".sqlite-shm")
}

fn build_relative_path(base_path: &Path, relative_parts: &[&str]) -> PathBuf {
    relative_parts
        .iter()
        .fold(base_path.to_path_buf(), |mut current_path, part| {
            current_path.push(part);
            current_path
        })
}
