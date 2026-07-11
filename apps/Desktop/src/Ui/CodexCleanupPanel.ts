import type { ActionStatus } from "../Domain/ActionStatus";
import {
  CODEX_CLEANUP_ACTION_LABEL,
  type CodexCleanupReport,
  type CodexCleanupTarget,
} from "../Domain/CodexCleanup";

export type CodexCleanupPanelProps = Readonly<{
  actionStatus: ActionStatus;
  cleanupReport: CodexCleanupReport | null;
  onClean: () => void;
}>;

const BYTE_UNITS: readonly string[] = ["B", "KB", "MB", "GB"];
const BYTES_PER_UNIT = 1024;

type CleanupSummaryValue = readonly [label: string, value: string];

export function createCodexCleanupPanel({
  actionStatus,
  cleanupReport,
  onClean,
}: CodexCleanupPanelProps): HTMLElementTagNameMap["section"] {
  const panel = document.createElement("section");
  panel.className = "CodexCleanupPanel";

  const header = document.createElement("header");
  header.className = "CodexCleanupHeader";

  const title = document.createElement("h2");
  title.className = "SectionTitle";
  title.textContent = "Limpeza";

  const heading = document.createElement("div");
  heading.className = "SectionHeading";

  const description = document.createElement("p");
  description.className = "SectionDescription";
  description.textContent = "Remova dados temporarios e historico local com seguranca.";
  heading.append(title, description);

  const cleanButton = document.createElement("button");
  cleanButton.className = "DangerButton";
  cleanButton.type = "button";
  cleanButton.disabled = actionStatus.state === "Running";
  cleanButton.textContent =
    actionStatus.state === "Running" && actionStatus.label === CODEX_CLEANUP_ACTION_LABEL
      ? "Limpando"
      : "Limpar agora";
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
        ["Conversas", "-"],
        ["Arquivos", "-"],
        ["Pastas", "-"],
        ["Liberado", "-"],
      ]
    : [
        ["Conversas", cleanupReport.removedThreadCount.toString()],
        ["Arquivos", cleanupReport.removedFileCount.toString()],
        ["Pastas", cleanupReport.removedDirectoryCount.toString()],
        ["Liberado", formatBytes(cleanupReport.freedBytes)],
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
    name.textContent = target.name;

    const result = document.createElement("span");
    result.className = "CleanupTargetResult";
    result.textContent = formatTargetResult(target);

    row.append(name, result);
    section.append(row);
  }

  if (section.childElementCount === 0) {
    const empty = document.createElement("p");
    empty.className = "CleanupEmptyState";
    empty.textContent = "Nada removido.";
    section.append(empty);
  }

  return section;
}

function createWarningList(warnings: readonly string[]): HTMLElementTagNameMap["section"] {
  const section = document.createElement("section");
  section.className = "CleanupWarnings";

  for (const warning of warnings) {
    const item = document.createElement("p");
    item.textContent = warning;
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
    parts.push(`${target.removedThreadCount} conversa(s)`);
  }

  if (target.removedGlobalStateReferenceCount > 0) {
    parts.push(`${target.removedGlobalStateReferenceCount} referencia(s)`);
  }

  if (target.removedFileCount > 0) {
    parts.push(`${target.removedFileCount} arquivo(s)`);
  }

  if (target.removedDirectoryCount > 0) {
    parts.push(`${target.removedDirectoryCount} pasta(s)`);
  }

  if (target.freedBytes > 0) {
    parts.push(formatBytes(target.freedBytes));
  }

  return parts.join(" | ");
}

function formatBytes(bytes: number): string {
  let value = bytes;
  let unitIndex = 0;

  while (value >= BYTES_PER_UNIT && unitIndex < BYTE_UNITS.length - 1) {
    value /= BYTES_PER_UNIT;
    unitIndex += 1;
  }

  const fractionDigits = unitIndex === 0 ? 0 : 2;
  return `${value.toFixed(fractionDigits)} ${BYTE_UNITS[unitIndex]}`;
}
