export type ProcessPriority = "Normal" | "High";

export type ProcessPriorityOption = Readonly<{
  value: ProcessPriority;
  label: string;
}>;

export const PROCESS_PRIORITY_OPTIONS = [
  {
    value: "Normal",
    label: "Normal",
  },
  {
    value: "High",
    label: "Alta",
  },
] as const satisfies readonly ProcessPriorityOption[];

export type SelectedProcessPriority = ProcessPriority | null;

export function parseProcessPriority(value: string): ProcessPriority {
  const option = PROCESS_PRIORITY_OPTIONS.find((priorityOption) => priorityOption.value === value);

  if (option === undefined) {
    throw new Error(`Unsupported process priority: ${value}`);
  }

  return option.value;
}

export function formatProcessPriority(priority: ProcessPriority): string {
  const option = PROCESS_PRIORITY_OPTIONS.find((priorityOption) => priorityOption.value === priority);

  if (option === undefined) {
    throw new Error(`Unsupported process priority: ${priority}`);
  }

  return option.label;
}
