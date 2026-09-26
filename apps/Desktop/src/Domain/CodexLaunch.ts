export type CodexLaunchPriority = "High";

export type CodexLaunchResponse = Readonly<{
  executablePath: string;
  processId: number;
  priority: CodexLaunchPriority;
  priorityStabilizationStarted: boolean;
}>;

export function parseCodexLaunchResponse(response: CodexLaunchResponse): CodexLaunchResponse {
  if (response.executablePath.length === 0) {
    throw new Error("Codex launch response is missing the executable path.");
  }

  if (!Number.isInteger(response.processId) || response.processId <= 0) {
    throw new Error("Codex launch response has an invalid process id.");
  }

  if (response.priority !== "High") {
    throw new Error("Codex launch response has an invalid priority.");
  }

  if (response.priorityStabilizationStarted !== true) {
    throw new Error("Codex launch response did not start priority stabilization.");
  }

  return response;
}
