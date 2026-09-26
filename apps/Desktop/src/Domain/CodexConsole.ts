import type {
  CodexProcess,
  CodexProcessElevation,
  CodexProcessPriority,
  CodexStatus,
} from "./CodexInstallation";
import type { PriorityStabilization } from "./PriorityStabilization";
import { translate } from "../i18n/catalog";

export type ConsoleMessage = string;

export function createCodexConsoleMessages(status: CodexStatus): readonly ConsoleMessage[] {
  switch (status.state) {
    case "Unchecked":
      return [translate("console.unchecked")];
    case "Checking":
      return [translate("console.checking")];
    case "Found":
      return appendPriorityStabilizationMessages(
        [...createProcessMessages(status.processes), createElevationCapabilityMessage(status)],
        status.priorityStabilization
      );
    case "NotFound":
      return appendPriorityStabilizationMessages(
        createProcessMessages(status.processes),
        status.priorityStabilization
      );
    case "Failed":
      return appendPriorityStabilizationMessages(
        [translate("action.errorPrefix", { message: status.message })],
        status.priorityStabilization
      );
  }
}

function createElevationCapabilityMessage(
  status: Extract<CodexStatus, { state: "Found" }>
): ConsoleMessage {
  if (status.packageAllowsElevation === false) {
    return translate("console.elevationCapability.denied");
  }
  if (status.packageAllowsElevation === true) {
    return translate("console.elevationCapability.allowed");
  }
  return translate("console.elevationCapability.failed", {
    message: status.elevationDiagnostic ?? translate("common.unknownError"),
  });
}

function createProcessMessages(processes: readonly CodexProcess[]): readonly ConsoleMessage[] {
  if (processes.length === 0) {
    return [translate("console.processNone")];
  }

  return processes.map(formatProcessMessage);
}

function formatProcessMessage(process: CodexProcess): ConsoleMessage {
  const elevation = formatElevation(process.elevation);
  const priority = formatPriority(process.priority);
  const executablePath = process.executablePath === null
    ? ""
    : translate("console.path", { path: process.executablePath });

  return translate("console.process", {
    name: process.processName,
    pid: process.processId,
    elevation,
    priority,
    path: executablePath,
  });
}

function formatElevation(elevation: CodexProcessElevation): string {
  switch (elevation) {
    case "Elevated":
      return translate("console.elevation.admin");
    case "NotElevated":
      return translate("console.elevation.normal");
    case "Unavailable":
      return translate("console.elevation.unavailable");
  }
}

function formatPriority(priority: CodexProcessPriority): string {
  switch (priority) {
    case "Idle":
      return translate("console.priority.idle");
    case "BelowNormal":
      return translate("console.priority.belowNormal");
    case "Normal":
      return translate("console.priority.normal");
    case "AboveNormal":
      return translate("console.priority.aboveNormal");
    case "High":
      return translate("console.priority.high");
    case "Realtime":
      return translate("console.priority.realtime");
    case "Unknown":
      return translate("console.priority.unknown");
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
      return [translate("console.priority.running")];
    case "Succeeded":
      return [createPrioritySucceededMessage(priorityStabilization)];
    case "Failed":
      return [
        translate("console.priority.failed", {
          message: priorityStabilization.message ?? translate("common.unknownError"),
        }),
      ];
  }
}

function createPrioritySucceededMessage(
  priorityStabilization: PriorityStabilization
): ConsoleMessage {
  if (priorityStabilization.updatedProcessIds.length === 0) {
    return translate("console.priority.succeeded", {
      attempts: priorityStabilization.attempts,
      processes: "",
    });
  }

  return translate("console.priority.succeeded", {
    attempts: priorityStabilization.attempts,
    processes: translate("console.priority.processes", {
      ids: priorityStabilization.updatedProcessIds.join(", "),
    }),
  });
}
