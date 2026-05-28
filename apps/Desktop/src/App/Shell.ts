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

export function createShell(): HTMLElementTagNameMap["section"] {
  const shell = document.createElement("section");
  shell.className = "AppShell";

  const header = document.createElement("header");
  header.className = "AppHeader";

  const heading = document.createElement("div");
  heading.append(
    createTextElement("h1", "AppTitle", "Codex Tools"),
    createTextElement("p", "AppStatus", "Base pronta")
  );

  const panel = document.createElement("section");
  panel.className = "Panel";
  panel.append(
    createTextElement("h2", "SectionTitle", "Controle"),
    createTextElement("p", "MutedText", "Aguardando comandos nativos.")
  );

  header.append(heading);
  shell.append(header, panel);

  return shell;
}
