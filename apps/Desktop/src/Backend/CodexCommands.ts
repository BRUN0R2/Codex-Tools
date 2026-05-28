import { invokeCommand } from "./CommandInvoker";
import type { CommandResult } from "../Domain/CommandResult";
import {
  parseCodexStatusResponse,
  type CodexStatus,
  type CodexStatusResponse,
} from "../Domain/CodexInstallation";

const GET_CODEX_STATUS_COMMAND_NAME = "get_codex_status";

export async function getCodexStatus(): Promise<CommandResult<CodexStatus>> {
  const result = await invokeCommand<CodexStatusResponse>(GET_CODEX_STATUS_COMMAND_NAME);

  if (!result.ok) {
    return result;
  }

  try {
    return {
      ok: true,
      value: parseCodexStatusResponse(result.value),
    };
  } catch (error: unknown) {
    return {
      ok: false,
      error: {
        code: "InvalidResponse",
        message: error instanceof Error ? error.message : "Invalid Codex status response.",
      },
    };
  }
}
