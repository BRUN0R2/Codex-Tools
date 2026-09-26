import { isTauri } from "@tauri-apps/api/core";
import {
  cleanCodexWorkspace,
  getCodexCliStatus,
  getCodexStatus,
  openCodex,
  openCodexCli,
  uninstallCodexProduct,
} from "../Backend/CodexCommands";
import type { AppSection } from "../Domain/AppSection";
import type { CodexLaunchTarget } from "../Domain/CodexLaunchTarget";
import { codexStatusesAreEqual } from "../Domain/CodexInstallation";
import type { CodexCleanupReport } from "../Domain/CodexCleanup";
import type { CodexUninstallReport } from "../Domain/CodexUninstall";
import type { CodexLaunchResponse } from "../Domain/CodexLaunch";
import { formatBytes } from "../i18n/format";
import { setLocalePreference, translate, translatePlural } from "../i18n/catalog";
import type { LocalePreference } from "../i18n/locale-selection";
import { translateCommandFailure } from "../i18n/diagnostics";
import {
  INITIAL_APP_STATE,
  clearConsoleMessages,
  setCheckingCodexCliStatus,
  setActiveSection,
  setOpeningRuntimeStatus,
  setCheckingCodexStatus,
  setCodexStatus,
  setCodexCliStatus,
  setCleanupReport,
  setFailedActionStatus,
  setFailedCodexStatus,
  setRunningActionStatus,
  setSelectedLaunchTarget,
  setSucceededActionStatus,
  setUninstallConfirmationArmed,
  setUninstallReport,
  setWaitingRuntimeStatus,
  retranslateConsoleMessages,
  type AppState,
} from "./AppState";
import { createShell } from "./Shell";
import {
  priorityStabilizationHasFailed,
  priorityStabilizationHasFinished,
} from "../Domain/PriorityStabilization";

const CODEX_OPEN_STATUS_POLL_INTERVAL_MILLISECONDS = 500;
const CODEX_OPEN_STATUS_POLL_BUDGET_MILLISECONDS = 35_000;

export function mountApp(root: HTMLElement): void {
  const nativeRuntimeAvailable = isTauri();
  let state: AppState = {
    ...INITIAL_APP_STATE,
    nativeRuntimeAvailable,
    consoleMessages: nativeRuntimeAvailable ? INITIAL_APP_STATE.consoleMessages : [],
  };
  let openingMonitorVersion = 0;

  function render(): void {
    root.replaceChildren(
      createShell({
        state,
        onArmUninstallConfirmation(): void {
          armUninstallConfirmation();
        },
        onCancelUninstallConfirmation(): void {
          cancelUninstallConfirmation();
        },
        onCleanCodex(): void {
          void cleanCodex();
        },
        onRefreshSelected(): void {
          void refreshSelectedCodexStatus();
        },
        onConsoleClear(): void {
          clearConsole();
        },
        onConsoleCopy(): void {
          void copyConsole();
        },
        onOpenSelected(): void {
          void launchSelectedCodex();
        },
        onSelectLaunchTarget(target: CodexLaunchTarget): void {
          state = setSelectedLaunchTarget(state, target);
          render();
        },
        onSelectLocale(preference: LocalePreference): void {
          setLocalePreference(preference);
          state = retranslateConsoleMessages(state);
          render();
        },
        onSelectSection(section: AppSection): void {
          selectSection(section);
        },
        onUninstallCodex(): void {
          void uninstallCodex();
        },
      })
    );
  }

  function selectSection(section: AppSection): void {
    state = setActiveSection(state, section);
    render();
  }

  async function refreshSelectedCodexStatus(): Promise<void> {
    if (state.selectedLaunchTarget === "desktop") {
      await refreshCodexStatus();
    } else {
      await refreshCodexCliStatus();
    }
  }

  async function launchSelectedCodex(): Promise<void> {
    if (state.selectedLaunchTarget === "desktop") {
      await launchCodex();
    } else {
      await launchCodexCli();
    }
  }

  async function refreshCodexStatus(): Promise<void> {
    state = setCheckingCodexStatus(state);
    render();

    const result = await getCodexStatus();

    state = result.ok
      ? setCodexStatus(state, result.value)
      : setFailedCodexStatus(state, translateCommandFailure(result.error, "general"));
    render();
  }

  async function refreshCodexCliStatus(): Promise<void> {
    state = setCheckingCodexCliStatus(state);
    render();

    const result = await getCodexCliStatus();
    state = result.ok
      ? setCodexCliStatus(state, result.value)
      : setCodexCliStatus(state, {
          state: "Failed",
          message: translateCommandFailure(result.error, "general"),
        });
    render();
  }

  function clearConsole(): void {
    state = setSucceededActionStatus(clearConsoleMessages(state), translate("console.cleared"));
    render();
  }

  async function copyConsole(): Promise<void> {
    if (state.consoleMessages.length === 0) {
      state = setSucceededActionStatus(state, translate("console.empty"));
      render();
      return;
    }

    try {
      await navigator.clipboard.writeText(state.consoleMessages.join("\n"));
      state = setSucceededActionStatus(state, translate("console.copied"));
    } catch {
      state = setFailedActionStatus(state, translate("console.copyFailed"));
    }

    render();
  }

  async function launchCodex(): Promise<void> {
    const monitorVersion = openingMonitorVersion + 1;
    openingMonitorVersion = monitorVersion;
    state = setRunningActionStatus(setOpeningRuntimeStatus(state), "action.openingDesktop");
    render();

    const result = await openCodex();

    if (result.ok) {
      state = setSucceededActionStatus(state, createCodexLaunchSuccessMessage(result.value));
      render();
      void monitorCodexOpening(monitorVersion);
      return;
    }

    state = setFailedActionStatus(
      setWaitingRuntimeStatus(state),
      translateCommandFailure(result.error, "desktop"),
    );
    render();
    await refreshCodexStatus();
  }

  async function launchCodexCli(): Promise<void> {
    const monitorVersion = openingMonitorVersion + 1;
    openingMonitorVersion = monitorVersion;
    state = setRunningActionStatus(state, "cli.opening");
    render();

    const result = await openCodexCli();
    if (!result.ok) {
      const message = translateCommandFailure(result.error, "cli");
      state = setFailedActionStatus(state, message);
      render();
      await refreshCodexCliStatus();
      return;
    }

    render();
    void monitorCodexOpening(monitorVersion, "cli", result.value);
  }

  async function cleanCodex(): Promise<void> {
    state = setRunningActionStatus(state, "cleanup.cleaning");
    render();

    const result = await cleanCodexWorkspace();
    if (!result.ok) {
      state = setFailedActionStatus(state, translateCommandFailure(result.error, "cleanup"));
      render();
      return;
    }

    state = setCleanupReport(
      setSucceededActionStatus(state, createCleanupSuccessMessage(result.value)),
      result.value
    );
    render();
    await refreshCodexStatus();
  }

  function armUninstallConfirmation(): void {
    state = setUninstallConfirmationArmed(state, true);
    render();
  }

  function cancelUninstallConfirmation(): void {
    state = setUninstallConfirmationArmed(state, false);
    render();
  }

  async function uninstallCodex(): Promise<void> {
    state = setRunningActionStatus(state, "uninstall.confirming");
    render();

    const result = await uninstallCodexProduct();
    if (!result.ok) {
      state = setFailedActionStatus(
        setUninstallConfirmationArmed(state, false),
        translateCommandFailure(result.error, "uninstall")
      );
      render();
      return;
    }

    state = setUninstallReport(
      setSucceededActionStatus(state, createUninstallSuccessMessage(result.value)),
      result.value
    );
    render();
    await refreshCodexStatus();
  }

  async function monitorCodexOpening(
    monitorVersion: number,
    target: "desktop" | "cli" = "desktop",
    cliLaunchResponse?: CodexLaunchResponse,
  ): Promise<void> {
    const deadline = Date.now() + CODEX_OPEN_STATUS_POLL_BUDGET_MILLISECONDS;

    while (Date.now() < deadline) {
      await delay(CODEX_OPEN_STATUS_POLL_INTERVAL_MILLISECONDS);

      if (monitorVersion !== openingMonitorVersion) {
        return;
      }

      const result = await getCodexStatus();
      const previousCodexStatus = state.codexStatus;
      const previousRuntimeStatus = state.runtimeStatus;
      state = result.ok
        ? setCodexStatus(state, result.value)
        : setFailedCodexStatus(state, translateCommandFailure(result.error, "general"));

      if (
        !codexStatusesAreEqual(previousCodexStatus, state.codexStatus) ||
        previousRuntimeStatus !== state.runtimeStatus
      ) {
        render();
      }

      if (result.ok && priorityStabilizationHasFailed(result.value.priorityStabilization)) {
        state = setFailedActionStatus(
          state,
          result.value.priorityStabilization.message ?? translate("common.priorityFailed")
        );
        render();
        return;
      }

      if (result.ok && priorityStabilizationHasFinished(result.value.priorityStabilization)) {
        if (target === "cli" && cliLaunchResponse !== undefined) {
          state = setSucceededActionStatus(state, createCliLaunchSuccessMessage(cliLaunchResponse));
          render();
        }
        return;
      }
    }

    if (target === "desktop" && state.runtimeStatus === "Opening") {
      state = setFailedActionStatus(
        setWaitingRuntimeStatus(state),
        translate("common.priorityTimeout")
      );
      render();
    } else if (target === "cli") {
      state = setFailedActionStatus(state, translate("common.priorityTimeout"));
      render();
    }
  }

  function delay(milliseconds: number): Promise<void> {
    return new Promise((resolve) => {
      window.setTimeout(resolve, milliseconds);
    });
  }

  function createCodexLaunchSuccessMessage(response: CodexLaunchResponse): string {
    return translate("action.desktopLaunchSuccess", { pid: response.processId });
  }

  function createCliLaunchSuccessMessage(response: CodexLaunchResponse): string {
    return translate("cli.launchSuccess", { pid: response.processId });
  }

  function createCleanupSuccessMessage(report: CodexCleanupReport): string {
    return translate("cleanup.complete", {
      threads: translatePlural("cleanup.threadCount", report.removedThreadCount, {
        count: report.removedThreadCount,
      }),
      files: translatePlural("cleanup.fileCount", report.removedFileCount, {
        count: report.removedFileCount,
      }),
      size: formatBytes(report.freedBytes),
    });
  }

  function createUninstallSuccessMessage(report: CodexUninstallReport): string {
    const packageState = report.packageRemoved
      ? translate("uninstall.removed")
      : translate("uninstall.absent");
    return translate("uninstall.complete", {
      files: translatePlural("uninstall.fileCount", report.removedFileCount, {
        count: report.removedFileCount,
      }),
      folders: translatePlural("uninstall.folderCount", report.removedDirectoryCount, {
        count: report.removedDirectoryCount,
      }),
      size: formatBytes(report.freedBytes),
      packageState,
    });
  }

  render();
  if (state.nativeRuntimeAvailable) {
    void refreshCodexStatus();
    void refreshCodexCliStatus();
  }
}
