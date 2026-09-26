import type { ConsoleMessage } from "../Domain/CodexConsole";
import { translate } from "../i18n/catalog";

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
  panel.className = "Panel CodexConsolePanel";

  const header = document.createElement("header");
  header.className = "PanelHeader CodexConsoleHeader";

  const heading = document.createElement("div");
  heading.className = "SectionHeading";

  const title = document.createElement("h2");
  title.className = "SectionTitle";
  title.textContent = translate("console.title");

  const description = document.createElement("p");
  description.className = "SectionDescription";
  description.textContent = translate("console.description");
  heading.append(title, description);

  const actions = document.createElement("div");
  actions.className = "CodexConsoleActions";
  actions.append(
    createConsoleButton(translate("console.clear"), messages.length === 0, onClear),
    createConsoleButton(translate("console.copy"), messages.length === 0, onCopy)
  );

  const output = document.createElement("pre");
  output.className = "CodexConsoleOutput";
  output.setAttribute("aria-live", "polite");
  output.textContent = messages.length === 0 ? translate("console.empty") : messages.join("\n");

  header.append(heading, actions);
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
