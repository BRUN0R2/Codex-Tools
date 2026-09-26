import { translate, translatePlural } from "./catalog";
import type { CommandFailure } from "../Domain/CommandResult";

export type CommandFailureContext = "cli" | "desktop" | "cleanup" | "uninstall" | "general";

export function translateCommandFailure(
  failure: CommandFailure,
  context: CommandFailureContext,
): string {
  switch (failure.code) {
    case "nativeRuntimeUnavailable":
      return translate("common.nativeRuntimeUnavailable");
    case "codexNotFound":
      return context === "cli" ? translate("cli.notFound") : translate("action.desktopNotFound");
    case "cleanupBlocked":
      return translate("cleanup.blocked");
    case "cleanupFailed":
      return translate("cleanup.failed", { detail: failure.message });
    case "uninstallBlocked":
      return translate("uninstall.blocked");
    case "uninstallFailed":
      return translate("uninstall.failed", { detail: failure.message });
    case "windowsApiFailed":
      return translate("common.windowsApiFailure", { detail: failure.message });
    case "invalidState":
      return translate("common.invalidState", { detail: failure.message });
    default:
      return failure.message;
  }
}

const CLEANUP_TARGET_KEYS: Readonly<Record<string, Parameters<typeof translate>[0]>> = {
  "Banco ativo": "cleanup.target.activeDatabase",
  "Indice de sessoes": "cleanup.target.sessionIndex",
  "Sessoes antigas": "cleanup.target.oldSessions",
  "Estado global": "cleanup.target.globalState",
  "Chats arquivados": "cleanup.target.archivedChats",
  Anexos: "cleanup.target.attachments",
  "Bancos legados": "cleanup.target.legacyDatabases",
  "Cache local": "cleanup.target.localCache",
  "Pastas vazias": "cleanup.target.emptyFolders",
};

const UNINSTALL_TARGET_KEYS: Readonly<Record<string, Parameters<typeof translate>[0]>> = {
  "Codex home": "uninstall.target.codexHome",
  "Cache de runtimes": "uninstall.target.runtimeCache",
  "LocalAppData OpenAI\\Codex": "uninstall.target.localCodex",
  "LocalAppData OpenAI": "uninstall.target.localOpenAi",
  "Dados do pacote MSIX": "uninstall.target.msixData",
  "ProgramData OpenAI\\Codex": "uninstall.target.programDataCodex",
  "ProgramData OpenAI": "uninstall.target.programDataOpenAi",
  "Workspace Documents\\Codex": "uninstall.target.documentsWorkspace",
  "Tarefa agendada elevada": "uninstall.target.elevatedTask",
  "AppCompat RUNASADMIN": "uninstall.target.appCompat",
  "Pacote MSIX OpenAI.Codex": "uninstall.target.msixPackage",
};

export function translateCleanupTargetName(name: string): string {
  const key = CLEANUP_TARGET_KEYS[name];
  if (key !== undefined) return translate(key);

  const temporaryDirectory = /^Temporarios (.+)$/u.exec(name)?.[1];
  return temporaryDirectory === undefined
    ? name
    : translate("cleanup.target.temporary", { directory: temporaryDirectory });
}

export function translateUninstallTargetName(name: string): string {
  const key = UNINSTALL_TARGET_KEYS[name];
  return key === undefined ? name : translate(key);
}

export function translateCleanupDiagnostic(message: string): string {
  if (message === "Cache de plugins preservado para manter ferramentas instaladas.") {
    return translate("cleanup.warning.pluginCache");
  }
  return message;
}

export function translateUninstallDiagnostic(message: string): string {
  const staticMessages: Readonly<Record<string, Parameters<typeof translate>[0]>> = {
    "Pasta .codex nao encontrada.": "uninstall.detail.codexHomeMissing",
    "Pasta .cache\\codex-runtimes nao encontrada.": "uninstall.detail.runtimeCacheMissing",
    "Pasta nao encontrada.": "uninstall.detail.folderMissing",
    "Pasta Packages nao encontrada.": "uninstall.detail.packagesFolderMissing",
    "Pasta OpenAI removida por estar vazia.": "uninstall.detail.emptyOpenAiFolderRemoved",
    "Nenhum pacote OpenAI.Codex_* encontrado em Packages.": "uninstall.detail.msixDataMissing",
    "Nenhuma tarefa elevada do Codex existia.": "uninstall.detail.elevatedTaskMissing",
    "Nenhuma entrada Codex encontrada no Registry.": "uninstall.detail.appCompatMissing",
    "Pacote desinstalado via Remove-AppxPackage.": "uninstall.detail.packageRemoved",
    "Pacote nao estava instalado para o usuario atual.": "uninstall.detail.packageMissing",
  };
  const key = staticMessages[message];
  if (key !== undefined) return translate(key);

  const removedTasks = /^([0-9]+) tarefa\(s\) elevada\(s\) do Codex removida\(s\)\.$/u.exec(message);
  if (removedTasks !== null) {
    return translatePlural("uninstall.detail.tasksRemoved", Number(removedTasks[1]), {
      count: Number(removedTasks[1]),
    });
  }

  const removedEntries = /^([0-9]+) entrada\(s\) removida\(s\)\.$/u.exec(message);
  if (removedEntries !== null) {
    return translatePlural("uninstall.detail.entriesRemoved", Number(removedEntries[1]), {
      count: Number(removedEntries[1]),
    });
  }

  const staticWarnings: Readonly<Record<string, Parameters<typeof translate>[0]>> = {
    "USERPROFILE ausente; cache codex-runtimes nao foi verificado.": "uninstall.warning.userProfileMissing",
    "LOCALAPPDATA ausente; dados locais OpenAI\\Codex nao foram verificados.": "uninstall.warning.localAppDataMissing",
    "LOCALAPPDATA\\OpenAI ainda contem outros dados e foi preservada.": "uninstall.warning.localOpenAiPreserved",
    "ProgramData ausente; requirements de sistema nao foram verificados.": "uninstall.warning.programDataMissing",
    "ProgramData\\OpenAI ainda contem outros dados e foi preservada.": "uninstall.warning.programDataOpenAiPreserved",
  };
  const warningKey = staticWarnings[message];
  if (warningKey !== undefined) return translate(warningKey);

  return message;
}
