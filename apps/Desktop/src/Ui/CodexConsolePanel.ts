import type { ConsoleMessage } from "../Domain/CodexConsole";

export type CodexConsolePanelProps = Readonly<{
  messages: readonly ConsoleMessage[];
  onClear: () => void;
  onCopy: () => void;
}>;

export function createCodexConsolePanel({
  messages,
  onClear,
  onCopy,
}: CodexConsolePanelProps): HTMLElementTagNameMap["section"] {
  const panel = document.createElement("section");
  panel.className = "CodexConsolePanel";

  const header = document.createElement("header");
  header.className = "CodexConsoleHeader";

  const title = document.createElement("h2");
  title.className = "CodexConsoleTitle";
  title.textContent = "Console";

  const actions = document.createElement("div");
  actions.className = "CodexConsoleActions";
  actions.append(
    createConsoleButton("Limpar", messages.length === 0, onClear),
    createConsoleButton("Copiar", messages.length === 0, onCopy)
  );

  const output = document.createElement("pre");
  output.className = "CodexConsoleOutput";
  output.setAttribute("aria-live", "polite");
  output.textContent = messages.length === 0 ? "Console vazio." : messages.join("\n");

  header.append(title, actions);
  panel.append(header, output);

  return panel;
}

function createConsoleButton(
  label: string,
  disabled: boolean,
  onClick: () => void
): HTMLButtonElement {
  const button = document.createElement("button");
  button.className = "SecondaryButton";
  button.type = "button";
  button.disabled = disabled;
  button.textContent = label;
  button.addEventListener("click", onClick);
  return button;
}
