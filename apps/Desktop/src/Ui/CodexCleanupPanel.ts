import type { ActionStatus } from "../Domain/ActionStatus";
import type { CodexCleanupReport, CodexCleanupTarget } from "../Domain/CodexCleanup";
import { formatBytes } from "../i18n/format";
import { translate, translatePlural } from "../i18n/catalog";
import { translateCleanupDiagnostic, translateCleanupTargetName } from "../i18n/diagnostics";

export type CodexCleanupPanelProps = Readonly<{
  actionStatus: ActionStatus;
  cleanupReport: CodexCleanupReport | null;
  onClean: () => void;
}>;

type CleanupSummaryValue = readonly [label: string, value: string];

export function createCodexCleanupPanel({
  actionStatus,
  cleanupReport,
  onClean,
}: CodexCleanupPanelProps): HTMLElementTagNameMap["section"] {
  const panel = document.createElement("section");
  panel.className = "Panel CodexCleanupPanel";

  const header = document.createElement("header");
  header.className = "CodexCleanupHeader";

  const title = document.createElement("h2");
  title.className = "SectionTitle";
  title.textContent = translate("cleanup.title");

  const heading = document.createElement("div");
  heading.className = "SectionHeading";

  const description = document.createElement("p");
  description.className = "SectionDescription";
  description.textContent = translate("cleanup.description");
  heading.append(title, description);

  const cleanButton = document.createElement("button");
  cleanButton.className = "DangerButton";
  cleanButton.type = "button";
  cleanButton.disabled = actionStatus.state === "Running";
  cleanButton.textContent =
    actionStatus.state === "Running" && actionStatus.label === translate("cleanup.cleaning")
      ? translate("cleanup.cleaning")
      : translate("cleanup.clean");
  cleanButton.addEventListener("click", onClean);

  header.append(heading, cleanButton);
  panel.append(header, createCleanupSummary(cleanupReport));

  if (cleanupReport !== null) {
    panel.append(createCleanupTargets(cleanupReport.targets));

    if (cleanupReport.warnings.length > 0) {
      panel.append(createWarningList(cleanupReport.warnings));
    }
  }

  return panel;
}

function createCleanupSummary(
  cleanupReport: CodexCleanupReport | null
): HTMLElementTagNameMap["section"] {
  const summary = document.createElement("section");
  summary.className = "CleanupSummary";

  const values: readonly CleanupSummaryValue[] = cleanupReport === null
    ? [
        [translate("cleanup.threads"), "-"],
        [translate("cleanup.files"), "-"],
        [translate("cleanup.folders"), "-"],
        [translate("cleanup.freed"), "-"],
      ]
    : [
        [translate("cleanup.threads"), cleanupReport.removedThreadCount.toString()],
        [translate("cleanup.files"), cleanupReport.removedFileCount.toString()],
        [translate("cleanup.folders"), cleanupReport.removedDirectoryCount.toString()],
        [translate("cleanup.freed"), formatBytes(cleanupReport.freedBytes)],
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

function createCleanupTargets(
  targets: readonly CodexCleanupTarget[]
): HTMLElementTagNameMap["section"] {
  const section = document.createElement("section");
  section.className = "CleanupTargets";

  for (const target of targets.filter(targetHasVisibleResult)) {
    const row = document.createElement("div");
    row.className = "CleanupTargetRow";

    const name = document.createElement("span");
    name.className = "CleanupTargetName";
    name.textContent = translateCleanupTargetName(target.name);

    const result = document.createElement("span");
    result.className = "CleanupTargetResult";
    result.textContent = formatTargetResult(target);

    row.append(name, result);
    section.append(row);
  }

  if (section.childElementCount === 0) {
    const empty = document.createElement("p");
    empty.className = "CleanupEmptyState";
    empty.textContent = translate("cleanup.empty");
    section.append(empty);
  }

  return section;
}

function createWarningList(warnings: readonly string[]): HTMLElementTagNameMap["section"] {
  const section = document.createElement("section");
  section.className = "CleanupWarnings";

  for (const warning of warnings) {
    const item = document.createElement("p");
    item.textContent = translateCleanupDiagnostic(warning);
    section.append(item);
  }

  return section;
}

function targetHasVisibleResult(target: CodexCleanupTarget): boolean {
  return (
    target.removedThreadCount > 0 ||
    target.removedGlobalStateReferenceCount > 0 ||
    target.removedFileCount > 0 ||
    target.removedDirectoryCount > 0 ||
    target.freedBytes > 0
  );
}

function formatTargetResult(target: CodexCleanupTarget): string {
  const parts: string[] = [];

  if (target.removedThreadCount > 0) {
    parts.push(translatePlural("cleanup.threadCount", target.removedThreadCount, {
      count: target.removedThreadCount,
    }));
  }

  if (target.removedGlobalStateReferenceCount > 0) {
    parts.push(translatePlural("cleanup.referenceCount", target.removedGlobalStateReferenceCount, {
      count: target.removedGlobalStateReferenceCount,
    }));
  }

  if (target.removedFileCount > 0) {
    parts.push(translatePlural("cleanup.fileCount", target.removedFileCount, {
      count: target.removedFileCount,
    }));
  }

  if (target.removedDirectoryCount > 0) {
    parts.push(translatePlural("cleanup.folderCount", target.removedDirectoryCount, {
      count: target.removedDirectoryCount,
    }));
  }

  if (target.freedBytes > 0) {
    parts.push(formatBytes(target.freedBytes));
  }

  return parts.join(" | ");
}
