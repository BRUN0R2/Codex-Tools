import {
  IDLE_ACTION_STATUS,
  createFailedActionStatus,
  createRunningActionStatus,
  createSucceededActionStatus,
  type ActionStatus,
} from "../Domain/ActionStatus";
import { INITIAL_APP_SECTION, type AppSection } from "../Domain/AppSection";
import {
  INITIAL_CODEX_LAUNCH_TARGET,
  type CodexLaunchTarget,
} from "../Domain/CodexLaunchTarget";
import type { CodexCleanupReport } from "../Domain/CodexCleanup";
import {
  CHECKING_CODEX_CLI_STATUS,
  UNCHECKED_CODEX_CLI_STATUS,
  type CodexCliStatus,
} from "../Domain/CodexCli";
import type { CodexUninstallReport } from "../Domain/CodexUninstall";
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
import type { TranslationKey } from "../i18n/catalog";

export type AppState = Readonly<{
  actionStatus: ActionStatus;
  activeSection: AppSection;
  cleanupReport: CodexCleanupReport | null;
  codexStatus: CodexStatus;
  codexCliStatus: CodexCliStatus;
  consoleMessages: readonly ConsoleMessage[];
  nativeRuntimeAvailable: boolean;
  runtimeStatus: RuntimeStatus;
  selectedLaunchTarget: CodexLaunchTarget;
  uninstallConfirmationArmed: boolean;
  uninstallReport: CodexUninstallReport | null;
}>;

export const INITIAL_APP_STATE: AppState = {
  actionStatus: IDLE_ACTION_STATUS,
  activeSection: INITIAL_APP_SECTION,
  cleanupReport: null,
  codexStatus: UNCHECKED_CODEX_STATUS,
  codexCliStatus: UNCHECKED_CODEX_CLI_STATUS,
  consoleMessages: createCodexConsoleMessages(UNCHECKED_CODEX_STATUS),
  nativeRuntimeAvailable: false,
  runtimeStatus: WAITING_RUNTIME_STATUS,
  selectedLaunchTarget: INITIAL_CODEX_LAUNCH_TARGET,
  uninstallConfirmationArmed: false,
  uninstallReport: null,
};

export function setCheckingCodexStatus(state: AppState): AppState {
  return {
    ...state,
    codexStatus: CHECKING_CODEX_STATUS,
    consoleMessages: createCodexConsoleMessages(CHECKING_CODEX_STATUS),
  };
}

export function setCheckingCodexCliStatus(state: AppState): AppState {
  return { ...state, codexCliStatus: CHECKING_CODEX_CLI_STATUS };
}

export function setCodexCliStatus(state: AppState, codexCliStatus: CodexCliStatus): AppState {
  return { ...state, codexCliStatus };
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

export function setRunningActionStatus(state: AppState, label: TranslationKey): AppState {
  return {
    ...state,
    actionStatus: createRunningActionStatus(label),
  };
}

export function setSelectedLaunchTarget(
  state: AppState,
  selectedLaunchTarget: CodexLaunchTarget,
): AppState {
  return { ...state, actionStatus: IDLE_ACTION_STATUS, selectedLaunchTarget };
}

export function retranslateConsoleMessages(state: AppState): AppState {
  return state.consoleMessages.length === 0
    ? state
    : { ...state, consoleMessages: createCodexConsoleMessages(state.codexStatus) };
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

export function setActiveSection(state: AppState, activeSection: AppSection): AppState {
  return {
    ...state,
    activeSection,
    uninstallConfirmationArmed:
      activeSection === "Uninstall" ? state.uninstallConfirmationArmed : false,
  };
}

export function setCleanupReport(
  state: AppState,
  cleanupReport: CodexCleanupReport
): AppState {
  return {
    ...state,
    cleanupReport,
  };
}

export function setUninstallConfirmationArmed(
  state: AppState,
  uninstallConfirmationArmed: boolean
): AppState {
  return {
    ...state,
    uninstallConfirmationArmed,
  };
}

export function setUninstallReport(
  state: AppState,
  uninstallReport: CodexUninstallReport
): AppState {
  return {
    ...state,
    uninstallConfirmationArmed: false,
    uninstallReport,
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
