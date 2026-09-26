import {
  IDLE_PRIORITY_STABILIZATION,
  parsePriorityStabilizationResponse,
  type PriorityStabilization,
  type PriorityStabilizationResponse,
} from "./PriorityStabilization";

export type CodexProcessPriority =
  | "Idle"
  | "BelowNormal"
  | "Normal"
  | "AboveNormal"
  | "High"
  | "Realtime"
  | "Unknown";

export type CodexProcessElevation = "Elevated" | "NotElevated" | "Unavailable";

export type CodexProcess = Readonly<{
  processId: number;
  processName: string;
  executablePath: string | null;
  priority: CodexProcessPriority;
  elevation: CodexProcessElevation;
}>;

export type CodexStatus =
  | Readonly<{
      state: "Unchecked";
      processes: readonly CodexProcess[];
      priorityStabilization: PriorityStabilization;
    }>
  | Readonly<{
      state: "Checking";
      processes: readonly CodexProcess[];
      priorityStabilization: PriorityStabilization;
    }>
  | Readonly<{
      state: "Found";
      executablePath: string;
      checkedPaths: readonly string[];
      processes: readonly CodexProcess[];
      priorityStabilization: PriorityStabilization;
      packageAllowsElevation: boolean | null;
      elevationDiagnostic: string | null;
    }>
  | Readonly<{
      state: "NotFound";
      checkedPaths: readonly string[];
      processes: readonly CodexProcess[];
      priorityStabilization: PriorityStabilization;
    }>
  | Readonly<{
      state: "Failed";
      message: string;
      processes: readonly CodexProcess[];
      priorityStabilization: PriorityStabilization;
    }>;

export type CodexStatusResponse = Readonly<{
  found: boolean;
  executablePath: string | null;
  checkedPaths: readonly string[];
  processes: readonly CodexProcessResponse[];
  priorityStabilization: PriorityStabilizationResponse;
  packageAllowsElevation: boolean | null;
  elevationDiagnostic: string | null;
}>;

export type CodexProcessResponse = Readonly<{
  processId: number;
  processName: string;
  executablePath: string | null;
  priority: CodexProcessPriority;
  elevation: CodexProcessElevation;
}>;

export const UNCHECKED_CODEX_STATUS: CodexStatus = {
  state: "Unchecked",
  processes: [],
  priorityStabilization: IDLE_PRIORITY_STABILIZATION,
};

export const CHECKING_CODEX_STATUS: CodexStatus = {
  state: "Checking",
  processes: [],
  priorityStabilization: IDLE_PRIORITY_STABILIZATION,
};

export function parseCodexStatusResponse(response: CodexStatusResponse): CodexStatus {
  const processes = parseCodexProcesses(response.processes);
  const priorityStabilization = parsePriorityStabilizationResponse(
    response.priorityStabilization
  );

  if (!response.found) {
    return {
      state: "NotFound",
      checkedPaths: response.checkedPaths,
      processes,
      priorityStabilization,
    };
  }

  if (response.executablePath === null || response.executablePath.length === 0) {
    throw new Error("Codex status response is missing the executable path.");
  }
  if (
    response.packageAllowsElevation !== null &&
    typeof response.packageAllowsElevation !== "boolean"
  ) {
    throw new Error("Codex status response has an invalid elevation capability.");
  }
  if (
    response.elevationDiagnostic !== null &&
    (typeof response.elevationDiagnostic !== "string" || response.elevationDiagnostic.length === 0)
  ) {
    throw new Error("Codex status response has an invalid elevation diagnostic.");
  }
  if (response.packageAllowsElevation === null && response.elevationDiagnostic === null) {
    throw new Error("Codex status response is missing the elevation capability result.");
  }

  return {
    state: "Found",
    executablePath: response.executablePath,
    checkedPaths: response.checkedPaths,
    processes,
    priorityStabilization,
    packageAllowsElevation: response.packageAllowsElevation,
    elevationDiagnostic: response.elevationDiagnostic,
  };
}

export function createFailedCodexStatus(message: string): CodexStatus {
  return {
    state: "Failed",
    message,
    processes: [],
    priorityStabilization: IDLE_PRIORITY_STABILIZATION,
  };
}

export function codexStatusesAreEqual(left: CodexStatus, right: CodexStatus): boolean {
  if (
    left.state !== right.state ||
    !codexProcessesAreEqual(left.processes, right.processes) ||
    !priorityStabilizationsAreEqual(
      left.priorityStabilization,
      right.priorityStabilization
    )
  ) {
    return false;
  }

  switch (left.state) {
    case "Unchecked":
      return right.state === "Unchecked";
    case "Checking":
      return right.state === "Checking";
    case "Found":
      return (
        right.state === "Found" &&
        left.executablePath === right.executablePath &&
        left.packageAllowsElevation === right.packageAllowsElevation &&
        left.elevationDiagnostic === right.elevationDiagnostic &&
        stringArraysAreEqual(left.checkedPaths, right.checkedPaths)
      );
    case "NotFound":
      return (
        right.state === "NotFound" &&
        stringArraysAreEqual(left.checkedPaths, right.checkedPaths)
      );
    case "Failed":
      return right.state === "Failed" && left.message === right.message;
  }
}

function parseCodexProcesses(processes: readonly CodexProcessResponse[]): readonly CodexProcess[] {
  if (!Array.isArray(processes)) {
    throw new Error("Codex status response has an invalid process list.");
  }

  return processes.map(parseCodexProcess);
}

function parseCodexProcess(process: CodexProcessResponse): CodexProcess {
  if (!Number.isInteger(process.processId) || process.processId <= 0) {
    throw new Error("Codex process response has an invalid process id.");
  }

  if (process.processName.length === 0) {
    throw new Error("Codex process response is missing the process name.");
  }

  if (
    process.executablePath !== null &&
    (typeof process.executablePath !== "string" || process.executablePath.length === 0)
  ) {
    throw new Error("Codex process response has an invalid executable path.");
  }

  if (!isCodexProcessPriority(process.priority)) {
    throw new Error(`Codex process response has an invalid priority: ${process.priority}`);
  }

  if (!isCodexProcessElevation(process.elevation)) {
    throw new Error(`Codex process response has an invalid elevation: ${process.elevation}`);
  }

  return process;
}

function codexProcessesAreEqual(
  left: readonly CodexProcess[],
  right: readonly CodexProcess[]
): boolean {
  return (
    left.length === right.length &&
    left.every((process, index) => {
      const otherProcess = right[index];
      return (
        otherProcess !== undefined &&
        process.processId === otherProcess.processId &&
        process.processName === otherProcess.processName &&
        process.executablePath === otherProcess.executablePath &&
        process.priority === otherProcess.priority &&
        process.elevation === otherProcess.elevation
      );
    })
  );
}

function priorityStabilizationsAreEqual(
  left: PriorityStabilization,
  right: PriorityStabilization
): boolean {
  return (
    left.state === right.state &&
    left.message === right.message &&
    left.attempts === right.attempts &&
    numberArraysAreEqual(left.updatedProcessIds, right.updatedProcessIds)
  );
}

function stringArraysAreEqual(
  left: readonly string[],
  right: readonly string[]
): boolean {
  return left.length === right.length && left.every((value, index) => value === right[index]);
}

function numberArraysAreEqual(
  left: readonly number[],
  right: readonly number[]
): boolean {
  return left.length === right.length && left.every((value, index) => value === right[index]);
}

function isCodexProcessPriority(value: string): value is CodexProcessPriority {
  return (
    value === "Idle" ||
    value === "BelowNormal" ||
    value === "Normal" ||
    value === "AboveNormal" ||
    value === "High" ||
    value === "Realtime" ||
    value === "Unknown"
  );
}

function isCodexProcessElevation(value: string): value is CodexProcessElevation {
  return value === "Elevated" || value === "NotElevated" || value === "Unavailable";
}
