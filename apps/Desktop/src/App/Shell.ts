import type { AppState } from "./AppState";
import { formatProcessPriority, type ProcessPriority } from "../Domain/ProcessPriority";
import { createAutomationActionPanel } from "../Ui/AutomationActionPanel";
import { createCodexActionPanel } from "../Ui/CodexActionPanel";
import { createCodexStatusPanel } from "../Ui/CodexStatusPanel";
import { createPersistentPriorityActionPanel } from "../Ui/PersistentPriorityActionPanel";
import { createPrioritySelector } from "../Ui/PrioritySelector";

type ShellProps = Readonly<{
  state: AppState;
  onCodexRefresh: () => void;
  onInstallAutomation: () => void;
  onInstallPersistentHighPriority: () => void;
  onOpenCodex: () => void;
  onPriorityChange: (priority: ProcessPriority) => void;
  onRemoveAutomation: () => void;
  onRemovePersistentHighPriority: () => void;
}>;

type TextElementTagName = "h1" | "h2" | "p";

function createTextElement<TagName extends TextElementTagName>(
  tagName: TagName,
  className: string,
  text: string
): HTMLElementTagNameMap[TagName] {
  const element = document.createElement(tagName);
  element.className = className;
  element.textContent = text;
  return element;
}

function createStatusText(state: AppState): string {
  if (state.selectedPriority === null) {
    return "Prioridade nao definida";
  }

  return `Prioridade ${formatProcessPriority(state.selectedPriority)}`;
}

export function createShell({
  state,
  onCodexRefresh,
  onInstallAutomation,
  onInstallPersistentHighPriority,
  onOpenCodex,
  onPriorityChange,
  onRemoveAutomation,
  onRemovePersistentHighPriority,
}: ShellProps): HTMLElementTagNameMap["section"] {
  const shell = document.createElement("section");
  shell.className = "AppShell";

  const header = document.createElement("header");
  header.className = "AppHeader";

  const heading = document.createElement("div");
  heading.append(
    createTextElement("h1", "AppTitle", "Codex Tools"),
    createTextElement("p", "AppStatus", createStatusText(state))
  );

  const panel = document.createElement("section");
  panel.className = "ControlSurface";

  panel.append(
    createTextElement("h2", "SectionTitle", "Controle"),
    createCodexStatusPanel({
      status: state.codexStatus,
      onRefresh: onCodexRefresh,
    }),
    createPrioritySelector({
      selectedPriority: state.selectedPriority,
      onChange: onPriorityChange,
    }),
    createCodexActionPanel({
      actionStatus: state.actionStatus,
      codexStatus: state.codexStatus,
      selectedPriority: state.selectedPriority,
      onOpenCodex,
    }),
    createPersistentPriorityActionPanel({
      actionStatus: state.actionStatus,
      persistentPriorityStatus: state.persistentPriorityStatus,
      onInstallPersistentHighPriority,
      onRemovePersistentHighPriority,
    }),
    createAutomationActionPanel({
      actionStatus: state.actionStatus,
      automationStatus: state.automationStatus,
      selectedPriority: state.selectedPriority,
      onInstallAutomation,
      onRemoveAutomation,
    })
  );

  header.append(heading);
  shell.append(header, panel);

  return shell;
}
