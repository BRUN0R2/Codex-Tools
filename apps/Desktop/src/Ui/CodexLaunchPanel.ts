import type { ActionStatus } from "../Domain/ActionStatus";
import type { CodexCliStatus } from "../Domain/CodexCli";
import type { CodexStatus } from "../Domain/CodexInstallation";
import type { CodexLaunchTarget } from "../Domain/CodexLaunchTarget";
import type { RuntimeStatus } from "../Domain/RuntimeStatus";
import { translate } from "../i18n/catalog";
import { createLaunchIcon, createRefreshIcon } from "./ApplicationIcons";

export type CodexLaunchPanelProps = Readonly<{
  actionStatus: ActionStatus;
  codexStatus: CodexStatus;
  cliStatus: CodexCliStatus;
  nativeRuntimeAvailable: boolean;
  selectedTarget: CodexLaunchTarget;
  runtimeStatus: RuntimeStatus;
  onOpen: () => void;
  onRefresh: () => void;
  onSelectTarget: (target: CodexLaunchTarget) => void;
}>;

export function createCodexLaunchPanel({
  actionStatus,
  codexStatus,
  cliStatus,
  nativeRuntimeAvailable,
  selectedTarget,
  runtimeStatus,
  onOpen,
  onRefresh,
  onSelectTarget,
}: CodexLaunchPanelProps): HTMLElementTagNameMap["section"] {
  const panel = document.createElement("section");
  panel.className = "Panel CodexLaunchPanel";

  const header = document.createElement("header");
  header.className = "PanelHeader CodexLaunchHeader";

  const heading = document.createElement("div");
  heading.className = "SectionHeading";

  const title = document.createElement("h2");
  title.className = "SectionTitle";
  title.textContent = translate("launch.title");

  const description = document.createElement("p");
  description.className = "SectionDescription";
  description.textContent = translate("launch.description");
  heading.append(title, description);

  const actions = document.createElement("div");
  actions.className = "CodexActionButtons";

  const targetField = document.createElement("label");
  targetField.className = "LaunchTargetField";
  const targetLabel = document.createElement("span");
  targetLabel.textContent = translate("launch.target");
  const targetSelect = document.createElement("select");
  targetSelect.className = "FieldSelect";
  targetSelect.disabled = actionStatus.state === "Running" || runtimeStatus === "Opening";
  for (const target of ["desktop", "cli"] as const) {
    const option = document.createElement("option");
    option.value = target;
    option.textContent = translate(target === "desktop" ? "status.desktop" : "status.cli");
    targetSelect.append(option);
  }
  targetSelect.value = selectedTarget;
  targetSelect.addEventListener("change", () => {
    if (targetSelect.value === "desktop" || targetSelect.value === "cli") {
      onSelectTarget(targetSelect.value);
    }
  });
  targetField.append(targetLabel, targetSelect);

  const selectedStatus = selectedTarget === "desktop" ? codexStatus : cliStatus;
  const refresh = createActionButton(
    "SecondaryButton",
    selectedStatus.state === "Checking" ? translate("action.checking") : translate("action.refresh"),
    createRefreshIcon(),
    onRefresh,
  );
  refresh.disabled = !nativeRuntimeAvailable ||
    selectedStatus.state === "Checking" || actionStatus.state === "Running";

  const opening = actionStatus.state === "Running" &&
    actionStatus.label === (selectedTarget === "desktop" ? "action.openingDesktop" : "cli.opening");
  const open = createActionButton(
    "PrimaryButton",
    opening ? translate("launch.opening") : translate("launch.open"),
    createLaunchIcon(),
    onOpen,
  );
  open.disabled = !nativeRuntimeAvailable || selectedStatus.state !== "Found" ||
    actionStatus.state === "Running" ||
    (selectedTarget === "desktop" && runtimeStatus === "Opening");
  if (opening) {
    open.classList.add("IsBusy");
    open.setAttribute("aria-busy", "true");
  }

  actions.append(targetField, refresh, open);
  header.append(heading, actions);

  const status = document.createElement("div");
  status.className = "CodexActionStatus";
  status.setAttribute("aria-atomic", "true");
  status.setAttribute("aria-live", "polite");
  if (nativeRuntimeAvailable && selectedTarget === "desktop") {
    const detail = document.createElement("p");
    detail.className = `RuntimeStatus RuntimeStatus--${runtimeStatus}`;
    detail.textContent = runtimeStatusText(runtimeStatus);
    status.append(detail);
  }

  if (actionStatus.state !== "Idle") {
    const feedback = document.createElement("p");
    feedback.className = `ActionFeedback ActionFeedback--${actionStatus.state}`;
    switch (actionStatus.state) {
      case "Running":
        feedback.textContent = translate("action.runningFeedback", {
          label: translate(actionStatus.label),
        });
        break;
      case "Succeeded":
        feedback.textContent = actionStatus.message;
        break;
      case "Failed":
        feedback.textContent = translate("action.errorPrefix", { message: actionStatus.message });
        break;
    }
    status.append(feedback);
  }

  panel.append(header);
  if (status.childElementCount > 0) {
    panel.append(status);
  }
  return panel;
}

function createActionButton(
  className: string,
  label: string,
  icon: SVGSVGElement,
  onClick: () => void,
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

function runtimeStatusText(status: RuntimeStatus): string {
  switch (status) {
    case "Waiting":
      return translate("action.waitingStatus");
    case "Opening":
      return translate("action.openingStatus");
    case "Ready":
      return translate("action.readyStatus");
  }
}
