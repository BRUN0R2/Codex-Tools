import { createShell } from "./Shell";
import {
  INITIAL_APP_STATE,
  selectProcessPriority,
  setCheckingCodexStatus,
  setCodexStatus,
  setFailedActionStatus,
  setFailedCodexStatus,
  setRunningActionStatus,
  setSucceededActionStatus,
  type AppState,
} from "./AppState";
import { getCodexStatus, openCodex } from "../Backend/CodexCommands";
import type { ProcessPriority } from "../Domain/ProcessPriority";

const OPEN_CODEX_ACTION_LABEL = "Abrindo Codex";

export function mountApp(root: HTMLElement): void {
  let state: AppState = INITIAL_APP_STATE;

  function render(): void {
    root.replaceChildren(
      createShell({
        state,
        onPriorityChange(priority: ProcessPriority): void {
          state = selectProcessPriority(state, priority);
          render();
        },
        onCodexRefresh(): void {
          void refreshCodexStatus();
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

    state = result.ok ? setCodexStatus(state, result.value) : setFailedCodexStatus(state, result.error.message);
    render();
  }

  async function launchCodex(): Promise<void> {
    const selectedPriority = state.selectedPriority;

    if (selectedPriority === null) {
      state = setFailedActionStatus(state, "Selecione a prioridade antes de abrir o Codex.");
      render();
      return;
    }

    state = setRunningActionStatus(state, OPEN_CODEX_ACTION_LABEL);
    render();

    const result = await openCodex(selectedPriority);

    state = result.ok
      ? setSucceededActionStatus(
          state,
          `Codex aberto com ${result.value.updatedProcessCount} processo(s) ajustado(s).`
        )
      : setFailedActionStatus(state, result.error.message);
    render();
  }

  render();
}
