import type { ActionStatus } from "../Domain/ActionStatus";
import type { CodexUninstallReport, CodexUninstallTarget } from "../Domain/CodexUninstall";
import { formatBytes } from "../i18n/format";
import { translate, translatePlural } from "../i18n/catalog";
import { translateUninstallDiagnostic, translateUninstallTargetName } from "../i18n/diagnostics";

export type CodexUninstallPanelProps = Readonly<{
  actionStatus: ActionStatus;
  confirmationArmed: boolean;
  nativeRuntimeAvailable: boolean;
  uninstallReport: CodexUninstallReport | null;
  onArmConfirmation: () => void;
  onCancelConfirmation: () => void;
  onUninstall: () => void;
}>;

type UninstallSummaryValue = readonly [label: string, value: string];

export function createCodexUninstallPanel({
  actionStatus,
  confirmationArmed,
  nativeRuntimeAvailable,
  uninstallReport,
  onArmConfirmation,
  onCancelConfirmation,
  onUninstall,
}: CodexUninstallPanelProps): HTMLElementTagNameMap["section"] {
  const panel = document.createElement("section");
  panel.className = "Panel CodexUninstallPanel";

  const header = document.createElement("header");
  header.className = "CodexCleanupHeader";

  const title = document.createElement("h2");
  title.className = "SectionTitle";
  title.textContent = translate("uninstall.title");

  const heading = document.createElement("div");
  heading.className = "SectionHeading";

  const description = document.createElement("p");
  description.className = "SectionDescription";
  description.textContent =
    translate("uninstall.description");
  heading.append(title, description);

  const actions = document.createElement("div");
  actions.className = "UninstallActions";

  if (confirmationArmed) {
    const cancelButton = document.createElement("button");
    cancelButton.className = "SecondaryButton";
    cancelButton.type = "button";
    cancelButton.disabled = actionStatus.state === "Running";
    cancelButton.textContent = translate("uninstall.cancel");
    cancelButton.addEventListener("click", onCancelConfirmation);

    const confirmButton = document.createElement("button");
    confirmButton.className = "DangerButton";
    confirmButton.type = "button";
    confirmButton.disabled = !nativeRuntimeAvailable || actionStatus.state === "Running";
    confirmButton.textContent =
      actionStatus.state === "Running" && actionStatus.label === "uninstall.confirming"
        ? translate("uninstall.confirming")
        : translate("uninstall.confirm");
    confirmButton.addEventListener("click", onUninstall);

    actions.append(cancelButton, confirmButton);
  } else {
    const armButton = document.createElement("button");
    armButton.className = "DangerButton";
    armButton.type = "button";
    armButton.disabled = !nativeRuntimeAvailable || actionStatus.state === "Running";
    armButton.textContent = translate("uninstall.removeAll");
    armButton.addEventListener("click", onArmConfirmation);
    actions.append(armButton);
  }

  header.append(heading, actions);
  panel.append(
    header,
    createScopeNotice(confirmationArmed),
    createUninstallSummary(uninstallReport)
  );

  if (uninstallReport !== null) {
    panel.append(createUninstallTargets(uninstallReport.targets));

    if (uninstallReport.warnings.length > 0) {
      panel.append(createWarningList(uninstallReport.warnings));
    }
  }

  return panel;
}

function createScopeNotice(confirmationArmed: boolean): HTMLElementTagNameMap["section"] {
  const section = document.createElement("section");
  section.className = confirmationArmed ? "UninstallScopeNotice IsArmed" : "UninstallScopeNotice";

  const title = document.createElement("strong");
  title.textContent = confirmationArmed
    ? translate("uninstall.confirmationTitle")
    : translate("uninstall.scopeTitle");

  const body = document.createElement("p");
  body.textContent = confirmationArmed
    ? translate("uninstall.confirmationBody")
    : translate("uninstall.scopeBody");

  section.append(title, body);
  return section;
}

function createUninstallSummary(
  uninstallReport: CodexUninstallReport | null
): HTMLElementTagNameMap["section"] {
  const summary = document.createElement("section");
  summary.className = "CleanupSummary";

  const values: readonly UninstallSummaryValue[] =
    uninstallReport === null
      ? [
          [translate("cleanup.files"), "-"],
          [translate("cleanup.folders"), "-"],
          [translate("uninstall.package"), "-"],
          [translate("cleanup.freed"), "-"],
        ]
      : [
          [translate("cleanup.files"), uninstallReport.removedFileCount.toString()],
          [translate("cleanup.folders"), uninstallReport.removedDirectoryCount.toString()],
          [translate("uninstall.package"), uninstallReport.packageRemoved
            ? translate("uninstall.removed")
            : translate("uninstall.absent")],
          [translate("cleanup.freed"), formatBytes(uninstallReport.freedBytes)],
        ];

  for (const [label, value] of values) {
    const item = document.createElement("div");
    item.className = "CleanupSummaryItem";

    const valueElement = document.createElement("strong");
    valueElement.textContent = value;

    const labelElement = document.createElement("span");
    labelElement.textContent = label;

    item.append(valueElement, labelElement);
    summary.append(item);
  }

  return summary;
}

function createUninstallTargets(
  targets: readonly CodexUninstallTarget[]
): HTMLElementTagNameMap["section"] {
  const section = document.createElement("section");
  section.className = "CleanupTargets";

  for (const target of targets) {
    const row = document.createElement("div");
    row.className = "CleanupTargetRow";

    const name = document.createElement("span");
    name.className = "CleanupTargetName";
    name.textContent = translateUninstallTargetName(target.name);

    const result = document.createElement("span");
    result.className = "CleanupTargetResult";
    result.textContent = formatTargetResult(target);

    row.append(name, result);
    section.append(row);
  }

  if (section.childElementCount === 0) {
    const empty = document.createElement("p");
    empty.className = "CleanupEmptyState";
    empty.textContent = translate("uninstall.empty");
    section.append(empty);
  }

  return section;
}

function createWarningList(warnings: readonly string[]): HTMLElementTagNameMap["section"] {
  const section = document.createElement("section");
  section.className = "CleanupWarnings";

  for (const warning of warnings) {
    const item = document.createElement("p");
    item.textContent = translateUninstallDiagnostic(warning);
    section.append(item);
  }

  return section;
}

function formatTargetResult(target: CodexUninstallTarget): string {
  const parts: string[] = [];

  if (target.removedFileCount > 0) {
    parts.push(translatePlural("uninstall.fileCount", target.removedFileCount, {
      count: target.removedFileCount,
    }));
  }

  if (target.removedDirectoryCount > 0) {
    parts.push(translatePlural("uninstall.folderCount", target.removedDirectoryCount, {
      count: target.removedDirectoryCount,
    }));
  }

  if (target.freedBytes > 0) {
    parts.push(formatBytes(target.freedBytes));
  }

  if (parts.length === 0) {
    return translateUninstallDiagnostic(target.details);
  }

  return `${parts.join(" | ")} · ${translateUninstallDiagnostic(target.details)}`;
}
