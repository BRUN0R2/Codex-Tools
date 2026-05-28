use std::collections::BTreeSet;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use serde::Serialize;

use crate::platform::windows_process::{
    CodexProcessInspection, WindowsProcessPriorityState, apply_high_priority_to_running_codex,
    inspect_running_codex_processes,
};

const PRIORITY_APPLICATION_MAX_ATTEMPTS: usize = 40;
const PRIORITY_APPLICATION_MIN_ATTEMPTS: usize = 24;
const PRIORITY_APPLICATION_RETRY_DELAY: Duration = Duration::from_millis(500);
const REQUIRED_STABLE_HIGH_PRIORITY_ATTEMPTS: usize = 3;
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
            let status_update = match apply_high_priority_with_retry() {
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

fn apply_high_priority_with_retry()
-> Result<PriorityStabilizationResult, PriorityStabilizationFailure> {
    let mut updated_process_ids = BTreeSet::new();
    let mut stable_high_priority_attempts = 0usize;

    for attempt_index in 0..PRIORITY_APPLICATION_MAX_ATTEMPTS {
        let current_attempt = attempt_index + 1;
        let priority_application = apply_high_priority_to_running_codex().map_err(|error| {
            PriorityStabilizationFailure::new(
                current_attempt,
                format!("Failed to apply Codex high priority: {error}"),
            )
        })?;

        updated_process_ids.extend(priority_application.updated_process_ids);

        let processes = inspect_running_codex_processes().map_err(|error| {
            PriorityStabilizationFailure::new(
                current_attempt,
                format!("Failed to inspect Codex priority after update: {error}"),
            )
        })?;

        if codex_processes_are_high_priority(&processes) {
            stable_high_priority_attempts += 1;
        } else {
            stable_high_priority_attempts = 0;
        }

        let minimum_attempts_completed = current_attempt >= PRIORITY_APPLICATION_MIN_ATTEMPTS;
        let priority_is_stable =
            stable_high_priority_attempts >= REQUIRED_STABLE_HIGH_PRIORITY_ATTEMPTS;

        if minimum_attempts_completed && priority_is_stable {
            return Ok(PriorityStabilizationResult {
                updated_process_ids: updated_process_ids.into_iter().collect(),
                attempts: current_attempt,
            });
        }

        if current_attempt < PRIORITY_APPLICATION_MAX_ATTEMPTS {
            thread::sleep(PRIORITY_APPLICATION_RETRY_DELAY);
        }
    }

    Err(PriorityStabilizationFailure::new(
        PRIORITY_APPLICATION_MAX_ATTEMPTS,
        "Codex high priority did not stabilize within the expected time.",
    ))
}

fn codex_processes_are_high_priority(processes: &[CodexProcessInspection]) -> bool {
    !processes.is_empty()
        && processes
            .iter()
            .all(|process| matches!(process.priority, WindowsProcessPriorityState::High))
}
