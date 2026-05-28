export type CodexStatus =
  | Readonly<{
      state: "Unchecked";
    }>
  | Readonly<{
      state: "Checking";
    }>
  | Readonly<{
      state: "Found";
      executablePath: string;
      checkedPaths: readonly string[];
    }>
  | Readonly<{
      state: "NotFound";
      checkedPaths: readonly string[];
    }>
  | Readonly<{
      state: "Failed";
      message: string;
    }>;

export type CodexStatusResponse = Readonly<{
  found: boolean;
  executablePath: string | null;
  checkedPaths: readonly string[];
}>;

export const UNCHECKED_CODEX_STATUS: CodexStatus = {
  state: "Unchecked",
};

export const CHECKING_CODEX_STATUS: CodexStatus = {
  state: "Checking",
};

export function parseCodexStatusResponse(response: CodexStatusResponse): CodexStatus {
  if (!response.found) {
    return {
      state: "NotFound",
      checkedPaths: response.checkedPaths,
    };
  }

  if (response.executablePath === null || response.executablePath.length === 0) {
    throw new Error("Codex status response is missing the executable path.");
  }

  return {
    state: "Found",
    executablePath: response.executablePath,
    checkedPaths: response.checkedPaths,
  };
}

export function createFailedCodexStatus(message: string): CodexStatus {
  return {
    state: "Failed",
    message,
  };
}
