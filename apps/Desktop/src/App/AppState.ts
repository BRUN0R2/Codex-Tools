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
import {
  createCodexConsoleMessages,
  type ConsoleMessage,
} from "../Domain/CodexConsole";
import {
  WAITING_RUNTIME_STATUS,
  createOpeningRuntimeStatus,
  createRuntimeStatusFromCodexStatus,
  createWaitingRuntimeStatus,
  type RuntimeStatus,
} from "../Domain/RuntimeStatus";

export type AppState = Readonly<{
  actionStatus: ActionStatus;
  codexStatus: CodexStatus;
  consoleMessages: readonly ConsoleMessage[];
  runtimeStatus: RuntimeStatus;
}>;

export const INITIAL_APP_STATE: AppState = {
  actionStatus: IDLE_ACTION_STATUS,
  codexStatus: UNCHECKED_CODEX_STATUS,
  consoleMessages: createCodexConsoleMessages(UNCHECKED_CODEX_STATUS),
  runtimeStatus: WAITING_RUNTIME_STATUS,
};

export function setCheckingCodexStatus(state: AppState): AppState {
  return {
    ...state,
    codexStatus: CHECKING_CODEX_STATUS,
    consoleMessages: createCodexConsoleMessages(CHECKING_CODEX_STATUS),
  };
}

export function setCodexStatus(state: AppState, codexStatus: CodexStatus): AppState {
  return {
    ...state,
    codexStatus,
    consoleMessages: createCodexConsoleMessages(codexStatus),
    runtimeStatus: createRuntimeStatusFromCodexStatus(state.runtimeStatus, codexStatus),
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

export function clearConsoleMessages(state: AppState): AppState {
  return {
    ...state,
    consoleMessages: [],
  };
}

export function setOpeningRuntimeStatus(state: AppState): AppState {
  return {
    ...state,
    runtimeStatus: createOpeningRuntimeStatus(),
  };
}

export function setWaitingRuntimeStatus(state: AppState): AppState {
  return {
    ...state,
    runtimeStatus: createWaitingRuntimeStatus(),
  };
}
