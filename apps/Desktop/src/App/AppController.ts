import { createShell } from "./Shell";
import {
  INITIAL_APP_STATE,
  selectProcessPriority,
  setAutomationStatus,
  setCheckingCodexStatus,
  setCodexStatus,
  setFailedActionStatus,
  setFailedAutomationStatus,
  setFailedCodexStatus,
  setFailedPersistentPriorityStatus,
  setPersistentPriorityStatus,
  setRunningActionStatus,
  setSucceededActionStatus,
  type AppState,
} from "./AppState";
import {
  getAutomationStatus,
  installCodexAutomation,
  removeCodexAutomation,
} from "../Backend/AutomationCommands";
import { getCodexStatus, openCodex } from "../Backend/CodexCommands";
import {
  getPersistentPriorityStatus,
  installPersistentHighPriority,
  removePersistentHighPriority,
} from "../Backend/PersistentPriorityCommands";
import type { ProcessPriority } from "../Domain/ProcessPriority";

const INSTALL_AUTOMATION_ACTION_LABEL = "Instalando automacao";
const INSTALL_PERSISTENT_PRIORITY_ACTION_LABEL = "Instalando prioridade alta";
const OPEN_CODEX_ACTION_LABEL = "Abrindo Codex";
const REMOVE_AUTOMATION_ACTION_LABEL = "Removendo automacao";
const REMOVE_PERSISTENT_PRIORITY_ACTION_LABEL = "Removendo prioridade alta";

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
        onInstallAutomation(): void {
          void installAutomation();
        },
        onInstallPersistentHighPriority(): void {
          void installPersistentPriority();
        },
        onOpenCodex(): void {
          void launchCodex();
        },
        onRemoveAutomation(): void {
          void removeAutomation();
        },
        onRemovePersistentHighPriority(): void {
          void removePersistentPriority();
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

  async function refreshAutomationStatus(): Promise<void> {
    const result = await getAutomationStatus();
    state = result.ok
      ? setAutomationStatus(state, result.value)
      : setFailedAutomationStatus(state, result.error.message);
    render();
  }

  async function refreshPersistentPriorityStatus(): Promise<void> {
    const result = await getPersistentPriorityStatus();
    state = result.ok
      ? setPersistentPriorityStatus(state, result.value)
      : setFailedPersistentPriorityStatus(state, result.error.message);
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

  async function installAutomation(): Promise<void> {
    const selectedPriority = state.selectedPriority;

    if (selectedPriority === null) {
      state = setFailedActionStatus(state, "Selecione a prioridade antes de instalar a automacao.");
      render();
      return;
    }

    state = setRunningActionStatus(state, INSTALL_AUTOMATION_ACTION_LABEL);
    render();

    const result = await installCodexAutomation(selectedPriority);

    state = result.ok
      ? setSucceededActionStatus(state, "Automacao instalada.")
      : setFailedActionStatus(state, result.error.message);
    state = result.ok
      ? setAutomationStatus(state, result.value)
      : setFailedAutomationStatus(state, result.error.message);
    render();
  }

  async function removeAutomation(): Promise<void> {
    state = setRunningActionStatus(state, REMOVE_AUTOMATION_ACTION_LABEL);
    render();

    const result = await removeCodexAutomation();

    state = result.ok
      ? setSucceededActionStatus(state, "Automacao removida.")
      : setFailedActionStatus(state, result.error.message);
    state = result.ok
      ? setAutomationStatus(state, result.value)
      : setFailedAutomationStatus(state, result.error.message);
    render();
  }

  async function installPersistentPriority(): Promise<void> {
    state = setRunningActionStatus(state, INSTALL_PERSISTENT_PRIORITY_ACTION_LABEL);
    render();

    const result = await installPersistentHighPriority();

    state = result.ok
      ? setSucceededActionStatus(state, "Prioridade alta instalada.")
      : setFailedActionStatus(state, result.error.message);
    state = result.ok
      ? setPersistentPriorityStatus(state, result.value)
      : setFailedPersistentPriorityStatus(state, result.error.message);
    render();
  }

  async function removePersistentPriority(): Promise<void> {
    state = setRunningActionStatus(state, REMOVE_PERSISTENT_PRIORITY_ACTION_LABEL);
    render();

    const result = await removePersistentHighPriority();

    state = result.ok
      ? setSucceededActionStatus(state, "Prioridade alta removida.")
      : setFailedActionStatus(state, result.error.message);
    state = result.ok
      ? setPersistentPriorityStatus(state, result.value)
      : setFailedPersistentPriorityStatus(state, result.error.message);
    render();
  }

  void refreshAutomationStatus();
  void refreshPersistentPriorityStatus();
  render();
}
