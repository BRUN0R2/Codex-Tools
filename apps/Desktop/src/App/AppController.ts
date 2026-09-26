import {
  cleanCodexWorkspace,
  getCodexStatus,
  openCodex,
  uninstallCodexProduct,
} from "../Backend/CodexCommands";
import type { AppSection } from "../Domain/AppSection";
import { codexStatusesAreEqual } from "../Domain/CodexInstallation";
import {
  CODEX_CLEANUP_ACTION_LABEL,
  type CodexCleanupReport,
} from "../Domain/CodexCleanup";
import {
  CODEX_UNINSTALL_ACTION_LABEL,
  type CodexUninstallReport,
} from "../Domain/CodexUninstall";
import type { CodexLaunchResponse } from "../Domain/CodexLaunch";
import {
  INITIAL_APP_STATE,
  clearConsoleMessages,
  setActiveSection,
  setOpeningRuntimeStatus,
  setCheckingCodexStatus,
  setCodexStatus,
  setCleanupReport,
  setFailedActionStatus,
  setFailedCodexStatus,
  setRunningActionStatus,
  setSucceededActionStatus,
  setUninstallConfirmationArmed,
  setUninstallReport,
  setWaitingRuntimeStatus,
  type AppState,
} from "./AppState";
import { createShell } from "./Shell";
import {
  priorityStabilizationHasFailed,
  priorityStabilizationHasFinished,
} from "../Domain/PriorityStabilization";

const OPEN_CODEX_ACTION_LABEL = "Abrindo Codex";
const CODEX_OPEN_STATUS_POLL_INTERVAL_MILLISECONDS = 500;
const CODEX_OPEN_STATUS_POLL_BUDGET_MILLISECONDS = 35_000;

export function mountApp(root: HTMLElement): void {
  let state: AppState = INITIAL_APP_STATE;
  let openingMonitorVersion = 0;

  function render(): void {
    root.replaceChildren(
      createShell({
        state,
        onArmUninstallConfirmation(): void {
          armUninstallConfirmation();
        },
        onCancelUninstallConfirmation(): void {
          cancelUninstallConfirmation();
        },
        onCleanCodex(): void {
          void cleanCodex();
        },
        onCodexRefresh(): void {
          void refreshCodexStatus();
        },
        onConsoleClear(): void {
          clearConsole();
        },
        onConsoleCopy(): void {
          void copyConsole();
        },
        onOpenCodex(): void {
          void launchCodex();
        },
        onSelectSection(section: AppSection): void {
          selectSection(section);
        },
        onUninstallCodex(): void {
          void uninstallCodex();
        },
      })
    );
  }

  function selectSection(section: AppSection): void {
    state = setActiveSection(state, section);
    render();
  }

  async function refreshCodexStatus(): Promise<void> {
    state = setCheckingCodexStatus(state);
    render();

    const result = await getCodexStatus();

    state = result.ok
      ? setCodexStatus(state, result.value)
      : setFailedCodexStatus(state, result.error.message);
    render();
  }

  function clearConsole(): void {
    state = setSucceededActionStatus(clearConsoleMessages(state), "Console limpo.");
    render();
  }

  async function copyConsole(): Promise<void> {
    if (state.consoleMessages.length === 0) {
      state = setSucceededActionStatus(state, "Console vazio.");
      render();
      return;
    }

    try {
      await navigator.clipboard.writeText(state.consoleMessages.join("\n"));
      state = setSucceededActionStatus(state, "Console copiado.");
    } catch {
      state = setFailedActionStatus(state, "Nao foi possivel copiar o console.");
    }

    render();
  }

  async function launchCodex(): Promise<void> {
    const monitorVersion = openingMonitorVersion + 1;
    openingMonitorVersion = monitorVersion;
    state = setRunningActionStatus(setOpeningRuntimeStatus(state), OPEN_CODEX_ACTION_LABEL);
    render();

    const result = await openCodex();

    if (result.ok) {
      state = setSucceededActionStatus(state, createCodexLaunchSuccessMessage(result.value));
      render();
      void monitorCodexOpening(monitorVersion);
      return;
    }

    state = setFailedActionStatus(setWaitingRuntimeStatus(state), result.error.message);
    render();
    await refreshCodexStatus();
  }

  async function cleanCodex(): Promise<void> {
    state = setRunningActionStatus(state, CODEX_CLEANUP_ACTION_LABEL);
    render();

    const result = await cleanCodexWorkspace();
    if (!result.ok) {
      state = setFailedActionStatus(state, result.error.message);
      render();
      return;
    }

    state = setCleanupReport(
      setSucceededActionStatus(state, createCleanupSuccessMessage(result.value)),
      result.value
    );
    render();
    await refreshCodexStatus();
  }

  function armUninstallConfirmation(): void {
    state = setUninstallConfirmationArmed(state, true);
    render();
  }

  function cancelUninstallConfirmation(): void {
    state = setUninstallConfirmationArmed(state, false);
    render();
  }

  async function uninstallCodex(): Promise<void> {
    state = setRunningActionStatus(state, CODEX_UNINSTALL_ACTION_LABEL);
    render();

    const result = await uninstallCodexProduct();
    if (!result.ok) {
      state = setFailedActionStatus(
        setUninstallConfirmationArmed(state, false),
        result.error.message
      );
      render();
      return;
    }

    state = setUninstallReport(
      setSucceededActionStatus(state, createUninstallSuccessMessage(result.value)),
      result.value
    );
    render();
    await refreshCodexStatus();
  }

  async function monitorCodexOpening(monitorVersion: number): Promise<void> {
    const deadline = Date.now() + CODEX_OPEN_STATUS_POLL_BUDGET_MILLISECONDS;

    while (Date.now() < deadline) {
      await delay(CODEX_OPEN_STATUS_POLL_INTERVAL_MILLISECONDS);

      if (monitorVersion !== openingMonitorVersion) {
        return;
      }

      const result = await getCodexStatus();
      const previousCodexStatus = state.codexStatus;
      const previousRuntimeStatus = state.runtimeStatus;
      state = result.ok
        ? setCodexStatus(state, result.value)
        : setFailedCodexStatus(state, result.error.message);

      if (
        !codexStatusesAreEqual(previousCodexStatus, state.codexStatus) ||
        previousRuntimeStatus !== state.runtimeStatus
      ) {
        render();
      }

      if (result.ok && priorityStabilizationHasFailed(result.value.priorityStabilization)) {
        state = setFailedActionStatus(
          state,
          result.value.priorityStabilization.message ??
            "Nao foi possivel estabilizar a prioridade alta do Codex."
        );
        render();
        return;
      }

      if (result.ok && priorityStabilizationHasFinished(result.value.priorityStabilization)) {
        return;
      }
    }

    if (state.runtimeStatus === "Opening") {
      state = setFailedActionStatus(
        setWaitingRuntimeStatus(state),
        "Nao foi possivel confirmar a prioridade alta do Codex no tempo esperado."
      );
      render();
    }
  }

  function delay(milliseconds: number): Promise<void> {
    return new Promise((resolve) => {
      window.setTimeout(resolve, milliseconds);
    });
  }

  function createCodexLaunchSuccessMessage(response: CodexLaunchResponse): string {
    return `Codex administrativo ativo (PID ${response.processId}, app-server elevado). Ajustando prioridade.`;
  }

  function createCleanupSuccessMessage(report: CodexCleanupReport): string {
    return `Limpeza concluida: ${report.removedThreadCount} conversa(s), ${report.removedFileCount} arquivo(s), ${formatBytes(report.freedBytes)} liberados.`;
  }

  function createUninstallSuccessMessage(report: CodexUninstallReport): string {
    const packageState = report.packageRemoved
      ? "pacote removido"
      : "pacote ausente ou nao removido";
    return `Desinstalacao concluida: ${report.removedFileCount} arquivo(s), ${report.removedDirectoryCount} pasta(s), ${formatBytes(report.freedBytes)} liberados; ${packageState}.`;
  }

  function formatBytes(bytes: number): string {
    const units = ["B", "KB", "MB", "GB"] as const;
    let value = bytes;
    let unitIndex = 0;

    while (value >= 1024 && unitIndex < units.length - 1) {
      value /= 1024;
      unitIndex += 1;
    }

    return `${value.toFixed(unitIndex === 0 ? 0 : 2)} ${units[unitIndex]}`;
  }

  render();
  void refreshCodexStatus();
}
