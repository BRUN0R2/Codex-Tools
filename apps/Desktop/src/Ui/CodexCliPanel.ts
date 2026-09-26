import type { ActionStatus } from "../Domain/ActionStatus";
import type { CodexCliStatus } from "../Domain/CodexCli";
import { translate } from "../i18n/catalog";
import { createLaunchIcon, createRefreshIcon } from "./ApplicationIcons";

export type CodexCliPanelProps = Readonly<{
  actionStatus: ActionStatus;
  status: CodexCliStatus;
  onOpen: () => void;
  onRefresh: () => void;
}>;

export function createCodexCliPanel({
  actionStatus,
  status,
  onOpen,
  onRefresh,
}: CodexCliPanelProps): HTMLElementTagNameMap["section"] {
  const panel = document.createElement("section");
  panel.className = "Panel CodexActionPanel";

  const header = document.createElement("header");
  header.className = "PanelHeader CodexActionHeader";

  const heading = document.createElement("div");
  heading.className = "SectionHeading";

  const title = document.createElement("h2");
  title.className = "SectionTitle";
  title.textContent = translate("status.cli");

  const description = document.createElement("p");
  description.className = "SectionDescription";
  description.textContent = translate("cli.description");
  heading.append(title, description);

  const actions = document.createElement("div");
  actions.className = "CodexActionButtons";
  const refresh = createButton(
    "SecondaryButton",
    status.state === "Checking" ? translate("action.checking") : translate("action.refresh"),
    createRefreshIcon(),
    onRefresh,
  );
  refresh.disabled = status.state === "Checking";

  const opening = actionStatus.state === "Running" && actionStatus.label === translate("cli.opening");
  const open = createButton(
    "PrimaryButton",
    opening ? translate("cli.opening") : translate("cli.open"),
    createLaunchIcon(),
    onOpen,
  );
  open.disabled = status.state !== "Found" || actionStatus.state === "Running";
  if (opening) {
    open.classList.add("IsBusy");
    open.setAttribute("aria-busy", "true");
  }
  actions.append(refresh, open);
  header.append(heading, actions);

  const detail = document.createElement("p");
  detail.className = "CodexStatusText";
  detail.textContent = cliStatusDetail(status);
  panel.append(header, detail);
  return panel;
}

function cliStatusDetail(status: CodexCliStatus): string {
  switch (status.state) {
    case "Unchecked":
      return translate("status.waitingDetail");
    case "Checking":
      return translate("status.checkingDetail");
    case "Found":
      return status.executablePath;
    case "NotFound":
      return translate("cli.notFound");
    case "Failed":
      return status.message;
  }
}

function createButton(
  className: string,
  label: string,
  icon: SVGSVGElement,
  onClick: () => void,
): HTMLButtonElement {
  const button = document.createElement("button");
  button.className = className;
  button.type = "button";
  const labelElement = document.createElement("span");
  labelElement.textContent = label;
  button.append(icon, labelElement);
  button.addEventListener("click", onClick);
  return button;
}
