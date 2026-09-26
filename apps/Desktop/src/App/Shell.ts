import type { AppState } from "./AppState";
import type { AppSection } from "../Domain/AppSection";
import type { CodexLaunchTarget } from "../Domain/CodexLaunchTarget";
import { createAppNavigation } from "../Ui/AppNavigation";
import { createBrandIcon } from "../Ui/ApplicationIcons";
import { createCodexLaunchPanel } from "../Ui/CodexLaunchPanel";
import { createCodexCleanupPanel } from "../Ui/CodexCleanupPanel";
import { createCodexConsolePanel } from "../Ui/CodexConsolePanel";
import { createCodexStatusPanel } from "../Ui/CodexStatusPanel";
import { createCodexUninstallPanel } from "../Ui/CodexUninstallPanel";
import { createSettingsPanel } from "../Ui/SettingsPanel";
import { createWindowChrome } from "../Ui/WindowChrome";
import { translate } from "../i18n/catalog";
import type { LocalePreference } from "../i18n/locale-selection";

type ShellProps = Readonly<{
  state: AppState;
  onArmUninstallConfirmation: () => void;
  onCancelUninstallConfirmation: () => void;
  onCleanCodex: () => void;
  onRefreshSelected: () => void;
  onConsoleClear: () => void;
  onConsoleCopy: () => void;
  onOpenSelected: () => void;
  onSelectLaunchTarget: (target: CodexLaunchTarget) => void;
  onSelectLocale: (preference: LocalePreference) => void;
  onSelectSection: (section: AppSection) => void;
  onUninstallCodex: () => void;
}>;

type TextElementTagName = "h1" | "h2" | "p";

function createTextElement<TagName extends TextElementTagName>(
  tagName: TagName,
  className: string,
  text: string
): HTMLElementTagNameMap[TagName] {
  const element = document.createElement(tagName);
  element.className = className;
  element.textContent = text;
  return element;
}

export function createShell({
  state,
  onArmUninstallConfirmation,
  onCancelUninstallConfirmation,
  onCleanCodex,
  onRefreshSelected,
  onConsoleClear,
  onConsoleCopy,
  onOpenSelected,
  onSelectLaunchTarget,
  onSelectLocale,
  onSelectSection,
  onUninstallCodex,
}: ShellProps): HTMLElementTagNameMap["section"] {
  const shell = document.createElement("section");
  shell.className = "AppShell";

  const sidebar = document.createElement("aside");
  sidebar.className = "AppSidebar";
  sidebar.setAttribute("aria-label", translate("app.sidebarLabel"));

  const identity = document.createElement("div");
  identity.className = "AppIdentity";
  identity.setAttribute("aria-label", "Codex Tools");

  const brandMark = document.createElement("span");
  brandMark.className = "AppBrandMark";
  brandMark.append(createBrandIcon());

  const brandName = document.createElement("strong");
  brandName.textContent = translate("sidebar.title");
  identity.append(brandMark, brandName);

  const sidebarFooter = document.createElement("div");
  sidebarFooter.className = "SidebarFooter";
  sidebarFooter.append(createSidebarRuntimeCard(state));

  sidebar.append(
    identity,
    createAppNavigation({
      activeSection: state.activeSection,
      onSelectSection,
    }),
    sidebarFooter
  );

  const workspace = document.createElement("main");
  workspace.className = "AppWorkspace";
  workspace.setAttribute("aria-label", sectionTitle(state.activeSection));

  const workspaceHeader = document.createElement("header");
  workspaceHeader.className = "WorkspaceHeader";
  workspaceHeader.append(
    createTextElement("h1", "WorkspaceTitle", sectionTitle(state.activeSection))
  );

  workspace.append(workspaceHeader);
  if (!state.nativeRuntimeAvailable && (
    state.activeSection === "Cleanup" || state.activeSection === "Uninstall"
  )) {
    const previewNotice = document.createElement("p");
    previewNotice.className = "PreviewNotice";
    previewNotice.textContent = translate("common.browserPreview");
    workspace.append(previewNotice);
  }
  workspace.append(
    createActiveSection({
      state,
      onArmUninstallConfirmation,
      onCancelUninstallConfirmation,
      onCleanCodex,
      onRefreshSelected,
      onConsoleClear,
      onConsoleCopy,
      onOpenSelected,
      onSelectLaunchTarget,
      onSelectLocale,
      onUninstallCodex,
    })
  );

  shell.append(createWindowChrome(), sidebar, workspace);
  return shell;
}

function createSidebarRuntimeCard(state: AppState): HTMLDivElement {
  const card = document.createElement("div");
  card.className = `SidebarRuntime SidebarRuntime--${sidebarStatusTone(state)}`;

  const indicator = document.createElement("span");
  indicator.className = "SidebarRuntimeIndicator";
  indicator.setAttribute("aria-hidden", "true");

  const content = document.createElement("span");
  content.className = "SidebarRuntimeContent";

  const title = document.createElement("strong");
  title.textContent = translate("sidebar.title");

  const status = document.createElement("small");
  status.textContent = sidebarStatusLabel(state);

  content.append(title, status);
  card.append(indicator, content);
  return card;
}

function sidebarStatusTone(state: AppState): "Attention" | "Ready" | "Running" | "Waiting" {
  if (!state.nativeRuntimeAvailable) {
    return "Waiting";
  }
  if (isOpeningCodexCli(state)) {
    return "Running";
  }

  if (state.runtimeStatus === "Ready") {
    return "Ready";
  }

  if (state.runtimeStatus === "Opening" || state.codexStatus.state === "Checking") {
    return "Running";
  }

  if (state.codexStatus.state === "Failed" || state.codexStatus.state === "NotFound") {
    return "Attention";
  }

  return "Waiting";
}

function sidebarStatusLabel(state: AppState): string {
  if (!state.nativeRuntimeAvailable) {
    return translate("sidebar.status.browserPreview");
  }
  if (isOpeningCodexCli(state)) {
    return translate("sidebar.status.openingCli");
  }

  if (state.runtimeStatus === "Ready") {
    return translate("sidebar.status.highPriority");
  }

  if (state.runtimeStatus === "Opening") {
    return translate("sidebar.status.opening");
  }

  switch (state.codexStatus.state) {
    case "Checking":
      return translate("sidebar.status.checking");
    case "Found":
      return translate("sidebar.status.installed");
    case "NotFound":
      return translate("sidebar.status.notFound");
    case "Failed":
      return translate("sidebar.status.attention");
    case "Unchecked":
      return translate("sidebar.status.waiting");
  }
}

function isOpeningCodexCli(state: AppState): boolean {
  return state.actionStatus.state === "Running" &&
    state.actionStatus.label === "cli.opening";
}

function sectionTitle(section: AppSection): string {
  switch (section) {
    case "Processes":
      return translate("section.processes");
    case "Cleanup":
      return translate("section.cleanup");
    case "Uninstall":
      return translate("section.uninstall");
    case "Settings":
      return translate("section.settings");
  }
}

function createActiveSection({
  state,
  onArmUninstallConfirmation,
  onCancelUninstallConfirmation,
  onCleanCodex,
  onRefreshSelected,
  onConsoleClear,
  onConsoleCopy,
  onOpenSelected,
  onSelectLaunchTarget,
  onSelectLocale,
  onUninstallCodex,
}: Omit<ShellProps, "onSelectSection">): HTMLElementTagNameMap["section"] {
  switch (state.activeSection) {
    case "Processes":
      return createProcessesSection({
        state,
        onRefreshSelected,
        onConsoleClear,
        onConsoleCopy,
        onOpenSelected,
        onSelectLaunchTarget,
      });
    case "Cleanup":
      return createCleanupSection({
        state,
        onCleanCodex,
      });
    case "Uninstall":
      return createUninstallSection({
        state,
        onArmUninstallConfirmation,
        onCancelUninstallConfirmation,
        onUninstallCodex,
      });
    case "Settings":
      return createSettingsSection({ onSelectLocale });
  }
}

function createProcessesSection({
  state,
  onRefreshSelected,
  onConsoleClear,
  onConsoleCopy,
  onOpenSelected,
  onSelectLaunchTarget,
}: Pick<
  ShellProps,
  | "state"
  | "onRefreshSelected"
  | "onConsoleClear"
  | "onConsoleCopy"
  | "onOpenSelected"
  | "onSelectLaunchTarget"
>): HTMLElementTagNameMap["section"] {
  const panel = createControlSurface();

  panel.append(
    createCodexStatusPanel({
      nativeRuntimeAvailable: state.nativeRuntimeAvailable,
      status: state.selectedLaunchTarget === "desktop" ? state.codexStatus : state.codexCliStatus,
      target: state.selectedLaunchTarget,
    }),
    createCodexLaunchPanel({
      actionStatus: state.actionStatus,
      codexStatus: state.codexStatus,
      cliStatus: state.codexCliStatus,
      nativeRuntimeAvailable: state.nativeRuntimeAvailable,
      onRefresh: onRefreshSelected,
      onOpen: onOpenSelected,
      onSelectTarget: onSelectLaunchTarget,
      runtimeStatus: state.runtimeStatus,
      selectedTarget: state.selectedLaunchTarget,
    }),
    createCodexConsolePanel({
      messages: state.consoleMessages,
      onClear: onConsoleClear,
      onCopy: onConsoleCopy,
    })
  );

  return panel;
}

function createSettingsSection({
  onSelectLocale,
}: Pick<ShellProps, "onSelectLocale">): HTMLElementTagNameMap["section"] {
  const panel = createControlSurface();
  panel.append(createSettingsPanel(onSelectLocale));
  return panel;
}

function createCleanupSection({
  state,
  onCleanCodex,
}: Pick<ShellProps, "state" | "onCleanCodex">): HTMLElementTagNameMap["section"] {
  const panel = createControlSurface();

  panel.append(
    createCodexCleanupPanel({
      actionStatus: state.actionStatus,
      cleanupReport: state.cleanupReport,
      nativeRuntimeAvailable: state.nativeRuntimeAvailable,
      onClean: onCleanCodex,
    })
  );

  return panel;
}

function createUninstallSection({
  state,
  onArmUninstallConfirmation,
  onCancelUninstallConfirmation,
  onUninstallCodex,
}: Pick<
  ShellProps,
  | "state"
  | "onArmUninstallConfirmation"
  | "onCancelUninstallConfirmation"
  | "onUninstallCodex"
>): HTMLElementTagNameMap["section"] {
  const panel = createControlSurface();

  panel.append(
    createCodexUninstallPanel({
      actionStatus: state.actionStatus,
      confirmationArmed: state.uninstallConfirmationArmed,
      nativeRuntimeAvailable: state.nativeRuntimeAvailable,
      uninstallReport: state.uninstallReport,
      onArmConfirmation: onArmUninstallConfirmation,
      onCancelConfirmation: onCancelUninstallConfirmation,
      onUninstall: onUninstallCodex,
    })
  );

  return panel;
}

function createControlSurface(): HTMLElementTagNameMap["section"] {
  const panel = document.createElement("section");
  panel.className = "ControlSurface";
  return panel;
}
