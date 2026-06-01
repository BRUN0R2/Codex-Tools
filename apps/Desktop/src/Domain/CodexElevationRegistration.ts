export type CodexElevationRegistrationResponse = Readonly<{
  registeredExecutablePaths: readonly string[];
  scannedDirectories: readonly string[];
}>;

export function parseCodexElevationRegistrationResponse(
  response: CodexElevationRegistrationResponse
): CodexElevationRegistrationResponse {
  if (!Array.isArray(response.registeredExecutablePaths)) {
    throw new Error("Codex elevation registration response has an invalid executable list.");
  }

  if (!Array.isArray(response.scannedDirectories)) {
    throw new Error("Codex elevation registration response has an invalid scanned directory list.");
  }

  return {
    registeredExecutablePaths: response.registeredExecutablePaths.map(parsePath),
    scannedDirectories: response.scannedDirectories.map(parsePath),
  };
}

function parsePath(path: string): string {
  if (typeof path !== "string" || path.length === 0) {
    throw new Error("Codex elevation registration response contains an invalid path.");
  }

  return path;
}
