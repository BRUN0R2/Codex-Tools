use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use serde::Serialize;

use crate::platform::windows_codex::{
    is_codex_app_server_executable_path, is_codex_desktop_executable_path,
};
use crate::platform::windows_process::{
    CodexProcessInspection, PriorityUpdate, WindowsProcessPriorityState,
    apply_high_priority_to_running_codex, inspect_running_codex_processes,
};

const CODEX_SESSION_STARTUP_BUDGET: Duration = Duration::from_secs(30);
const PRIORITY_OBSERVATION_INTERVAL: Duration = Duration::from_millis(200);
const REQUIRED_READY_CONFIRMATIONS: usize = 3;
const STABILIZATION_THREAD_NAME: &str = "codex-priority-stabilization";

#[derive(Clone)]
pub struct PriorityStabilizationStore {
    status: Arc<Mutex<PriorityStabilizationSnapshot>>,
}

impl Default for PriorityStabilizationStore {
    fn default() -> Self {
        Self {
            status: Arc::new(Mutex::new(PriorityStabilizationSnapshot::idle())),
        }
    }
}

impl PriorityStabilizationStore {
    pub fn snapshot(&self) -> Result<PriorityStabilizationSnapshot, String> {
        self.status
            .lock()
            .map(|status| status.clone())
            .map_err(|_| "Priority stabilization status is unavailable.".to_owned())
    }

    fn replace(&self, status: PriorityStabilizationSnapshot) -> Result<(), String> {
        let mut current_status = self
            .status
            .lock()
            .map_err(|_| "Priority stabilization status is unavailable.".to_owned())?;

        *current_status = status;
        Ok(())
    }

    fn mark_running(&self) -> Result<(), String> {
        self.replace(PriorityStabilizationSnapshot::running())
    }

    fn mark_succeeded(&self, result: PriorityStabilizationResult) -> Result<(), String> {
        self.replace(PriorityStabilizationSnapshot::succeeded(result))
    }

    fn mark_failed(&self, failure: PriorityStabilizationFailure) -> Result<(), String> {
        self.replace(PriorityStabilizationSnapshot::failed(failure))
    }
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PriorityStabilizationSnapshot {
    pub state: PriorityStabilizationState,
    pub message: Option<String>,
    pub updated_process_ids: Vec<u32>,
    pub attempts: usize,
}

impl PriorityStabilizationSnapshot {
    fn idle() -> Self {
        Self {
            state: PriorityStabilizationState::Idle,
            message: None,
            updated_process_ids: Vec::new(),
            attempts: 0,
        }
    }

    fn running() -> Self {
        Self {
            state: PriorityStabilizationState::Running,
            message: None,
            updated_process_ids: Vec::new(),
            attempts: 0,
        }
    }

    fn succeeded(result: PriorityStabilizationResult) -> Self {
        Self {
            state: PriorityStabilizationState::Succeeded,
            message: None,
            updated_process_ids: result.updated_process_ids,
            attempts: result.attempts,
        }
    }

    fn failed(failure: PriorityStabilizationFailure) -> Self {
        Self {
            state: PriorityStabilizationState::Failed,
            message: Some(failure.message),
            updated_process_ids: Vec::new(),
            attempts: failure.attempts,
        }
    }
}

#[derive(Clone, Copy, Serialize)]
pub enum PriorityStabilizationState {
    Idle,
    Running,
    Succeeded,
    Failed,
}

pub fn start_high_priority_stabilization(store: PriorityStabilizationStore) -> Result<(), String> {
    store.mark_running()?;
    let worker_store = store.clone();

    match thread::Builder::new()
        .name(STABILIZATION_THREAD_NAME.to_owned())
        .spawn(move || {
            let status_update = match apply_high_priority_until_session_is_ready() {
                Ok(result) => worker_store.mark_succeeded(result),
                Err(failure) => worker_store.mark_failed(failure),
            };

            if let Err(error) = status_update {
                eprintln!("Failed to publish Codex priority stabilization status: {error}");
            }
        }) {
        Ok(_) => Ok(()),
        Err(error) => {
            let message = format!("Failed to start Codex priority stabilization: {error}");
            store.mark_failed(PriorityStabilizationFailure::new(0, message.clone()))?;
            Err(message)
        }
    }
}

struct PriorityStabilizationResult {
    updated_process_ids: Vec<u32>,
    attempts: usize,
}

struct PriorityStabilizationFailure {
    message: String,
    attempts: usize,
}

impl PriorityStabilizationFailure {
    fn new(attempts: usize, message: impl Into<String>) -> Self {
        Self {
            attempts,
            message: message.into(),
        }
    }
}

fn apply_high_priority_until_session_is_ready()
-> Result<PriorityStabilizationResult, PriorityStabilizationFailure> {
    let deadline = Instant::now() + CODEX_SESSION_STARTUP_BUDGET;
    let mut updated_process_ids = BTreeSet::new();
    let mut priority_failures = BTreeMap::new();
    let mut ready_confirmations = 0usize;
    let mut desktop_was_observed = false;
    let mut attempt = 0usize;

    loop {
        attempt += 1;
        let priority_updates = apply_high_priority_to_running_codex().map_err(|error| {
            PriorityStabilizationFailure::new(
                attempt,
                format!("Failed to apply Codex high priority: {error}"),
            )
        })?;
        record_priority_updates(
            &mut updated_process_ids,
            &mut priority_failures,
            priority_updates,
        );

        let processes = inspect_running_codex_processes().map_err(|error| {
            PriorityStabilizationFailure::new(
                attempt,
                format!("Failed to inspect Codex priority after update: {error}"),
            )
        })?;
        let session = inspect_codex_session(&processes);
        if session.desktop_process_count > 0 {
            desktop_was_observed = true;
        }

        if session_is_ready(&session) {
            ready_confirmations += 1;
            if ready_confirmations >= REQUIRED_READY_CONFIRMATIONS {
                return Ok(PriorityStabilizationResult {
                    updated_process_ids: updated_process_ids.into_iter().collect(),
                    attempts: attempt,
                });
            }
        } else {
            ready_confirmations = 0;
            if desktop_was_observed && session.desktop_process_count == 0 {
                return Err(PriorityStabilizationFailure::new(
                    attempt,
                    "Codex desktop process exited before high priority was established.",
                ));
            }
            if Instant::now() >= deadline {
                return Err(PriorityStabilizationFailure::new(
                    attempt,
                    describe_startup_failure(&session, &priority_failures),
                ));
            }
        }

        thread::sleep(PRIORITY_OBSERVATION_INTERVAL);
    }
}

fn record_priority_updates(
    updated_process_ids: &mut BTreeSet<u32>,
    priority_failures: &mut BTreeMap<u32, String>,
    priority_updates: Vec<PriorityUpdate>,
) {
    for update in priority_updates {
        match update.error {
            None => {
                updated_process_ids.insert(update.process_id);
                priority_failures.remove(&update.process_id);
            }
            Some(message) => {
                priority_failures.insert(update.process_id, message);
            }
        }
    }
}

struct CodexSessionSnapshot {
    desktop_process_count: usize,
    app_server_process_count: usize,
    processes_outside_high_priority: Vec<ProcessPriorityGap>,
}

struct ProcessPriorityGap {
    process_id: u32,
    process_name: String,
    priority: WindowsProcessPriorityState,
}

fn inspect_codex_session(processes: &[CodexProcessInspection]) -> CodexSessionSnapshot {
    let mut snapshot = CodexSessionSnapshot {
        desktop_process_count: 0,
        app_server_process_count: 0,
        processes_outside_high_priority: Vec::new(),
    };

    for process in processes {
        let executable_path = process.executable_path.as_deref().unwrap_or("");
        if is_codex_desktop_executable_path(executable_path) {
            snapshot.desktop_process_count += 1;
        }
        if is_codex_app_server_executable_path(executable_path) {
            snapshot.app_server_process_count += 1;
        }
        if process.priority != WindowsProcessPriorityState::High {
            snapshot
                .processes_outside_high_priority
                .push(ProcessPriorityGap {
                    process_id: process.process_id,
                    process_name: process.process_name.clone(),
                    priority: process.priority,
                });
        }
    }

    snapshot
        .processes_outside_high_priority
        .sort_by_key(|process| process.process_id);
    snapshot
}

fn session_is_ready(snapshot: &CodexSessionSnapshot) -> bool {
    snapshot.desktop_process_count > 0
        && snapshot.app_server_process_count > 0
        && snapshot.processes_outside_high_priority.is_empty()
}

fn describe_startup_failure(
    snapshot: &CodexSessionSnapshot,
    priority_failures: &BTreeMap<u32, String>,
) -> String {
    let mut reasons = Vec::new();

    if snapshot.desktop_process_count == 0 {
        reasons.push("the Codex desktop process is not running".to_owned());
    }
    if snapshot.app_server_process_count == 0 {
        reasons.push("the Codex app-server is not running".to_owned());
    }
    for process in &snapshot.processes_outside_high_priority {
        reasons.push(describe_process_gap(process, priority_failures));
    }
    if reasons.is_empty() {
        reasons.push("the Codex session was not high priority".to_owned());
    }

    format!(
        "Codex did not reach high priority within the startup budget: {}.",
        reasons.join("; ")
    )
}

fn describe_process_gap(
    process: &ProcessPriorityGap,
    priority_failures: &BTreeMap<u32, String>,
) -> String {
    match priority_failures.get(&process.process_id) {
        Some(message) => format!(
            "PID {} ({}) is {}: {}",
            process.process_id,
            process.process_name,
            process.priority,
            message.trim_end_matches('.')
        ),
        None => format!(
            "PID {} ({}) is {}",
            process.process_id, process.process_name, process.priority
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CodexSessionSnapshot, describe_startup_failure, inspect_codex_session, session_is_ready,
    };
    use crate::platform::windows_process::{
        CodexProcessInspection, WindowsProcessElevationState, WindowsProcessPriorityState,
    };
    use std::collections::BTreeMap;

    const DESKTOP_PATH: &str = r"C:\Program Files\WindowsApps\OpenAI.Codex_26.917.9434.0_x64__2p2nqsd0c76g0\app\ChatGPT.exe";
    const APP_SERVER_PATH: &str = r"C:\Program Files\WindowsApps\OpenAI.Codex_26.917.9434.0_x64__2p2nqsd0c76g0\app\resources\codex.exe";

    #[test]
    fn ready_session_requires_desktop_app_server_and_high_priority() {
        let ready = inspect_codex_session(&[
            process(
                10,
                "ChatGPT.exe",
                Some(DESKTOP_PATH),
                WindowsProcessPriorityState::High,
                WindowsProcessElevationState::NotElevated,
            ),
            process(
                11,
                "codex.exe",
                Some(APP_SERVER_PATH),
                WindowsProcessPriorityState::High,
                WindowsProcessElevationState::NotElevated,
            ),
        ]);
        let renderer_still_normal = inspect_codex_session(&[
            process(
                10,
                "ChatGPT.exe",
                Some(DESKTOP_PATH),
                WindowsProcessPriorityState::High,
                WindowsProcessElevationState::Elevated,
            ),
            process(
                12,
                "ChatGPT.exe",
                Some(DESKTOP_PATH),
                WindowsProcessPriorityState::Normal,
                WindowsProcessElevationState::NotElevated,
            ),
            process(
                11,
                "codex.exe",
                Some(APP_SERVER_PATH),
                WindowsProcessPriorityState::High,
                WindowsProcessElevationState::Elevated,
            ),
        ]);
        let app_server_not_started = inspect_codex_session(&[process(
            10,
            "ChatGPT.exe",
            Some(DESKTOP_PATH),
            WindowsProcessPriorityState::High,
            WindowsProcessElevationState::NotElevated,
        )]);
        assert!(session_is_ready(&ready));
        assert!(!session_is_ready(&renderer_still_normal));
        assert!(!session_is_ready(&app_server_not_started));
    }

    #[test]
    fn startup_failure_names_the_missing_condition_and_the_process() {
        let snapshot = inspect_codex_session(&[process(
            41,
            "ChatGPT.exe",
            Some(DESKTOP_PATH),
            WindowsProcessPriorityState::Normal,
            WindowsProcessElevationState::Elevated,
        )]);
        let mut priority_failures = BTreeMap::new();
        priority_failures.insert(41, "Access is denied.".to_owned());

        let message = describe_startup_failure(&snapshot, &priority_failures);

        assert!(message.contains("the Codex app-server is not running"));
        assert!(message.contains("PID 41 (ChatGPT.exe) is Normal: Access is denied."));
        assert!(!message.contains("did not stabilize within the expected time"));
    }

    #[test]
    fn empty_session_reports_that_the_desktop_never_started() {
        let message = describe_startup_failure(
            &CodexSessionSnapshot {
                desktop_process_count: 0,
                app_server_process_count: 0,
                processes_outside_high_priority: Vec::new(),
            },
            &BTreeMap::new(),
        );

        assert!(message.contains("the Codex desktop process is not running"));
        assert!(message.contains("the Codex app-server is not running"));
    }

    fn process(
        process_id: u32,
        process_name: &str,
        executable_path: Option<&str>,
        priority: WindowsProcessPriorityState,
        elevation: WindowsProcessElevationState,
    ) -> CodexProcessInspection {
        CodexProcessInspection {
            process_id,
            parent_process_id: 0,
            process_name: process_name.to_owned(),
            executable_path: executable_path.map(str::to_owned),
            priority,
            elevation,
        }
    }
}
