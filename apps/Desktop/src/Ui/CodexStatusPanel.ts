import type { CodexStatus } from "../Domain/CodexInstallation";
import { translate } from "../i18n/catalog";

export type CodexStatusPanelProps = Readonly<{
  status: CodexStatus;
}>;

export function createCodexStatusPanel({
  status,
}: CodexStatusPanelProps): HTMLElementTagNameMap["section"] {
  const panel = document.createElement("section");
  panel.className = `CodexStatusPanel CodexStatusPanel--${status.state}`;

  const content = document.createElement("div");
  content.className = "CodexStatusContent";

  const eyebrow = document.createElement("p");
  eyebrow.className = "PanelEyebrow";
  eyebrow.textContent = translate("status.desktop");

  const title = document.createElement("h2");
  title.className = "CodexStatusTitle";
  title.textContent = createCodexStatusTitle(status);

  const detail = document.createElement("p");
  detail.className = "CodexStatusText";
  detail.textContent = createCodexStatusDetail(status);

  const badge = document.createElement("div");
  badge.className = "CodexStatusBadge";

  const indicator = document.createElement("span");
  indicator.className = "CodexStatusIndicator";
  indicator.setAttribute("aria-hidden", "true");

  const badgeLabel = document.createElement("span");
  badgeLabel.textContent = createCodexStatusBadge(status);

  content.append(eyebrow, title, detail);
  badge.append(indicator, badgeLabel);
  panel.append(content, badge);
  return panel;
}

function createCodexStatusTitle(status: CodexStatus): string {
  switch (status.state) {
    case "Unchecked":
      return translate("status.waitingTitle");
    case "Checking":
      return translate("status.checkingTitle");
    case "Found":
      return translate("status.foundTitle");
    case "NotFound":
      return translate("status.notFoundTitle");
    case "Failed":
      return translate("status.failedTitle");
  }
}

function createCodexStatusDetail(status: CodexStatus): string {
  switch (status.state) {
    case "Unchecked":
      return translate("status.waitingDetail");
    case "Checking":
      return translate("status.checkingDetail");
    case "Found":
      return status.executablePath;
    case "NotFound":
      return translate("status.checkedPaths", { count: status.checkedPaths.length });
    case "Failed":
      return status.message;
  }
}

function createCodexStatusBadge(status: CodexStatus): string {
  switch (status.state) {
    case "Unchecked":
      return translate("status.waitingBadge");
    case "Checking":
      return translate("status.checkingBadge");
    case "Found":
      return translate("status.installedBadge");
    case "NotFound":
      return translate("status.notFoundBadge");
    case "Failed":
      return translate("status.errorBadge");
  }
}
