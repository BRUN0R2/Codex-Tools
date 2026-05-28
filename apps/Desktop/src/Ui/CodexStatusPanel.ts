import type {
  CodexProcess,
  CodexProcessElevation,
  CodexProcessPriority,
  CodexStatus,
} from "../Domain/CodexInstallation";

export type CodexStatusPanelProps = Readonly<{
  status: CodexStatus;
  onRefresh: () => void;
}>;

export function createCodexStatusPanel({
  status,
  onRefresh,
}: CodexStatusPanelProps): HTMLElementTagNameMap["section"] {
  const panel = document.createElement("section");
  panel.className = "CodexStatusPanel";

  const detectionRow = document.createElement("div");
  detectionRow.className = "CodexDetectionRow";

  const statusText = document.createElement("p");
  statusText.className = "CodexStatusText";
  statusText.textContent = createCodexStatusText(status);

  const button = document.createElement("button");
  button.className = "PrimaryButton";
  button.type = "button";
  button.disabled = status.state === "Checking";
  button.textContent = status.state === "Checking" ? "Verificando" : "Verificar Codex";
  button.addEventListener("click", onRefresh);

  detectionRow.append(statusText, button);
  panel.append(detectionRow, createProcessReport(status.processes));

  return panel;
}

function createCodexStatusText(status: CodexStatus): string {
  switch (status.state) {
    case "Unchecked":
      return "Codex nao verificado";
    case "Checking":
      return "Verificando Codex";
    case "Found":
      return status.executablePath;
    case "NotFound":
      return `${status.checkedPaths.length} caminhos verificados`;
    case "Failed":
      return status.message;
  }
}

function createProcessReport(processes: readonly CodexProcess[]): HTMLElementTagNameMap["section"] {
  const report = document.createElement("section");
  report.className = "CodexProcessReport";

  report.append(
    createProcessGroup(
      "Processos com prioridade alta:",
      processes.filter((process) => process.priority === "High"),
      "Nenhum processo Codex em prioridade alta."
    )
  );

  const remainingProcesses = processes.filter((process) => process.priority !== "High");

  if (remainingProcesses.length > 0) {
    report.append(
      createProcessGroup(
        "Processos Codex fora da prioridade alta:",
        remainingProcesses,
        "Nenhum processo Codex fora da prioridade alta."
      )
    );
  }

  return report;
}

function createProcessGroup(
  title: string,
  processes: readonly CodexProcess[],
  emptyMessage: string
): HTMLElementTagNameMap["section"] {
  const group = document.createElement("section");
  group.className = "CodexProcessGroup";

  const heading = document.createElement("p");
  heading.className = "CodexProcessGroupTitle";
  heading.textContent = title;
  group.append(heading);

  if (processes.length === 0) {
    const emptyText = document.createElement("p");
    emptyText.className = "CodexProcessEmptyText";
    emptyText.textContent = emptyMessage;
    group.append(emptyText);
    return group;
  }

  const list = document.createElement("ul");
  list.className = "CodexProcessList";

  for (const process of processes) {
    const item = document.createElement("li");
    item.className = "CodexProcessItem";
    item.textContent = formatCodexProcess(process);
    list.append(item);
  }

  group.append(list);
  return group;
}

function formatCodexProcess(process: CodexProcess): string {
  const elevation = formatElevation(process.elevation);
  const priority = formatPriority(process.priority);

  return `${process.processName} (PID ${process.processId}) - ${elevation} - ${priority}`;
}

function formatPriority(priority: CodexProcessPriority): string {
  switch (priority) {
    case "Normal":
      return "Prioridade normal";
    case "High":
      return "Prioridade alta";
    case "Other":
      return "Outra prioridade";
    case "Unknown":
      return "Prioridade indisponivel";
  }
}

function formatElevation(elevation: CodexProcessElevation): string {
  switch (elevation) {
    case "Elevated":
      return "Administrador";
    case "NotElevated":
      return "Sem administrador";
    case "Unavailable":
      return "Administrador indisponivel";
  }
}
