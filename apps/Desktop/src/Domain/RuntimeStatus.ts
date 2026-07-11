import type { CodexStatus } from "./CodexInstallation";

export type RuntimeStatus = "Waiting" | "Opening" | "Ready";

export const WAITING_RUNTIME_STATUS: RuntimeStatus = "Waiting";

export function createWaitingRuntimeStatus(): RuntimeStatus {
  return "Waiting";
}

export function createOpeningRuntimeStatus(): RuntimeStatus {
  return "Opening";
}

export function createRuntimeStatusFromCodexStatus(
  currentStatus: RuntimeStatus,
  codexStatus: CodexStatus
): RuntimeStatus {
  if (codexStatusHasOnlyHighPriorityProcesses(codexStatus)) {
    return "Ready";
  }

  return currentStatus === "Opening" ? "Opening" : "Waiting";
}

export function codexStatusHasProcesses(codexStatus: CodexStatus): boolean {
  return codexStatus.processes.length > 0;
}

export function codexStatusHasOnlyHighPriorityProcesses(codexStatus: CodexStatus): boolean {
  return codexStatus.processes.length > 0 && codexStatus.processes.every((process) => process.priority === "High");
}
