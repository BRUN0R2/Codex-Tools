import { getCodexStatus, openCodex } from "../Backend/CodexCommands";
import {
  INITIAL_APP_STATE,
  clearConsoleMessages,
  setCheckingCodexStatus,
  setCodexStatus,
  setFailedActionStatus,
  setFailedCodexStatus,
  setRunningActionStatus,
  setSucceededActionStatus,
  type AppState,
} from "./AppState";
import { createShell } from "./Shell";

const OPEN_CODEX_ACTION_LABEL = "Abrindo Codex";

export function mountApp(root: HTMLElement): void {
  let state: AppState = INITIAL_APP_STATE;

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
    state = setRunningActionStatus(state, OPEN_CODEX_ACTION_LABEL);
    render();

    const result = await openCodex();

    if (result.ok) {
      state = setSucceededActionStatus(
        state,
        `Codex aberto em prioridade alta. ${result.value.updatedProcessCount} processo(s) ajustado(s).`
      );
      render();
      await refreshCodexStatus();
      return;
    }

    state = setFailedActionStatus(state, result.error.message);
    render();
  }

  render();
  void refreshCodexStatus();
}
