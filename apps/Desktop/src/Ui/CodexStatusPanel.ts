import type { CodexStatus } from "../Domain/CodexInstallation";

export type CodexStatusPanelProps = Readonly<{
  status: CodexStatus;
}>;

export function createCodexStatusPanel({ status }: CodexStatusPanelProps): HTMLElementTagNameMap["section"] {
  const panel = document.createElement("section");
  panel.className = `CodexStatusPanel CodexStatusPanel--${status.state}`;

  const indicator = document.createElement("span");
  indicator.className = "CodexStatusIndicator";
  indicator.setAttribute("aria-hidden", "true");

  const statusText = document.createElement("p");
  statusText.className = "CodexStatusText";
  statusText.textContent = createCodexStatusText(status);

  panel.append(indicator, statusText);

  return panel;
}

function createCodexStatusText(status: CodexStatus): string {
  switch (status.state) {
    case "Unchecked":
      return "Codex nao verificado";
    case "Checking":
      return "Verificando Codex";
    case "Found":
      return status.executablePath;
    case "NotFound":
      return `${status.checkedPaths.length} caminhos verificados`;
    case "Failed":
      return status.message;
  }
}
