import type {
  CodexProcess,
  CodexProcessElevation,
  CodexProcessPriority,
  CodexStatus,
} from "./CodexInstallation";

export type ConsoleMessage = string;

export function createCodexConsoleMessages(status: CodexStatus): readonly ConsoleMessage[] {
  switch (status.state) {
    case "Unchecked":
      return ["Codex nao verificado."];
    case "Checking":
      return ["Verificando Codex."];
    case "Found":
    case "NotFound":
      return createProcessMessages(status.processes);
    case "Failed":
      return [`Erro: ${status.message}`];
  }
}

function createProcessMessages(processes: readonly CodexProcess[]): readonly ConsoleMessage[] {
  if (processes.length === 0) {
    return ["Nenhum processo Codex em execucao."];
  }

  return processes.map(formatProcessMessage);
}

function formatProcessMessage(process: CodexProcess): ConsoleMessage {
  const elevation = formatElevation(process.elevation);
  const priority = formatPriority(process.priority);

  return `Processo: ${process.processName} | PID: ${process.processId} | Elevação: ${elevation} | Prioridade: ${priority}`;
}

function formatElevation(elevation: CodexProcessElevation): string {
  switch (elevation) {
    case "Elevated":
      return "administrador";
    case "NotElevated":
      return "normal";
    case "Unavailable":
      return "indisponivel";
  }
}

function formatPriority(priority: CodexProcessPriority): string {
  switch (priority) {
    case "Normal":
      return "normal";
    case "High":
      return "alta";
    case "Other":
      return "outra";
    case "Unknown":
      return "indisponivel";
  }
}
