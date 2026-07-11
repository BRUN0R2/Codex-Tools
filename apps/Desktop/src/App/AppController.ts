import {
  cleanCodexWorkspace,
  getCodexStatus,
  openCodex,
  registerCodexRunAsAdministrator,
} from "../Backend/CodexCommands";
import type { AppSection } from "../Domain/AppSection";
import {
  CODEX_CLEANUP_ACTION_LABEL,
  type CodexCleanupReport,
} from "../Domain/CodexCleanup";
import type { CodexLaunchMethod, CodexLaunchResponse } from "../Domain/CodexLaunch";
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
  setWaitingRuntimeStatus,
  type AppState,
} from "./AppState";
import { createShell } from "./Shell";
import {
  priorityStabilizationHasFailed,
  priorityStabilizationHasFinished,
} from "../Domain/PriorityStabilization";

const OPEN_CODEX_ACTION_LABEL = "Abrindo Codex";
const SAVE_ADMIN_MODE_ACTION_LABEL = "Salvando admin";
const CODEX_OPEN_STATUS_POLL_ATTEMPTS = 36;
const CODEX_OPEN_STATUS_POLL_INTERVAL_MILLISECONDS = 700;

export function mountApp(root: HTMLElement): void {
  let state: AppState = INITIAL_APP_STATE;
  let openingMonitorVersion = 0;

  function render(): void {
    root.replaceChildren(
      createShell({
        state,
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
        onRegisterRunAsAdministrator(): void {
          void saveRunAsAdministratorMode();
        },
        onSelectSection(section: AppSection): void {
          selectSection(section);
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

  async function saveRunAsAdministratorMode(): Promise<void> {
    state = setRunningActionStatus(state, SAVE_ADMIN_MODE_ACTION_LABEL);
    render();

    const result = await registerCodexRunAsAdministrator();
    if (!result.ok) {
      state = setFailedActionStatus(state, result.error.message);
      render();
      return;
    }

    const registeredCount = result.value.registeredExecutablePaths.length;
    state = setSucceededActionStatus(
      state,
      `Modo administrador salvo para ${registeredCount} executavel(is) do Codex.`
    );
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

  async function monitorCodexOpening(monitorVersion: number): Promise<void> {
    for (let attemptIndex = 0; attemptIndex < CODEX_OPEN_STATUS_POLL_ATTEMPTS; attemptIndex += 1) {
      await delay(CODEX_OPEN_STATUS_POLL_INTERVAL_MILLISECONDS);

      if (monitorVersion !== openingMonitorVersion) {
        return;
      }

      const result = await getCodexStatus();
      state = result.ok
        ? setCodexStatus(state, result.value)
        : setFailedCodexStatus(state, result.error.message);
      render();

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
    const method = formatCodexLaunchMethod(response.launchMethod);
    const elevation = response.appServerElevationObserved
      ? "app-server elevado confirmado"
      : "app-server elevado ainda nao confirmado";
    const fallback = response.fallbackUsed ? " com fallback" : "";
    const diagnostic =
      response.diagnosticMessage === null ? "" : ` Detalhe: ${response.diagnosticMessage}`;

    return `Codex iniciado via ${method}${fallback}; ${elevation}. Ajustando prioridade.${diagnostic}`;
  }

  function createCleanupSuccessMessage(report: CodexCleanupReport): string {
    return `Limpeza concluida: ${report.removedThreadCount} conversa(s), ${report.removedFileCount} arquivo(s), ${formatBytes(report.freedBytes)} liberados.`;
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

  function formatCodexLaunchMethod(method: CodexLaunchMethod): string {
    switch (method) {
      case "ElevatedScheduledTask":
        return "tarefa elevada";
      case "ShellExecuteRunAs":
        return "runas";
    }
  }

  render();
  void refreshCodexStatus();
}
