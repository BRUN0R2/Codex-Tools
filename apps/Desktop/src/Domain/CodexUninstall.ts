export type CodexUninstallTarget = Readonly<{
  name: string;
  removedFileCount: number;
  removedDirectoryCount: number;
  freedBytes: number;
  details: string;
}>;

export const CODEX_UNINSTALL_ACTION_LABEL = "Desinstalando Codex";

export type CodexUninstallReport = Readonly<{
  removedFileCount: number;
  removedDirectoryCount: number;
  freedBytes: number;
  packageRemoved: boolean;
  targets: readonly CodexUninstallTarget[];
  warnings: readonly string[];
}>;

export function parseCodexUninstallResponse(
  response: CodexUninstallReport
): CodexUninstallReport {
  assertCount(response.removedFileCount, "Codex uninstall response has an invalid file count.");
  assertCount(
    response.removedDirectoryCount,
    "Codex uninstall response has an invalid directory count."
  );
  assertCount(response.freedBytes, "Codex uninstall response has an invalid freed byte count.");

  if (typeof response.packageRemoved !== "boolean") {
    throw new Error("Codex uninstall response has an invalid packageRemoved flag.");
  }

  if (!Array.isArray(response.targets)) {
    throw new Error("Codex uninstall response has an invalid target list.");
  }

  if (
    !Array.isArray(response.warnings) ||
    response.warnings.some((warning) => typeof warning !== "string")
  ) {
    throw new Error("Codex uninstall response has an invalid warning list.");
  }

  response.targets.forEach(parseCodexUninstallTarget);
  return response;
}

function parseCodexUninstallTarget(target: CodexUninstallTarget): void {
  assertString(target.name, "Codex uninstall target is missing its name.");
  assertString(target.details, "Codex uninstall target is missing details.");
  assertCount(target.removedFileCount, "Codex uninstall target has an invalid file count.");
  assertCount(
    target.removedDirectoryCount,
    "Codex uninstall target has an invalid directory count."
  );
  assertCount(target.freedBytes, "Codex uninstall target has an invalid freed byte count.");
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
