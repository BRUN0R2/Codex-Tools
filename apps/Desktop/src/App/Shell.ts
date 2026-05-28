import type { AppState } from "./AppState";
import { createCodexActionPanel } from "../Ui/CodexActionPanel";
import { createCodexStatusPanel } from "../Ui/CodexStatusPanel";

type ShellProps = Readonly<{
  state: AppState;
  onCodexRefresh: () => void;
  onOpenCodex: () => void;
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

export function createShell({
  state,
  onCodexRefresh,
  onOpenCodex,
}: ShellProps): HTMLElementTagNameMap["section"] {
  const shell = document.createElement("section");
  shell.className = "AppShell";

  const header = document.createElement("header");
  header.className = "AppHeader";

  const heading = document.createElement("div");
  heading.append(
    createTextElement("h1", "AppTitle", "Codex Tools"),
    createTextElement("p", "AppStatus", "Prioridade Alta")
  );

  const panel = document.createElement("section");
  panel.className = "ControlSurface";

  panel.append(
    createTextElement("h2", "SectionTitle", "Controle"),
    createCodexStatusPanel({
      status: state.codexStatus,
      onRefresh: onCodexRefresh,
    }),
    createCodexActionPanel({
      actionStatus: state.actionStatus,
      codexStatus: state.codexStatus,
      onOpenCodex,
    })
  );

  header.append(heading);
  shell.append(header, panel);

  return shell;
}
