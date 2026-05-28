import type { AppState } from "./AppState";
import { formatProcessPriority, type ProcessPriority } from "../Domain/ProcessPriority";
import { createCodexActionPanel } from "../Ui/CodexActionPanel";
import { createCodexStatusPanel } from "../Ui/CodexStatusPanel";
import { createPrioritySelector } from "../Ui/PrioritySelector";

type ShellProps = Readonly<{
  state: AppState;
  onCodexRefresh: () => void;
  onOpenCodex: () => void;
  onPriorityChange: (priority: ProcessPriority) => void;
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
  onOpenCodex,
  onPriorityChange,
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
    })
  );

  header.append(heading);
  shell.append(header, panel);

  return shell;
}
