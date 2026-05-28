export type ActionStatus =
  | Readonly<{
      state: "Idle";
    }>
  | Readonly<{
      state: "Running";
      label: string;
    }>
  | Readonly<{
      state: "Succeeded";
      message: string;
    }>
  | Readonly<{
      state: "Failed";
      message: string;
    }>;

export const IDLE_ACTION_STATUS: ActionStatus = {
  state: "Idle",
};

export function createRunningActionStatus(label: string): ActionStatus {
  return {
    state: "Running",
    label,
  };
}

export function createSucceededActionStatus(message: string): ActionStatus {
  return {
    state: "Succeeded",
    message,
  };
}

export function createFailedActionStatus(message: string): ActionStatus {
  return {
    state: "Failed",
    message,
  };
}
