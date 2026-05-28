export type CodexProcessPriority = "Normal" | "High" | "Other" | "Unknown";

export type CodexProcessElevation = "Elevated" | "NotElevated" | "Unavailable";

export type CodexProcess = Readonly<{
  processId: number;
  processName: string;
  priority: CodexProcessPriority;
  elevation: CodexProcessElevation;
}>;

export type CodexStatus =
  | Readonly<{
      state: "Unchecked";
      processes: readonly CodexProcess[];
    }>
  | Readonly<{
      state: "Checking";
      processes: readonly CodexProcess[];
    }>
  | Readonly<{
      state: "Found";
      executablePath: string;
      checkedPaths: readonly string[];
      processes: readonly CodexProcess[];
    }>
  | Readonly<{
      state: "NotFound";
      checkedPaths: readonly string[];
      processes: readonly CodexProcess[];
    }>
  | Readonly<{
      state: "Failed";
      message: string;
      processes: readonly CodexProcess[];
    }>;

export type CodexStatusResponse = Readonly<{
  found: boolean;
  executablePath: string | null;
  checkedPaths: readonly string[];
  processes: readonly CodexProcessResponse[];
}>;

export type CodexProcessResponse = Readonly<{
  processId: number;
  processName: string;
  priority: CodexProcessPriority;
  elevation: CodexProcessElevation;
}>;

export const UNCHECKED_CODEX_STATUS: CodexStatus = {
  state: "Unchecked",
  processes: [],
};

export const CHECKING_CODEX_STATUS: CodexStatus = {
  state: "Checking",
  processes: [],
};

export function parseCodexStatusResponse(response: CodexStatusResponse): CodexStatus {
  const processes = parseCodexProcesses(response.processes);

  if (!response.found) {
    return {
      state: "NotFound",
      checkedPaths: response.checkedPaths,
      processes,
    };
  }

  if (response.executablePath === null || response.executablePath.length === 0) {
    throw new Error("Codex status response is missing the executable path.");
  }

  return {
    state: "Found",
    executablePath: response.executablePath,
    checkedPaths: response.checkedPaths,
    processes,
  };
}

export function createFailedCodexStatus(message: string): CodexStatus {
  return {
    state: "Failed",
    message,
    processes: [],
  };
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

  if (!isCodexProcessPriority(process.priority)) {
    throw new Error(`Codex process response has an invalid priority: ${process.priority}`);
  }

  if (!isCodexProcessElevation(process.elevation)) {
    throw new Error(`Codex process response has an invalid elevation: ${process.elevation}`);
  }

  return process;
}

function isCodexProcessPriority(value: string): value is CodexProcessPriority {
  return value === "Normal" || value === "High" || value === "Other" || value === "Unknown";
}

function isCodexProcessElevation(value: string): value is CodexProcessElevation {
  return value === "Elevated" || value === "NotElevated" || value === "Unavailable";
}
