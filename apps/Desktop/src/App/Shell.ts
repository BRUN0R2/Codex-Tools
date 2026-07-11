import type { AppState } from "./AppState";
import type { AppSection } from "../Domain/AppSection";
import { createAppNavigation } from "../Ui/AppNavigation";
import { createCodexActionPanel } from "../Ui/CodexActionPanel";
import { createCodexCleanupPanel } from "../Ui/CodexCleanupPanel";
import { createCodexConsolePanel } from "../Ui/CodexConsolePanel";
import { createCodexStatusPanel } from "../Ui/CodexStatusPanel";

type ShellProps = Readonly<{
  state: AppState;
  onCleanCodex: () => void;
  onCodexRefresh: () => void;
  onConsoleClear: () => void;
  onConsoleCopy: () => void;
  onOpenCodex: () => void;
  onRegisterRunAsAdministrator: () => void;
  onSelectSection: (section: AppSection) => void;
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
  onCleanCodex,
  onCodexRefresh,
  onConsoleClear,
  onConsoleCopy,
  onOpenCodex,
  onRegisterRunAsAdministrator,
  onSelectSection,
}: ShellProps): HTMLElementTagNameMap["section"] {
  const shell = document.createElement("section");
  shell.className = "AppShell";

  const header = document.createElement("header");
  header.className = "AppHeader";

  const heading = document.createElement("div");
  heading.className = "AppIdentity";

  const brandMark = document.createElement("span");
  brandMark.className = "AppBrandMark";
  brandMark.setAttribute("aria-hidden", "true");
  brandMark.textContent = "C";

  const headingContent = document.createElement("div");
  headingContent.append(
    createTextElement("p", "AppEyebrow", "Windows utility"),
    createTextElement("h1", "AppTitle", "Codex Tools")
  );
  heading.append(brandMark, headingContent);

  const workspace = document.createElement("section");
  workspace.className = "AppWorkspace";
  workspace.append(
    createAppNavigation({
      activeSection: state.activeSection,
      onSelectSection,
    }),
    createActiveSection({
      state,
      onCleanCodex,
      onCodexRefresh,
      onConsoleClear,
      onConsoleCopy,
      onOpenCodex,
      onRegisterRunAsAdministrator,
    })
  );

  header.append(heading);
  shell.append(header, workspace);

  return shell;
}

function createActiveSection({
  state,
  onCleanCodex,
  onCodexRefresh,
  onConsoleClear,
  onConsoleCopy,
  onOpenCodex,
  onRegisterRunAsAdministrator,
}: Omit<ShellProps, "onSelectSection">): HTMLElementTagNameMap["section"] {
  switch (state.activeSection) {
    case "Admin":
      return createAdminSection({
        state,
        onCodexRefresh,
        onConsoleClear,
        onConsoleCopy,
        onOpenCodex,
        onRegisterRunAsAdministrator,
      });
    case "Cleanup":
      return createCleanupSection({
        state,
        onCleanCodex,
      });
  }
}

function createAdminSection({
  state,
  onCodexRefresh,
  onConsoleClear,
  onConsoleCopy,
  onOpenCodex,
  onRegisterRunAsAdministrator,
}: Omit<ShellProps, "onCleanCodex" | "onSelectSection">): HTMLElementTagNameMap["section"] {
  const panel = createControlSurface();

  panel.append(
    createTextElement("h2", "SectionTitle", "Admin"),
    createCodexStatusPanel({
      status: state.codexStatus,
    }),
    createCodexConsolePanel({
      messages: state.consoleMessages,
      onClear: onConsoleClear,
      onCopy: onConsoleCopy,
    }),
    createCodexActionPanel({
      actionStatus: state.actionStatus,
      codexStatus: state.codexStatus,
      onRefresh: onCodexRefresh,
      onOpenCodex,
      onRegisterRunAsAdministrator,
      runtimeStatus: state.runtimeStatus,
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

function createControlSurface(): HTMLElementTagNameMap["section"] {
  const panel = document.createElement("section");
  panel.className = "ControlSurface";
  return panel;
}
