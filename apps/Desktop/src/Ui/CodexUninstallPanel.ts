import type { ActionStatus } from "../Domain/ActionStatus";
import {
  CODEX_UNINSTALL_ACTION_LABEL,
  type CodexUninstallReport,
  type CodexUninstallTarget,
} from "../Domain/CodexUninstall";

export type CodexUninstallPanelProps = Readonly<{
  actionStatus: ActionStatus;
  confirmationArmed: boolean;
  uninstallReport: CodexUninstallReport | null;
  onArmConfirmation: () => void;
  onCancelConfirmation: () => void;
  onUninstall: () => void;
}>;

const BYTE_UNITS: readonly string[] = ["B", "KB", "MB", "GB"];
const BYTES_PER_UNIT = 1024;

type UninstallSummaryValue = readonly [label: string, value: string];

export function createCodexUninstallPanel({
  actionStatus,
  confirmationArmed,
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
  title.textContent = "Desinstalacao";

  const heading = document.createElement("div");
  heading.className = "SectionHeading";

  const description = document.createElement("p");
  description.className = "SectionDescription";
  description.textContent =
    "Remove o pacote Codex e apaga dados locais, cache, configuracoes e residuais relacionados.";
  heading.append(title, description);

  const actions = document.createElement("div");
  actions.className = "UninstallActions";

  if (confirmationArmed) {
    const cancelButton = document.createElement("button");
    cancelButton.className = "SecondaryButton";
    cancelButton.type = "button";
    cancelButton.disabled = actionStatus.state === "Running";
    cancelButton.textContent = "Cancelar";
    cancelButton.addEventListener("click", onCancelConfirmation);

    const confirmButton = document.createElement("button");
    confirmButton.className = "DangerButton";
    confirmButton.type = "button";
    confirmButton.disabled = actionStatus.state === "Running";
    confirmButton.textContent =
      actionStatus.state === "Running" && actionStatus.label === CODEX_UNINSTALL_ACTION_LABEL
        ? "Desinstalando"
        : "Confirmar exclusao total";
    confirmButton.addEventListener("click", onUninstall);

    actions.append(cancelButton, confirmButton);
  } else {
    const armButton = document.createElement("button");
    armButton.className = "DangerButton";
    armButton.type = "button";
    armButton.disabled = actionStatus.state === "Running";
    armButton.textContent = "Desinstalar e apagar tudo";
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
    ? "Confirmacao necessaria"
    : "Escopo da desinstalacao";

  const body = document.createElement("p");
  body.textContent = confirmationArmed
    ? "Esta acao e irreversivel. Feche o Codex e o ChatGPT Desktop antes de continuar."
    : "Serao removidos home .codex, LocalAppData OpenAI\\Codex, dados MSIX, ProgramData, cache de runtimes, Documents\\Codex, tarefa elevada, AppCompat RUNASADMIN e o pacote OpenAI.Codex.";

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
          ["Arquivos", "-"],
          ["Pastas", "-"],
          ["Pacote", "-"],
          ["Liberado", "-"],
        ]
      : [
          ["Arquivos", uninstallReport.removedFileCount.toString()],
          ["Pastas", uninstallReport.removedDirectoryCount.toString()],
          ["Pacote", uninstallReport.packageRemoved ? "Removido" : "Ausente"],
          ["Liberado", formatBytes(uninstallReport.freedBytes)],
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
    empty.textContent = "Nenhum alvo processado.";
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

function formatTargetResult(target: CodexUninstallTarget): string {
  const parts: string[] = [];

  if (target.removedFileCount > 0) {
    parts.push(`${target.removedFileCount} arquivo(s)`);
  }

  if (target.removedDirectoryCount > 0) {
    parts.push(`${target.removedDirectoryCount} pasta(s)`);
  }

  if (target.freedBytes > 0) {
    parts.push(formatBytes(target.freedBytes));
  }

  if (parts.length === 0) {
    return target.details;
  }

  return `${parts.join(" | ")} · ${target.details}`;
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
