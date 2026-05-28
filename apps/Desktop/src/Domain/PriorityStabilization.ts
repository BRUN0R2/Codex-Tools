export type PriorityStabilizationState = "Idle" | "Running" | "Succeeded" | "Failed";

export type PriorityStabilization = Readonly<{
  state: PriorityStabilizationState;
  message: string | null;
  updatedProcessIds: readonly number[];
  attempts: number;
}>;

export type PriorityStabilizationResponse = Readonly<{
  state: PriorityStabilizationState;
  message: string | null;
  updatedProcessIds: readonly number[];
  attempts: number;
}>;

export const IDLE_PRIORITY_STABILIZATION: PriorityStabilization = {
  state: "Idle",
  message: null,
  updatedProcessIds: [],
  attempts: 0,
};

export function parsePriorityStabilizationResponse(
  response: PriorityStabilizationResponse
): PriorityStabilization {
  if (!isPriorityStabilizationState(response.state)) {
    throw new Error(`Priority stabilization response has an invalid state: ${response.state}`);
  }

  if (response.message !== null && response.message.length === 0) {
    throw new Error("Priority stabilization response has an empty message.");
  }

  if (!Number.isInteger(response.attempts) || response.attempts < 0) {
    throw new Error("Priority stabilization response has an invalid attempt count.");
  }

  if (!Array.isArray(response.updatedProcessIds)) {
    throw new Error("Priority stabilization response has an invalid process id list.");
  }

  if (
    response.updatedProcessIds.some(
      (processId) => !Number.isInteger(processId) || processId <= 0
    )
  ) {
    throw new Error("Priority stabilization response has an invalid process id.");
  }

  return response;
}

export function priorityStabilizationHasFinished(
  priorityStabilization: PriorityStabilization
): boolean {
  return priorityStabilization.state === "Succeeded" || priorityStabilization.state === "Failed";
}

export function priorityStabilizationHasFailed(
  priorityStabilization: PriorityStabilization
): boolean {
  return priorityStabilization.state === "Failed";
}

function isPriorityStabilizationState(value: string): value is PriorityStabilizationState {
  return value === "Idle" || value === "Running" || value === "Succeeded" || value === "Failed";
}
