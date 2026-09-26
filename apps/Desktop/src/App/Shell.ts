import type { AppState } from "./AppState";
import type { AppSection } from "../Domain/AppSection";
import { createAppNavigation } from "../Ui/AppNavigation";
import { createBrandIcon } from "../Ui/ApplicationIcons";
import { createCodexActionPanel } from "../Ui/CodexActionPanel";
import { createCodexCleanupPanel } from "../Ui/CodexCleanupPanel";
import { createCodexConsolePanel } from "../Ui/CodexConsolePanel";
import { createCodexStatusPanel } from "../Ui/CodexStatusPanel";
import { createCodexUninstallPanel } from "../Ui/CodexUninstallPanel";
import { createWindowChrome } from "../Ui/WindowChrome";

type ShellProps = Readonly<{
  state: AppState;
  onArmUninstallConfirmation: () => void;
  onCancelUninstallConfirmation: () => void;
  onCleanCodex: () => void;
  onCodexRefresh: () => void;
  onConsoleClear: () => void;
  onConsoleCopy: () => void;
  onOpenCodex: () => void;
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
  onCodexRefresh,
  onConsoleClear,
  onConsoleCopy,
  onOpenCodex,
  onSelectSection,
  onUninstallCodex,
}: ShellProps): HTMLElementTagNameMap["section"] {
  const shell = document.createElement("section");
  shell.className = "AppShell";

  const sidebar = document.createElement("aside");
  sidebar.className = "AppSidebar";
  sidebar.setAttribute("aria-label", "Navegacao do Codex Tools");

  const identity = document.createElement("div");
  identity.className = "AppIdentity";
  identity.setAttribute("aria-label", "Codex Tools");

  const brandMark = document.createElement("span");
  brandMark.className = "AppBrandMark";
  brandMark.append(createBrandIcon());

  const brandName = document.createElement("strong");
  brandName.textContent = "Codex Tools";
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

  workspace.append(
    workspaceHeader,
    createActiveSection({
      state,
      onArmUninstallConfirmation,
      onCancelUninstallConfirmation,
      onCleanCodex,
      onCodexRefresh,
      onConsoleClear,
      onConsoleCopy,
      onOpenCodex,
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
  title.textContent = "Codex Desktop";

  const status = document.createElement("small");
  status.textContent = sidebarStatusLabel(state);

  content.append(title, status);
  card.append(indicator, content);
  return card;
}

function sidebarStatusTone(state: AppState): "Attention" | "Ready" | "Running" | "Waiting" {
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
  if (state.runtimeStatus === "Ready") {
    return "Prioridade alta";
  }

  if (state.runtimeStatus === "Opening") {
    return "Abrindo Codex";
  }

  switch (state.codexStatus.state) {
    case "Checking":
      return "Verificando instalacao";
    case "Found":
      return "Codex instalado";
    case "NotFound":
      return "Codex nao encontrado";
    case "Failed":
      return "Atencao necessaria";
    case "Unchecked":
      return "Aguardando verificacao";
  }
}

function sectionTitle(section: AppSection): string {
  switch (section) {
    case "Processes":
      return "Processos e prioridade";
    case "Cleanup":
      return "Limpeza";
    case "Uninstall":
      return "Desinstalacao";
  }
}

function createActiveSection({
  state,
  onArmUninstallConfirmation,
  onCancelUninstallConfirmation,
  onCleanCodex,
  onCodexRefresh,
  onConsoleClear,
  onConsoleCopy,
  onOpenCodex,
  onUninstallCodex,
}: Omit<ShellProps, "onSelectSection">): HTMLElementTagNameMap["section"] {
  switch (state.activeSection) {
    case "Processes":
      return createProcessesSection({
        state,
        onCodexRefresh,
        onConsoleClear,
        onConsoleCopy,
        onOpenCodex,
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
  }
}

function createProcessesSection({
  state,
  onCodexRefresh,
  onConsoleClear,
  onConsoleCopy,
  onOpenCodex,
}: Pick<
  ShellProps,
  | "state"
  | "onCodexRefresh"
  | "onConsoleClear"
  | "onConsoleCopy"
  | "onOpenCodex"
>): HTMLElementTagNameMap["section"] {
  const panel = createControlSurface();

  panel.append(
    createCodexStatusPanel({
      status: state.codexStatus,
    }),
    createCodexActionPanel({
      actionStatus: state.actionStatus,
      codexStatus: state.codexStatus,
      onRefresh: onCodexRefresh,
      onOpenCodex,
      runtimeStatus: state.runtimeStatus,
    }),
    createCodexConsolePanel({
      messages: state.consoleMessages,
      onClear: onConsoleClear,
      onCopy: onConsoleCopy,
    })
  );

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
