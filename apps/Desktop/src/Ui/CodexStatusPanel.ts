import type { CodexCliStatus } from "../Domain/CodexCli";
import type { CodexStatus } from "../Domain/CodexInstallation";
import type { CodexLaunchTarget } from "../Domain/CodexLaunchTarget";
import { translate } from "../i18n/catalog";

export type CodexStatusPanelProps = Readonly<{
  nativeRuntimeAvailable: boolean;
  status: CodexStatus | CodexCliStatus;
  target: CodexLaunchTarget;
}>;

export function createCodexStatusPanel({
  nativeRuntimeAvailable,
  status,
  target,
}: CodexStatusPanelProps): HTMLElementTagNameMap["section"] {
  const panel = document.createElement("section");
  panel.className = `CodexStatusPanel CodexStatusPanel--${nativeRuntimeAvailable ? status.state : "Preview"}`;

  const content = document.createElement("div");
  content.className = "CodexStatusContent";

  const eyebrow = document.createElement("p");
  eyebrow.className = "PanelEyebrow";
  eyebrow.textContent = translate(target === "desktop" ? "status.desktop" : "status.cli");

  const title = document.createElement("h2");
  title.className = "CodexStatusTitle";
  title.textContent = nativeRuntimeAvailable
    ? createCodexStatusTitle(status)
    : translate("status.previewTitle");

  const detail = document.createElement("p");
  detail.className = "CodexStatusText";
  detail.textContent = nativeRuntimeAvailable
    ? createCodexStatusDetail(status, target)
    : translate("status.previewDetail");

  const badge = document.createElement("div");
  badge.className = "CodexStatusBadge";

  const indicator = document.createElement("span");
  indicator.className = "CodexStatusIndicator";
  indicator.setAttribute("aria-hidden", "true");

  const badgeLabel = document.createElement("span");
  badgeLabel.textContent = nativeRuntimeAvailable
    ? createCodexStatusBadge(status)
    : translate("status.previewBadge");

  content.append(eyebrow, title, detail);
  badge.append(indicator, badgeLabel);
  panel.append(content, badge);
  return panel;
}

function createCodexStatusTitle(status: CodexStatus | CodexCliStatus): string {
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

function createCodexStatusDetail(
  status: CodexStatus | CodexCliStatus,
  target: CodexLaunchTarget,
): string {
  switch (status.state) {
    case "Unchecked":
      return translate("status.waitingDetail");
    case "Checking":
      return translate("status.checkingDetail");
    case "Found":
      return status.executablePath;
    case "NotFound":
      return target === "cli"
        ? translate("cli.notFound")
        : translate("status.checkedPaths", { count: status.checkedPaths.length });
    case "Failed":
      return status.message;
  }
}

function createCodexStatusBadge(status: CodexStatus | CodexCliStatus): string {
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
