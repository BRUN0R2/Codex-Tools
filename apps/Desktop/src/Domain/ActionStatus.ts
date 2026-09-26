import type { TranslationKey } from "../i18n/catalog";

export type ActionStatus =
  | Readonly<{
      state: "Idle";
    }>
  | Readonly<{
      state: "Running";
      label: TranslationKey;
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

export function createRunningActionStatus(label: TranslationKey): ActionStatus {
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
