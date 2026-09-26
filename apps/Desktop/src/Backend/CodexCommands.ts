import { invokeCommand } from "./CommandInvoker";
import type { CommandResult } from "../Domain/CommandResult";
import {
  parseCodexLaunchResponse,
  type CodexLaunchResponse,
} from "../Domain/CodexLaunch";
import {
  parseCodexCleanupResponse,
  type CodexCleanupReport,
} from "../Domain/CodexCleanup";
import {
  parseCodexUninstallResponse,
  type CodexUninstallReport,
} from "../Domain/CodexUninstall";
import {
  parseCodexStatusResponse,
  type CodexStatus,
  type CodexStatusResponse,
} from "../Domain/CodexInstallation";
import {
  parseCodexCliStatusResponse,
  type CodexCliStatus,
  type CodexCliStatusResponse,
} from "../Domain/CodexCli";

const CLEAN_CODEX_WORKSPACE_COMMAND_NAME = "clean_codex_workspace";
const UNINSTALL_CODEX_PRODUCT_COMMAND_NAME = "uninstall_codex_product";
const GET_CODEX_STATUS_COMMAND_NAME = "get_codex_status";
const OPEN_CODEX_COMMAND_NAME = "open_codex";
const GET_CODEX_CLI_STATUS_COMMAND_NAME = "get_codex_cli_status";
const OPEN_CODEX_CLI_COMMAND_NAME = "open_codex_cli";

export async function getCodexCliStatus(): Promise<CommandResult<CodexCliStatus>> {
  const result = await invokeCommand<CodexCliStatusResponse>(GET_CODEX_CLI_STATUS_COMMAND_NAME);
  if (!result.ok) return result;

  try {
    return { ok: true, value: parseCodexCliStatusResponse(result.value) };
  } catch (error: unknown) {
    return {
      ok: false,
      error: {
        code: "InvalidResponse",
        message: error instanceof Error ? error.message : "Invalid Codex CLI status response.",
      },
    };
  }
}

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

export async function cleanCodexWorkspace(): Promise<CommandResult<CodexCleanupReport>> {
  const result = await invokeCommand<CodexCleanupReport>(CLEAN_CODEX_WORKSPACE_COMMAND_NAME);

  if (!result.ok) {
    return result;
  }

  try {
    return {
      ok: true,
      value: parseCodexCleanupResponse(result.value),
    };
  } catch (error: unknown) {
    return {
      ok: false,
      error: {
        code: "InvalidResponse",
        message:
          error instanceof Error ? error.message : "Invalid Codex cleanup response.",
      },
    };
  }
}

export async function uninstallCodexProduct(): Promise<CommandResult<CodexUninstallReport>> {
  const result = await invokeCommand<CodexUninstallReport>(UNINSTALL_CODEX_PRODUCT_COMMAND_NAME);

  if (!result.ok) {
    return result;
  }

  try {
    return {
      ok: true,
      value: parseCodexUninstallResponse(result.value),
    };
  } catch (error: unknown) {
    return {
      ok: false,
      error: {
        code: "InvalidResponse",
        message:
          error instanceof Error ? error.message : "Invalid Codex uninstall response.",
      },
    };
  }
}

export async function openCodex(): Promise<CommandResult<CodexLaunchResponse>> {
  const result = await invokeCommand<CodexLaunchResponse>(OPEN_CODEX_COMMAND_NAME);

  if (!result.ok) {
    return result;
  }

  try {
    return {
      ok: true,
      value: parseCodexLaunchResponse(result.value),
    };
  } catch (error: unknown) {
    return {
      ok: false,
      error: {
        code: "InvalidResponse",
        message: error instanceof Error ? error.message : "Invalid Codex launch response.",
      },
    };
  }
}

export async function openCodexCli(): Promise<CommandResult<CodexLaunchResponse>> {
  const result = await invokeCommand<CodexLaunchResponse>(OPEN_CODEX_CLI_COMMAND_NAME);
  if (!result.ok) return result;

  try {
    return { ok: true, value: parseCodexLaunchResponse(result.value) };
  } catch (error: unknown) {
    return {
      ok: false,
      error: {
        code: "InvalidResponse",
        message: error instanceof Error ? error.message : "Invalid Codex CLI launch response.",
      },
    };
  }
}
