import { invoke, isTauri } from "@tauri-apps/api/core";
import type { CommandFailure, CommandResult } from "../Domain/CommandResult";

type BackendCommandFailure = Readonly<{
  code?: unknown;
  message?: unknown;
}>;

type CommandArguments = Readonly<Record<string, unknown>>;

const UNEXPECTED_ERROR_CODE = "UnexpectedError";
const NATIVE_RUNTIME_UNAVAILABLE_CODE = "nativeRuntimeUnavailable";

export async function invokeCommand<Response>(
  commandName: string,
  commandArguments?: CommandArguments
): Promise<CommandResult<Response>> {
  if (!isTauri()) {
    return {
      ok: false,
      error: {
        code: NATIVE_RUNTIME_UNAVAILABLE_CODE,
        message: "The Tauri runtime is unavailable in this browser.",
      },
    };
  }

  try {
    return {
      ok: true,
      value:
        commandArguments === undefined
          ? await invoke<Response>(commandName)
          : await invoke<Response>(commandName, commandArguments),
    };
  } catch (error: unknown) {
    return {
      ok: false,
      error: normalizeCommandFailure(error),
    };
  }
}

function normalizeCommandFailure(error: unknown): CommandFailure {
  if (isBackendCommandFailure(error)) {
    return {
      code: typeof error.code === "string" ? error.code : UNEXPECTED_ERROR_CODE,
      message: typeof error.message === "string" ? error.message : "Backend command failed.",
    };
  }

  if (error instanceof Error) {
    return {
      code: UNEXPECTED_ERROR_CODE,
      message: error.message,
    };
  }

  return {
    code: UNEXPECTED_ERROR_CODE,
    message: "Backend command failed with an unknown error.",
  };
}

function isBackendCommandFailure(error: unknown): error is BackendCommandFailure {
  return typeof error === "object" && error !== null;
}
