import { invoke } from "@tauri-apps/api/core";
import type { CommandFailure, CommandResult } from "../Domain/CommandResult";

type BackendCommandFailure = Readonly<{
  code?: unknown;
  message?: unknown;
}>;

const UNEXPECTED_ERROR_CODE = "UnexpectedError";

export async function invokeCommand<Response>(commandName: string): Promise<CommandResult<Response>> {
  try {
    return {
      ok: true,
      value: await invoke<Response>(commandName),
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
