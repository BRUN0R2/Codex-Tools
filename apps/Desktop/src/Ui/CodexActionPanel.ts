import type { ActionStatus } from "../Domain/ActionStatus";
import type { CodexStatus } from "../Domain/CodexInstallation";

export type CodexActionPanelProps = Readonly<{
  actionStatus: ActionStatus;
  codexStatus: CodexStatus;
  onOpenCodex: () => void;
}>;

export function createCodexActionPanel({
  actionStatus,
  codexStatus,
  onOpenCodex,
}: CodexActionPanelProps): HTMLElementTagNameMap["section"] {
  const panel = document.createElement("section");
  panel.className = "CodexActionPanel";

  const openCodexButton = document.createElement("button");
  openCodexButton.className = "PrimaryButton";
  openCodexButton.type = "button";
  openCodexButton.disabled = !canOpenCodex(codexStatus, actionStatus);
  openCodexButton.textContent = actionStatus.state === "Running" ? actionStatus.label : "Abrir Codex";
  openCodexButton.addEventListener("click", onOpenCodex);

  panel.append(createActionStatusText(actionStatus), openCodexButton);

  return panel;
}

function canOpenCodex(codexStatus: CodexStatus, actionStatus: ActionStatus): boolean {
  return codexStatus.state === "Found" && actionStatus.state !== "Running";
}

function createActionStatusText(actionStatus: ActionStatus): HTMLParagraphElement {
  const text = document.createElement("p");
  text.className = "ActionStatusText";

  switch (actionStatus.state) {
    case "Idle":
      text.textContent = "Pronto para abrir Codex em prioridade alta.";
      break;
    case "Running":
      text.textContent = actionStatus.label;
      break;
    case "Succeeded":
      text.textContent = actionStatus.message;
      break;
    case "Failed":
      text.textContent = actionStatus.message;
      break;
  }

  return text;
}
