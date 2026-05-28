export type CodexLaunchPriority = "High";

export type CodexLaunchResponse = Readonly<{
  executablePath: string;
  priority: CodexLaunchPriority;
  priorityStabilizationStarted: boolean;
}>;

export function parseCodexLaunchResponse(response: CodexLaunchResponse): CodexLaunchResponse {
  if (response.executablePath.length === 0) {
    throw new Error("Codex launch response is missing the executable path.");
  }

  if (response.priority !== "High") {
    throw new Error("Codex launch response has an invalid priority.");
  }

  if (response.priorityStabilizationStarted !== true) {
    throw new Error("Codex launch response did not start priority stabilization.");
  }

  return response;
}
