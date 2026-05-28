import { invokeCommand } from "./CommandInvoker";
import {
  parseAutomationStatusResponse,
  type AutomationStatus,
  type AutomationStatusResponse,
} from "../Domain/Automation";
import type { CommandResult } from "../Domain/CommandResult";
import type { ProcessPriority } from "../Domain/ProcessPriority";

const GET_AUTOMATION_STATUS_COMMAND_NAME = "get_automation_status";
const INSTALL_CODEX_AUTOMATION_COMMAND_NAME = "install_codex_automation";
const REMOVE_CODEX_AUTOMATION_COMMAND_NAME = "remove_codex_automation";

export async function getAutomationStatus(): Promise<CommandResult<AutomationStatus>> {
  return invokeAutomationStatusCommand(GET_AUTOMATION_STATUS_COMMAND_NAME);
}

export async function installCodexAutomation(priority: ProcessPriority): Promise<CommandResult<AutomationStatus>> {
  return invokeAutomationStatusCommand(INSTALL_CODEX_AUTOMATION_COMMAND_NAME, {
    request: {
      priority,
    },
  });
}

export async function removeCodexAutomation(): Promise<CommandResult<AutomationStatus>> {
  return invokeAutomationStatusCommand(REMOVE_CODEX_AUTOMATION_COMMAND_NAME);
}

async function invokeAutomationStatusCommand(
  commandName: string,
  commandArguments?: Readonly<Record<string, unknown>>
): Promise<CommandResult<AutomationStatus>> {
  const result = await invokeCommand<AutomationStatusResponse>(commandName, commandArguments);

  if (!result.ok) {
    return result;
  }

  try {
    return {
      ok: true,
      value: parseAutomationStatusResponse(result.value),
    };
  } catch (error: unknown) {
    return {
      ok: false,
      error: {
        code: "InvalidResponse",
        message: error instanceof Error ? error.message : "Invalid automation status response.",
      },
    };
  }
}
