import type { CodexStatus } from "../Domain/CodexInstallation";

export type CodexStatusPanelProps = Readonly<{
  status: CodexStatus;
  onRefresh: () => void;
}>;

export function createCodexStatusPanel({ status, onRefresh }: CodexStatusPanelProps): HTMLElementTagNameMap["section"] {
  const panel = document.createElement("section");
  panel.className = "CodexStatusPanel";

  const statusText = document.createElement("p");
  statusText.className = "CodexStatusText";
  statusText.textContent = createCodexStatusText(status);

  const button = document.createElement("button");
  button.className = "PrimaryButton";
  button.type = "button";
  button.disabled = status.state === "Checking";
  button.textContent = status.state === "Checking" ? "Verificando" : "Verificar Codex";
  button.addEventListener("click", onRefresh);

  panel.append(statusText, button);

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
