import type { ActionStatus } from "../Domain/ActionStatus";
import type { CodexStatus } from "../Domain/CodexInstallation";

export type CodexActionPanelProps = Readonly<{
  actionStatus: ActionStatus;
  codexStatus: CodexStatus;
  onRefresh: () => void;
  onOpenCodex: () => void;
}>;

export function createCodexActionPanel({
  actionStatus,
  codexStatus,
  onRefresh,
  onOpenCodex,
}: CodexActionPanelProps): HTMLElementTagNameMap["section"] {
  const panel = document.createElement("section");
  panel.className = "CodexActionPanel";

  const actions = document.createElement("div");
  actions.className = "CodexActionButtons";

  const refreshCodexButton = document.createElement("button");
  refreshCodexButton.className = "SecondaryButton";
  refreshCodexButton.type = "button";
  refreshCodexButton.disabled = !canRefreshCodex(codexStatus, actionStatus);
  refreshCodexButton.textContent = codexStatus.state === "Checking" ? "Verificando" : "Verificar Codex";
  refreshCodexButton.addEventListener("click", onRefresh);

  const openCodexButton = document.createElement("button");
  openCodexButton.className = "PrimaryButton";
  openCodexButton.type = "button";
  openCodexButton.disabled = !canOpenCodex(codexStatus, actionStatus);
  openCodexButton.textContent = actionStatus.state === "Running" ? actionStatus.label : "Abrir Codex";
  openCodexButton.addEventListener("click", onOpenCodex);

  actions.append(refreshCodexButton, openCodexButton);
  panel.append(createActionStatusText(actionStatus), actions);

  return panel;
}

function canRefreshCodex(codexStatus: CodexStatus, actionStatus: ActionStatus): boolean {
  return codexStatus.state !== "Checking" && actionStatus.state !== "Running";
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
