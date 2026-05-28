export type PersistentPriorityTarget = Readonly<{
  executableName: string;
  installed: boolean;
  priorityValue: number | null;
}>;

export type PersistentPriorityStatus =
  | Readonly<{
      state: "Unchecked";
    }>
  | Readonly<{
      state: "Installed";
      targets: readonly PersistentPriorityTarget[];
    }>
  | Readonly<{
      state: "PartiallyInstalled";
      targets: readonly PersistentPriorityTarget[];
    }>
  | Readonly<{
      state: "NotInstalled";
      targets: readonly PersistentPriorityTarget[];
    }>
  | Readonly<{
      state: "Failed";
      message: string;
    }>;

export type PersistentPriorityTargetResponse = Readonly<{
  executableName: string;
  installed: boolean;
  priorityValue: number | null;
}>;

export type PersistentPriorityStatusResponse = Readonly<{
  installed: boolean;
  targets: readonly PersistentPriorityTargetResponse[];
}>;

export const UNCHECKED_PERSISTENT_PRIORITY_STATUS: PersistentPriorityStatus = {
  state: "Unchecked",
};

export function parsePersistentPriorityStatusResponse(
  response: PersistentPriorityStatusResponse
): PersistentPriorityStatus {
  if (response.targets.length === 0) {
    throw new Error("Persistent priority status response is missing targets.");
  }

  const targets = response.targets.map(parsePersistentPriorityTarget);
  const installedTargetCount = targets.filter((target) => target.installed).length;

  if (response.installed) {
    return {
      state: "Installed",
      targets,
    };
  }

  if (installedTargetCount > 0) {
    return {
      state: "PartiallyInstalled",
      targets,
    };
  }

  return {
    state: "NotInstalled",
    targets,
  };
}

export function createFailedPersistentPriorityStatus(message: string): PersistentPriorityStatus {
  return {
    state: "Failed",
    message,
  };
}

function parsePersistentPriorityTarget(
  response: PersistentPriorityTargetResponse
): PersistentPriorityTarget {
  if (response.executableName.length === 0) {
    throw new Error("Persistent priority target is missing the executable name.");
  }

  if (response.priorityValue !== null && !Number.isInteger(response.priorityValue)) {
    throw new Error(`Persistent priority target ${response.executableName} has an invalid value.`);
  }

  return {
    executableName: response.executableName,
    installed: response.installed,
    priorityValue: response.priorityValue,
  };
}
