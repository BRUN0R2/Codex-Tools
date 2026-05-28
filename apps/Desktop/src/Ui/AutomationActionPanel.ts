import type { ActionStatus } from "../Domain/ActionStatus";
import type { AutomationStatus } from "../Domain/Automation";
import type { SelectedProcessPriority } from "../Domain/ProcessPriority";

export type AutomationActionPanelProps = Readonly<{
  actionStatus: ActionStatus;
  automationStatus: AutomationStatus;
  selectedPriority: SelectedProcessPriority;
  onInstallAutomation: () => void;
  onRemoveAutomation: () => void;
}>;

export function createAutomationActionPanel({
  actionStatus,
  automationStatus,
  selectedPriority,
  onInstallAutomation,
  onRemoveAutomation,
}: AutomationActionPanelProps): HTMLElementTagNameMap["section"] {
  const panel = document.createElement("section");
  panel.className = "AutomationActionPanel";

  const installButton = document.createElement("button");
  installButton.className = "PrimaryButton";
  installButton.type = "button";
  installButton.disabled = !canInstallAutomation(actionStatus, selectedPriority);
  installButton.textContent = "Instalar automacao";
  installButton.addEventListener("click", onInstallAutomation);

  const removeButton = document.createElement("button");
  removeButton.className = "SecondaryButton";
  removeButton.type = "button";
  removeButton.disabled = !canRemoveAutomation(actionStatus, automationStatus);
  removeButton.textContent = "Remover automacao";
  removeButton.addEventListener("click", onRemoveAutomation);

  panel.append(installButton, removeButton, createAutomationStatusText(automationStatus));

  return panel;
}

function canInstallAutomation(actionStatus: ActionStatus, selectedPriority: SelectedProcessPriority): boolean {
  return actionStatus.state !== "Running" && selectedPriority !== null;
}

function canRemoveAutomation(actionStatus: ActionStatus, automationStatus: AutomationStatus): boolean {
  return actionStatus.state !== "Running" && automationStatus.state === "Installed";
}

function createAutomationStatusText(automationStatus: AutomationStatus): HTMLParagraphElement {
  const text = document.createElement("p");
  text.className = "ActionStatusText";

  switch (automationStatus.state) {
    case "Unchecked":
      text.textContent = "Automacao nao verificada";
      break;
    case "Installed":
      text.textContent = automationStatus.taskName;
      break;
    case "NotInstalled":
      text.textContent = "Automacao nao instalada";
      break;
    case "Failed":
      text.textContent = automationStatus.message;
      break;
  }

  return text;
}
