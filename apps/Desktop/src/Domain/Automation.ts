export type AutomationStatus =
  | Readonly<{
      state: "Unchecked";
    }>
  | Readonly<{
      state: "Installed";
      executablePath: string;
      shortcutPath: string;
      taskName: string;
    }>
  | Readonly<{
      state: "NotInstalled";
      executablePath: string;
      shortcutPath: string;
      taskName: string;
    }>
  | Readonly<{
      state: "Failed";
      message: string;
    }>;

export type AutomationStatusResponse = Readonly<{
  installed: boolean;
  executablePath: string;
  shortcutPath: string;
  taskName: string;
}>;

export const UNCHECKED_AUTOMATION_STATUS: AutomationStatus = {
  state: "Unchecked",
};

export function parseAutomationStatusResponse(response: AutomationStatusResponse): AutomationStatus {
  if (response.executablePath.length === 0) {
    throw new Error("Automation status response is missing the executable path.");
  }

  if (response.shortcutPath.length === 0) {
    throw new Error("Automation status response is missing the shortcut path.");
  }

  if (response.taskName.length === 0) {
    throw new Error("Automation status response is missing the task name.");
  }

  return {
    state: response.installed ? "Installed" : "NotInstalled",
    executablePath: response.executablePath,
    shortcutPath: response.shortcutPath,
    taskName: response.taskName,
  };
}

export function createFailedAutomationStatus(message: string): AutomationStatus {
  return {
    state: "Failed",
    message,
  };
}
