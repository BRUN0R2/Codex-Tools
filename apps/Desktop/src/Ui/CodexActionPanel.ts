import type { ActionStatus } from "../Domain/ActionStatus";
import type { CodexStatus } from "../Domain/CodexInstallation";
import type { SelectedProcessPriority } from "../Domain/ProcessPriority";

export type CodexActionPanelProps = Readonly<{
  actionStatus: ActionStatus;
  codexStatus: CodexStatus;
  selectedPriority: SelectedProcessPriority;
  onOpenCodex: () => void;
}>;

export function createCodexActionPanel({
  actionStatus,
  codexStatus,
  selectedPriority,
  onOpenCodex,
}: CodexActionPanelProps): HTMLElementTagNameMap["section"] {
  const panel = document.createElement("section");
  panel.className = "CodexActionPanel";

  const openCodexButton = document.createElement("button");
  openCodexButton.className = "PrimaryButton";
  openCodexButton.type = "button";
  openCodexButton.disabled = !canOpenCodex(codexStatus, selectedPriority, actionStatus);
  openCodexButton.textContent = actionStatus.state === "Running" ? actionStatus.label : "Abrir Codex";
  openCodexButton.addEventListener("click", onOpenCodex);

  panel.append(openCodexButton, createActionStatusText(actionStatus));

  return panel;
}

function canOpenCodex(
  codexStatus: CodexStatus,
  selectedPriority: SelectedProcessPriority,
  actionStatus: ActionStatus
): boolean {
  return codexStatus.state === "Found" && selectedPriority !== null && actionStatus.state !== "Running";
}

function createActionStatusText(actionStatus: ActionStatus): HTMLParagraphElement {
  const text = document.createElement("p");
  text.className = "ActionStatusText";

  switch (actionStatus.state) {
    case "Idle":
      text.textContent = "Selecione a prioridade e verifique o Codex";
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
