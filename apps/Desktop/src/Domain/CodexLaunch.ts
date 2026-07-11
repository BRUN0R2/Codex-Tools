export type CodexLaunchPriority = "High";
export type CodexLaunchMethod = "ElevatedScheduledTask" | "ShellExecuteRunAs";

export type CodexLaunchResponse = Readonly<{
  executablePath: string;
  launchMethod: CodexLaunchMethod;
  appServerElevationObserved: boolean;
  fallbackUsed: boolean;
  diagnosticMessage: string | null;
  priority: CodexLaunchPriority;
  priorityStabilizationStarted: boolean;
}>;

export function parseCodexLaunchResponse(response: CodexLaunchResponse): CodexLaunchResponse {
  if (response.executablePath.length === 0) {
    throw new Error("Codex launch response is missing the executable path.");
  }

  if (!isCodexLaunchMethod(response.launchMethod)) {
    throw new Error(`Codex launch response has an invalid launch method: ${response.launchMethod}`);
  }

  if (typeof response.appServerElevationObserved !== "boolean") {
    throw new Error("Codex launch response has an invalid app-server elevation flag.");
  }

  if (typeof response.fallbackUsed !== "boolean") {
    throw new Error("Codex launch response has an invalid fallback flag.");
  }

  if (
    response.diagnosticMessage !== null &&
    typeof response.diagnosticMessage !== "string"
  ) {
    throw new Error("Codex launch response has an invalid diagnostic message.");
  }

  if (response.priority !== "High") {
    throw new Error("Codex launch response has an invalid priority.");
  }

  if (response.priorityStabilizationStarted !== true) {
    throw new Error("Codex launch response did not start priority stabilization.");
  }

  return response;
}

function isCodexLaunchMethod(value: string): value is CodexLaunchMethod {
  return value === "ElevatedScheduledTask" || value === "ShellExecuteRunAs";
}
