import {
  PROCESS_PRIORITY_OPTIONS,
  parseProcessPriority,
  type ProcessPriority,
  type SelectedProcessPriority,
} from "../Domain/ProcessPriority";

export type PrioritySelectorProps = Readonly<{
  selectedPriority: SelectedProcessPriority;
  onChange: (priority: ProcessPriority) => void;
}>;

const PROCESS_PRIORITY_FIELD_NAME = "ProcessPriority";

export function createPrioritySelector({
  selectedPriority,
  onChange,
}: PrioritySelectorProps): HTMLFieldSetElement {
  const fieldset = document.createElement("fieldset");
  fieldset.className = "PrioritySelector";

  const legend = document.createElement("legend");
  legend.className = "FieldLabel";
  legend.textContent = "Prioridade";

  const segmentedControl = document.createElement("div");
  segmentedControl.className = "SegmentedControl";

  for (const priorityOption of PROCESS_PRIORITY_OPTIONS) {
    const input = document.createElement("input");
    input.id = `${PROCESS_PRIORITY_FIELD_NAME}${priorityOption.value}`;
    input.className = "SegmentedInput";
    input.type = "radio";
    input.name = PROCESS_PRIORITY_FIELD_NAME;
    input.value = priorityOption.value;
    input.checked = selectedPriority === priorityOption.value;
    input.addEventListener("change", () => {
      if (input.checked) {
        onChange(parseProcessPriority(input.value));
      }
    });

    const label = document.createElement("label");
    label.className = "SegmentedOption";
    label.htmlFor = input.id;
    label.textContent = priorityOption.label;

    segmentedControl.append(input, label);
  }

  fieldset.append(legend, segmentedControl);

  return fieldset;
}
