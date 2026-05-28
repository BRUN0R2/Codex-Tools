import type { ProcessPriority, SelectedProcessPriority } from "../Domain/ProcessPriority";

export type AppState = Readonly<{
  selectedPriority: SelectedProcessPriority;
}>;

export const INITIAL_APP_STATE: AppState = {
  selectedPriority: null,
};

export function selectProcessPriority(state: AppState, priority: ProcessPriority): AppState {
  return {
    ...state,
    selectedPriority: priority,
  };
}
