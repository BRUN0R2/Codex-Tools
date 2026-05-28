import { createShell } from "./Shell";
import {
  INITIAL_APP_STATE,
  selectProcessPriority,
  setCheckingCodexStatus,
  setCodexStatus,
  setFailedCodexStatus,
  type AppState,
} from "./AppState";
import { getCodexStatus } from "../Backend/CodexCommands";
import type { ProcessPriority } from "../Domain/ProcessPriority";

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

  render();
}
