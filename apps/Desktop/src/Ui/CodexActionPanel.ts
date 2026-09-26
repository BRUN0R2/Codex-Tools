import type { ActionStatus } from "../Domain/ActionStatus";
import type { CodexStatus } from "../Domain/CodexInstallation";
import type { RuntimeStatus } from "../Domain/RuntimeStatus";
import { createLaunchIcon, createRefreshIcon } from "./ApplicationIcons";

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
  panel.className = "Panel CodexActionPanel";

  const header = document.createElement("header");
  header.className = "PanelHeader CodexActionHeader";

  const heading = document.createElement("div");
  heading.className = "SectionHeading";

  const title = document.createElement("h2");
  title.className = "SectionTitle";
  title.textContent = "Controle do Codex";

  const description = document.createElement("p");
  description.className = "SectionDescription";
  description.textContent = createControlDescription();
  heading.append(title, description);

  const actions = document.createElement("div");
  actions.className = "CodexActionButtons";

  const refreshLabel = codexStatus.state === "Checking" ? "Verificando" : "Verificar";
  const refreshCodexButton = createActionButton(
    "SecondaryButton",
    refreshLabel,
    createRefreshIcon(),
    onRefresh
  );
  refreshCodexButton.disabled = !canRefreshCodex(codexStatus);

  const isOpeningCodex = runtimeStatus === "Opening";
  const openLabel = isOpeningCodex
    ? "Abrindo Codex como administrador"
    : "Abrir Codex como administrador";
  const openCodexButton = createActionButton(
    "PrimaryButton",
    openLabel,
    createLaunchIcon(),
    onOpenCodex
  );
  openCodexButton.disabled = !canOpenCodex(codexStatus, actionStatus, runtimeStatus);
  if (isOpeningCodex) {
    openCodexButton.classList.add("IsBusy");
    openCodexButton.setAttribute("aria-busy", "true");
  }

  actions.append(refreshCodexButton, openCodexButton);
  header.append(heading, actions);

  const status = document.createElement("div");
  status.className = "CodexActionStatus";
  status.setAttribute("aria-atomic", "true");
  status.setAttribute("aria-live", "polite");
  status.append(createRuntimeStatusElement(runtimeStatus));

  const actionFeedback = createActionFeedbackElement(actionStatus);
  if (actionFeedback !== null) {
    status.append(actionFeedback);
  }

  panel.append(header, status);
  return panel;
}

function createActionButton(
  className: string,
  label: string,
  icon: SVGSVGElement,
  onClick: () => void
): HTMLButtonElement {
  const button = document.createElement("button");
  button.className = className;
  button.type = "button";

  const labelElement = document.createElement("span");
  labelElement.textContent = label;

  button.append(icon, labelElement);
  button.addEventListener("click", onClick);
  return button;
}

function canRefreshCodex(codexStatus: CodexStatus): boolean {
  return codexStatus.state !== "Checking";
}

function createControlDescription(): string {
  return "Abre uma sessao administrativa com perfil proprio, preservando a instancia atual.";
}

function canOpenCodex(
  codexStatus: CodexStatus,
  actionStatus: ActionStatus,
  runtimeStatus: RuntimeStatus
): boolean {
  return (
    codexStatus.state === "Found" &&
    actionStatus.state !== "Running" &&
    runtimeStatus !== "Opening"
  );
}

function createRuntimeStatusElement(runtimeStatus: RuntimeStatus): HTMLParagraphElement {
  const element = document.createElement("p");
  element.className = `RuntimeStatus RuntimeStatus--${runtimeStatus}`;
  element.textContent = `Prioridade: ${formatRuntimeStatus(runtimeStatus)}`;
  return element;
}

function createActionFeedbackElement(actionStatus: ActionStatus): HTMLParagraphElement | null {
  if (actionStatus.state === "Idle") {
    return null;
  }

  const element = document.createElement("p");
  element.className = `ActionFeedback ActionFeedback--${actionStatus.state}`;

  switch (actionStatus.state) {
    case "Running":
      element.textContent = `${actionStatus.label}...`;
      break;
    case "Succeeded":
      element.textContent = actionStatus.message;
      break;
    case "Failed":
      element.textContent = `Erro: ${actionStatus.message}`;
      break;
  }

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
