export type CodexLaunchPriority = "High";

export type CodexLaunchResponse = Readonly<{
  executablePath: string;
  priority: CodexLaunchPriority;
  updatedProcessCount: number;
}>;

export function parseCodexLaunchResponse(response: CodexLaunchResponse): CodexLaunchResponse {
  if (response.executablePath.length === 0) {
    throw new Error("Codex launch response is missing the executable path.");
  }

  if (response.priority !== "High") {
    throw new Error("Codex launch response has an invalid priority.");
  }

  if (!Number.isInteger(response.updatedProcessCount) || response.updatedProcessCount < 0) {
    throw new Error("Codex launch response has an invalid process count.");
  }

  return response;
}
