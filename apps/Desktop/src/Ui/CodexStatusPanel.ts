import type { CodexStatus } from "../Domain/CodexInstallation";

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
  eyebrow.textContent = "Codex Desktop";

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
      return "Aguardando verificacao";
    case "Checking":
      return "Verificando instalacao";
    case "Found":
      return "Codex localizado";
    case "NotFound":
      return "Codex nao encontrado";
    case "Failed":
      return "Falha ao verificar o Codex";
  }
}

function createCodexStatusDetail(status: CodexStatus): string {
  switch (status.state) {
    case "Unchecked":
      return "A verificacao automatica ainda nao foi concluida.";
    case "Checking":
      return "Procurando a instalacao e os processos em execucao.";
    case "Found":
      return status.executablePath;
    case "NotFound":
      return `${status.checkedPaths.length} caminhos verificados.`;
    case "Failed":
      return status.message;
  }
}

function createCodexStatusBadge(status: CodexStatus): string {
  switch (status.state) {
    case "Unchecked":
      return "Aguardando";
    case "Checking":
      return "Verificando";
    case "Found":
      return "Instalado";
    case "NotFound":
      return "Nao encontrado";
    case "Failed":
      return "Erro";
  }
}
