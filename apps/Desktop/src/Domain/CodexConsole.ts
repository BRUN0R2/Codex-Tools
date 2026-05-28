import type {
  CodexProcess,
  CodexProcessElevation,
  CodexProcessPriority,
  CodexStatus,
} from "./CodexInstallation";
import type { PriorityStabilization } from "./PriorityStabilization";

export type ConsoleMessage = string;

export function createCodexConsoleMessages(status: CodexStatus): readonly ConsoleMessage[] {
  switch (status.state) {
    case "Unchecked":
      return ["Codex nao verificado."];
    case "Checking":
      return ["Verificando Codex."];
    case "Found":
    case "NotFound":
      return appendPriorityStabilizationMessages(
        createProcessMessages(status.processes),
        status.priorityStabilization
      );
    case "Failed":
      return appendPriorityStabilizationMessages(
        [`Erro: ${status.message}`],
        status.priorityStabilization
      );
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
    case "Idle":
      return "baixa";
    case "BelowNormal":
      return "abaixo do normal";
    case "Normal":
      return "normal";
    case "AboveNormal":
      return "acima do normal";
    case "High":
      return "alta";
    case "Realtime":
      return "tempo real";
    case "Unknown":
      return "indisponivel";
  }
}

function appendPriorityStabilizationMessages(
  messages: readonly ConsoleMessage[],
  priorityStabilization: PriorityStabilization
): readonly ConsoleMessage[] {
  const priorityMessages = createPriorityStabilizationMessages(priorityStabilization);
  return priorityMessages.length === 0 ? messages : [...messages, ...priorityMessages];
}

function createPriorityStabilizationMessages(
  priorityStabilization: PriorityStabilization
): readonly ConsoleMessage[] {
  switch (priorityStabilization.state) {
    case "Idle":
      return [];
    case "Running":
      return ["Estabilizando prioridade alta do Codex."];
    case "Succeeded":
      return [createPrioritySucceededMessage(priorityStabilization)];
    case "Failed":
      return [
        `Erro de prioridade: ${priorityStabilization.message ?? "falha desconhecida."}`,
      ];
  }
}

function createPrioritySucceededMessage(
  priorityStabilization: PriorityStabilization
): ConsoleMessage {
  if (priorityStabilization.updatedProcessIds.length === 0) {
    return `Prioridade alta estabilizada em ${priorityStabilization.attempts} tentativas.`;
  }

  return `Prioridade alta estabilizada em ${priorityStabilization.attempts} tentativas | PIDs: ${priorityStabilization.updatedProcessIds.join(", ")}`;
}
