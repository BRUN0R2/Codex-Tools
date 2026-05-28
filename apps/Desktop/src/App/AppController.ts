import { getCodexStatus, openCodex } from "../Backend/CodexCommands";
import {
  INITIAL_APP_STATE,
  clearConsoleMessages,
  setOpeningRuntimeStatus,
  setCheckingCodexStatus,
  setCodexStatus,
  setFailedActionStatus,
  setFailedCodexStatus,
  setRunningActionStatus,
  setSucceededActionStatus,
  setWaitingRuntimeStatus,
  type AppState,
} from "./AppState";
import { createShell } from "./Shell";
import { codexStatusHasOnlyHighPriorityProcesses } from "../Domain/RuntimeStatus";

const OPEN_CODEX_ACTION_LABEL = "Abrindo Codex";
const CODEX_OPEN_STATUS_POLL_ATTEMPTS = 36;
const CODEX_OPEN_STATUS_POLL_INTERVAL_MILLISECONDS = 700;

export function mountApp(root: HTMLElement): void {
  let state: AppState = INITIAL_APP_STATE;
  let openingMonitorVersion = 0;

  function render(): void {
    root.replaceChildren(
      createShell({
        state,
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
      })
    );
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
      state = setSucceededActionStatus(state, "Codex iniciado. Ajustando prioridade em segundo plano.");
      render();
      void monitorCodexOpening(monitorVersion);
      return;
    }

    state = setFailedActionStatus(setWaitingRuntimeStatus(state), result.error.message);
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

      if (result.ok && codexStatusHasOnlyHighPriorityProcesses(result.value)) {
        return;
      }
    }

    if (state.runtimeStatus === "Opening") {
      state = setWaitingRuntimeStatus(state);
      render();
    }
  }

  function delay(milliseconds: number): Promise<void> {
    return new Promise((resolve) => {
      window.setTimeout(resolve, milliseconds);
    });
  }

  render();
  void refreshCodexStatus();
}
