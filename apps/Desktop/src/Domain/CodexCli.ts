export type CodexCliStatus =
  | Readonly<{ state: "Unchecked" }>
  | Readonly<{ state: "Checking" }>
  | Readonly<{ state: "Found"; executablePath: string; checkedPaths: readonly string[] }>
  | Readonly<{ state: "NotFound"; checkedPaths: readonly string[] }>
  | Readonly<{ state: "Failed"; message: string }>;

export type CodexCliStatusResponse = Readonly<{
  found: boolean;
  executablePath: string | null;
  checkedPaths: readonly string[];
}>;

export const UNCHECKED_CODEX_CLI_STATUS: CodexCliStatus = { state: "Unchecked" };
export const CHECKING_CODEX_CLI_STATUS: CodexCliStatus = { state: "Checking" };

export function parseCodexCliStatusResponse(response: CodexCliStatusResponse): CodexCliStatus {
  if (!Array.isArray(response.checkedPaths) || !response.checkedPaths.every(isNonEmptyString)) {
    throw new Error("Codex CLI status response has an invalid checked path list.");
  }
  if (typeof response.found !== "boolean") {
    throw new Error("Codex CLI status response has an invalid found value.");
  }
  if (!response.found) {
    if (response.executablePath !== null) {
      throw new Error("Codex CLI status response has an unexpected executable path.");
    }
    return { state: "NotFound", checkedPaths: response.checkedPaths };
  }
  if (!isNonEmptyString(response.executablePath)) {
    throw new Error("Codex CLI status response is missing the executable path.");
  }
  return {
    state: "Found",
    executablePath: response.executablePath,
    checkedPaths: response.checkedPaths,
  };
}

function isNonEmptyString(value: unknown): value is string {
  return typeof value === "string" && value.length > 0;
}
