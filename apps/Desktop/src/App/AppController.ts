import { createShell } from "./Shell";
import { INITIAL_APP_STATE, selectProcessPriority, type AppState } from "./AppState";
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
      })
    );
  }

  render();
}
