export type CodexCleanupTarget = Readonly<{
  name: string;
  removedThreadCount: number;
  removedGlobalStateReferenceCount: number;
  removedFileCount: number;
  removedDirectoryCount: number;
  freedBytes: number;
}>;

export const CODEX_CLEANUP_ACTION_LABEL = "Limpando Codex";

export type CodexCleanupReport = Readonly<{
  codexHomePath: string;
  removedThreadCount: number;
  removedGlobalStateReferenceCount: number;
  removedFileCount: number;
  removedDirectoryCount: number;
  freedBytes: number;
  targets: readonly CodexCleanupTarget[];
  warnings: readonly string[];
}>;

export function parseCodexCleanupResponse(response: CodexCleanupReport): CodexCleanupReport {
  assertString(response.codexHomePath, "Codex cleanup response is missing the Codex home path.");
  assertCount(response.removedThreadCount, "Codex cleanup response has an invalid thread count.");
  assertCount(
    response.removedGlobalStateReferenceCount,
    "Codex cleanup response has an invalid global state reference count."
  );
  assertCount(response.removedFileCount, "Codex cleanup response has an invalid file count.");
  assertCount(
    response.removedDirectoryCount,
    "Codex cleanup response has an invalid directory count."
  );
  assertCount(response.freedBytes, "Codex cleanup response has an invalid freed byte count.");

  if (!Array.isArray(response.targets)) {
    throw new Error("Codex cleanup response has an invalid target list.");
  }

  if (!Array.isArray(response.warnings) || response.warnings.some((warning) => typeof warning !== "string")) {
    throw new Error("Codex cleanup response has an invalid warning list.");
  }

  response.targets.forEach(parseCodexCleanupTarget);
  return response;
}

function parseCodexCleanupTarget(target: CodexCleanupTarget): void {
  assertString(target.name, "Codex cleanup target is missing its name.");
  assertCount(target.removedThreadCount, "Codex cleanup target has an invalid thread count.");
  assertCount(
    target.removedGlobalStateReferenceCount,
    "Codex cleanup target has an invalid global state reference count."
  );
  assertCount(target.removedFileCount, "Codex cleanup target has an invalid file count.");
  assertCount(
    target.removedDirectoryCount,
    "Codex cleanup target has an invalid directory count."
  );
  assertCount(target.freedBytes, "Codex cleanup target has an invalid freed byte count.");
}

function assertString(value: string, message: string): void {
  if (typeof value !== "string" || value.length === 0) {
    throw new Error(message);
  }
}

function assertCount(value: number, message: string): void {
  if (!Number.isSafeInteger(value) || value < 0) {
    throw new Error(message);
  }
}
