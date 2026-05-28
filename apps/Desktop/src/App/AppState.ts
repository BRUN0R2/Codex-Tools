import {
  IDLE_ACTION_STATUS,
  createFailedActionStatus,
  createRunningActionStatus,
  createSucceededActionStatus,
  type ActionStatus,
} from "../Domain/ActionStatus";
import {
  CHECKING_CODEX_STATUS,
  UNCHECKED_CODEX_STATUS,
  createFailedCodexStatus,
  type CodexStatus,
} from "../Domain/CodexInstallation";
import type { ProcessPriority, SelectedProcessPriority } from "../Domain/ProcessPriority";

export type AppState = Readonly<{
  actionStatus: ActionStatus;
  codexStatus: CodexStatus;
  selectedPriority: SelectedProcessPriority;
}>;

export const INITIAL_APP_STATE: AppState = {
  actionStatus: IDLE_ACTION_STATUS,
  codexStatus: UNCHECKED_CODEX_STATUS,
  selectedPriority: null,
};

export function selectProcessPriority(state: AppState, priority: ProcessPriority): AppState {
  return {
    ...state,
    selectedPriority: priority,
  };
}

export function setCheckingCodexStatus(state: AppState): AppState {
  return {
    ...state,
    codexStatus: CHECKING_CODEX_STATUS,
  };
}

export function setCodexStatus(state: AppState, codexStatus: CodexStatus): AppState {
  return {
    ...state,
    codexStatus,
  };
}

export function setFailedCodexStatus(state: AppState, message: string): AppState {
  return setCodexStatus(state, createFailedCodexStatus(message));
}

export function setRunningActionStatus(state: AppState, label: string): AppState {
  return {
    ...state,
    actionStatus: createRunningActionStatus(label),
  };
}

export function setSucceededActionStatus(state: AppState, message: string): AppState {
  return {
    ...state,
    actionStatus: createSucceededActionStatus(message),
  };
}

export function setFailedActionStatus(state: AppState, message: string): AppState {
  return {
    ...state,
    actionStatus: createFailedActionStatus(message),
  };
}
