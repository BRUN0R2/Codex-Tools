import type { ActionStatus } from "../Domain/ActionStatus";
import type { CodexStatus } from "../Domain/CodexInstallation";
import type { RuntimeStatus } from "../Domain/RuntimeStatus";

export type CodexActionPanelProps = Readonly<{
  actionStatus: ActionStatus;
  codexStatus: CodexStatus;
  onRefresh: () => void;
  onOpenCodex: () => void;
  runtimeStatus: RuntimeStatus;
}>;

export function createCodexActionPanel({
  actionStatus,
  codexStatus,
  onRefresh,
  onOpenCodex,
  runtimeStatus,
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
  openCodexButton.disabled = !canOpenCodex(codexStatus, actionStatus, runtimeStatus);
  openCodexButton.textContent = actionStatus.state === "Running" ? actionStatus.label : "Abrir Codex";
  openCodexButton.addEventListener("click", onOpenCodex);

  actions.append(refreshCodexButton, openCodexButton);
  panel.append(createRuntimeStatusElement(runtimeStatus), actions);

  return panel;
}

function canRefreshCodex(codexStatus: CodexStatus, actionStatus: ActionStatus): boolean {
  return codexStatus.state !== "Checking" && actionStatus.state !== "Running";
}

function canOpenCodex(
  codexStatus: CodexStatus,
  actionStatus: ActionStatus,
  runtimeStatus: RuntimeStatus
): boolean {
  return codexStatus.state === "Found" && actionStatus.state !== "Running" && runtimeStatus !== "Opening";
}

function createRuntimeStatusElement(runtimeStatus: RuntimeStatus): HTMLParagraphElement {
  const element = document.createElement("p");
  element.className = `RuntimeStatus RuntimeStatus--${runtimeStatus}`;
  element.textContent = `Estatus: ${formatRuntimeStatus(runtimeStatus)}`;
  return element;
}

function formatRuntimeStatus(runtimeStatus: RuntimeStatus): string {
  switch (runtimeStatus) {
    case "Waiting":
      return "esperando...";
    case "Opening":
      return "abrindo codex...";
    case "Ready":
      return "pronto.";
  }
}
