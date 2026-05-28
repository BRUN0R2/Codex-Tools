import type { ActionStatus } from "../Domain/ActionStatus";
import type { PersistentPriorityStatus } from "../Domain/PersistentPriority";

export type PersistentPriorityActionPanelProps = Readonly<{
  actionStatus: ActionStatus;
  persistentPriorityStatus: PersistentPriorityStatus;
  onInstallPersistentHighPriority: () => void;
  onRemovePersistentHighPriority: () => void;
}>;

export function createPersistentPriorityActionPanel({
  actionStatus,
  persistentPriorityStatus,
  onInstallPersistentHighPriority,
  onRemovePersistentHighPriority,
}: PersistentPriorityActionPanelProps): HTMLElementTagNameMap["section"] {
  const panel = document.createElement("section");
  panel.className = "PersistentPriorityActionPanel";

  const installButton = document.createElement("button");
  installButton.className = "PrimaryButton";
  installButton.type = "button";
  installButton.disabled = !canInstallPersistentPriority(actionStatus, persistentPriorityStatus);
  installButton.textContent = "Instalar prioridade alta";
  installButton.addEventListener("click", onInstallPersistentHighPriority);

  const removeButton = document.createElement("button");
  removeButton.className = "SecondaryButton";
  removeButton.type = "button";
  removeButton.disabled = !canRemovePersistentPriority(actionStatus, persistentPriorityStatus);
  removeButton.textContent = "Remover prioridade alta";
  removeButton.addEventListener("click", onRemovePersistentHighPriority);

  panel.append(installButton, removeButton, createPersistentPriorityStatusText(persistentPriorityStatus));

  return panel;
}

function canInstallPersistentPriority(
  actionStatus: ActionStatus,
  persistentPriorityStatus: PersistentPriorityStatus
): boolean {
  return actionStatus.state !== "Running" && persistentPriorityStatus.state !== "Installed";
}

function canRemovePersistentPriority(
  actionStatus: ActionStatus,
  persistentPriorityStatus: PersistentPriorityStatus
): boolean {
  return (
    actionStatus.state !== "Running" &&
    (persistentPriorityStatus.state === "Installed" ||
      persistentPriorityStatus.state === "PartiallyInstalled")
  );
}

function createPersistentPriorityStatusText(
  persistentPriorityStatus: PersistentPriorityStatus
): HTMLParagraphElement {
  const text = document.createElement("p");
  text.className = "ActionStatusText";

  switch (persistentPriorityStatus.state) {
    case "Unchecked":
      text.textContent = "Prioridade alta nao verificada";
      break;
    case "Installed":
      text.textContent = "Prioridade alta instalada";
      break;
    case "PartiallyInstalled":
      text.textContent = "Prioridade alta parcial";
      break;
    case "NotInstalled":
      text.textContent = "Prioridade alta nao instalada";
      break;
    case "Failed":
      text.textContent = persistentPriorityStatus.message;
      break;
  }

  return text;
}
