import { invokeCommand } from "./CommandInvoker";
import {
  parsePersistentPriorityStatusResponse,
  type PersistentPriorityStatus,
  type PersistentPriorityStatusResponse,
} from "../Domain/PersistentPriority";
import type { CommandResult } from "../Domain/CommandResult";

const GET_PERSISTENT_PRIORITY_STATUS_COMMAND_NAME = "get_persistent_priority_status";
const INSTALL_PERSISTENT_HIGH_PRIORITY_COMMAND_NAME = "install_persistent_high_priority";
const REMOVE_PERSISTENT_HIGH_PRIORITY_COMMAND_NAME = "remove_persistent_high_priority";

export async function getPersistentPriorityStatus(): Promise<CommandResult<PersistentPriorityStatus>> {
  return invokePersistentPriorityStatusCommand(GET_PERSISTENT_PRIORITY_STATUS_COMMAND_NAME);
}

export async function installPersistentHighPriority(): Promise<CommandResult<PersistentPriorityStatus>> {
  return invokePersistentPriorityStatusCommand(INSTALL_PERSISTENT_HIGH_PRIORITY_COMMAND_NAME);
}

export async function removePersistentHighPriority(): Promise<CommandResult<PersistentPriorityStatus>> {
  return invokePersistentPriorityStatusCommand(REMOVE_PERSISTENT_HIGH_PRIORITY_COMMAND_NAME);
}

async function invokePersistentPriorityStatusCommand(
  commandName: string
): Promise<CommandResult<PersistentPriorityStatus>> {
  const result = await invokeCommand<PersistentPriorityStatusResponse>(commandName);

  if (!result.ok) {
    return result;
  }

  try {
    return {
      ok: true,
      value: parsePersistentPriorityStatusResponse(result.value),
    };
  } catch (error: unknown) {
    return {
      ok: false,
      error: {
        code: "InvalidResponse",
        message: error instanceof Error ? error.message : "Invalid persistent priority status response.",
      },
    };
  }
}
