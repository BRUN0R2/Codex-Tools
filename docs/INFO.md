# INFO - Codex Tools

Codex Tools sera reconstruido como um utilitario moderno para Windows.

## Stack confirmada

- Tauri `2.11.6`.
- Tauri API `2.11.1`.
- Tauri CLI `2.11.5`.
- Tauri Build `2.6.3`.
- Vite `8.3.1`.
- TypeScript `7.0.2`.
- Rust `1.98.0` (instalacao Windows usada na validacao atual).
- Node.js `26.10.0` (instalacao Windows usada na validacao).
- npm `12.1.0` (instalacao Windows usada na validacao).
- CMake `4.3.2`.
- MSVC `14.51`.

## Decisoes

- Usar Tauri 2 com Vanilla TypeScript para evitar framework sem necessidade real.
- Manter a biblioteca Rust como `rlib` padrao, ligada ao executavel Windows;
  nao gerar bibliotecas `cdylib` e `staticlib` destinadas a outros alvos.
- Usar npm e `package-lock.json` para instalar dependencias de forma reproduzivel.
- O launcher valida Node.js, npm e Cargo para ambos os modos. Se o `PATH` do
  terminal estiver desatualizado, procura Node.js com npm em
  `%ProgramFiles%\nodejs`; se necessario, procura `npm.cmd` em `%APPDATA%\npm`.
  Esses diretorios entram apenas no processo atual e nos filhos do Tauri.
- Em um checkout sem os binarios locais do projeto, o launcher executa `npm ci`.
- Manter o app desktop em `apps/Desktop`.
- Manter frontend, backend Tauri e futuro core nativo isolados por responsabilidade.
- Usar convencoes de modulo do Rust no backend Tauri quando exigidas pelo toolchain.
- Manter `docs/RULES.md` alinhado a Rust, TypeScript, Tauri e APIs Windows isoladas.
- O fluxo principal usa prioridade alta fixa para todos os processos Codex.
- Prioridade em tempo real permanece fora do produto.
- Detectar o desktop Codex pelos pacotes versionados em `Program Files\WindowsApps`,
  incluindo o executavel atual `ChatGPT.exe` e o nome legado `Codex.exe`.
- Detectar binarios primarios Codex em pacotes versionados, runtimes hashados em
  `LOCALAPPDATA` e pelo `PATH`.
- Codex Tools deve pedir elevacao de administrador antes da janela abrir.
- A release embute manifest Windows com `requireAdministrator`.
- O token elevado ativa `SeDebugPrivilege` e `SeIncreaseBasePriorityPrivilege` para gerenciar processos Codex.
- A janela usa CSP explicito porque o app roda elevado.
- Abrir o desktop pelo item `shell:AppsFolder` do AUMID
  `OpenAI.Codex_2p2nqsd0c76g0!App`, com `ShellExecuteExW`, verbo `runas`,
  `--do-not-de-elevate` e um perfil administrativo persistente em
  `%LOCALAPPDATA%\CodexTools\CodexAdminProfile`, sem encerrar a sessao existente.
- O pacote MSIX `OpenAI.Codex` instalado em 2026-09-26 (versao `26.924.2738.0`)
  declara `runFullTrust`, mas nao `allowElevation`. Testes com `shell:AppsFolder`
  criaram `ChatGPT.exe` com identidade MSIX e `codex.exe app-server` elevado,
  enquanto a execucao direta do `.exe` falhou sem identidade de pacote.
  Portanto, nao inferir impossibilidade de elevacao apenas pelo manifesto;
  verificar os tokens dos processos reais.
  Referencias: https://learn.microsoft.com/windows/apps/package-and-deploy/app-capability-declarations#restricted-capabilities
  e https://learn.microsoft.com/windows/win32/api/appmodel/nf-appmodel-getpackagefullname
- Inspecionar `AppxManifest.xml` do pacote selecionado como diagnostico, sem
  usar `allowElevation` como bloqueio. O estado de elevacao real e exibido
  individualmente para cada processo e o lancamento exige app-server elevado.
- Nao executar o binario em `WindowsApps` por `CreateProcess` nem por tarefa
  agendada. Esse caminho termina com `0x80070005` e nenhum processo nasce, entao
  a prioridade nao tem sessao para estabilizar. A tarefa tambem nascia com
  prioridade 7, abaixo do normal.
- A tarefa agendada elevada nao e mais criada. A desinstalacao ainda remove
  `OpenAI Codex Desktop Elevated` e `OpenAI Codex Elevated`.
- A estabilizacao de prioridade fica pronta quando existem processos desktop e
  app-server e todos os processos Codex detectados estao em prioridade alta. Elevacao e
  informada separadamente; nao e requisito para aplicar prioridade alta.
- A falha de prioridade nomeia a condicao ausente ou o PID que recusou a
  prioridade.
- A estabilizacao de prioridade roda em segundo plano para manter a interface responsiva.
- O status da estabilizacao deve expor `Idle`, `Running`, `Succeeded` e `Failed`.
- O status de runtime `pronto` exige ao menos um processo Codex e todos os
  processos detectados em prioridade alta.
- Mostrar processos Codex em execucao com PID, prioridade atual e estado de administrador.
- Elevacao `normal` no console indica token normal. Processos auxiliares do
  Chromium podem ser normais mesmo com desktop e app-server elevados.
- Manter automacao generica fora da interface simplificada.
- A acao `Abrir Codex como administrador` nao encerra processos Codex existentes.
  O perfil separado evita o bloqueio de instancia unica da sessao corrente.
- Inspecionar app-servers executados a partir do pacote desktop e dos runtimes
  hashados em `LOCALAPPDATA` como processos `codex.exe`.
- Manter compatibilidade com a pasta versionada dinamica
  `OpenAI.Codex_*__2p2nqsd0c76g0`.
- Usar `rusqlite` com SQLite embutido para limpar o banco local do Codex sem
  depender de `sqlite3.exe` instalado no Windows.
- Usar `serde_json` para limpar o estado global do Codex com parser JSON real,
  evitando manipulacao textual fragil.
- Bloquear a limpeza geral quando processos Codex estiverem abertos para evitar
  disputa de escrita no SQLite e no estado local.
- Preservar credenciais, configuracoes, skills e plugins instalados; limpar
  conversas, sessoes, anexos, temporarios, cache local e bancos legados.
- Separar a aba `Desinstalacao` da limpeza parcial: remocao total e irreversivel
  de dados Codex/ChatGPT Desktop, com confirmacao em dois passos na interface.
- A desinstalacao remove home `.codex`, `LOCALAPPDATA\OpenAI\Codex`, dados do
  pacote MSIX `OpenAI.Codex_*`, `ProgramData\OpenAI\Codex`, cache
  `.cache\codex-runtimes`, workspace `Documents\Codex`, tarefa elevada
  `OpenAI Codex Elevated`, entradas AppCompat `RUNASADMIN` de caminhos Codex e o
  pacote MSIX via `Remove-AppxPackage`.
- Bloquear a desinstalacao enquanto processos Codex estiverem abertos.
- Preservar pastas `OpenAI` pai apenas quando ainda contiverem outros dados.

## Validacao atual

- `npm run build`.
- `cargo check --manifest-path apps/Desktop/src-tauri/Cargo.toml`.
- `cargo clippy --manifest-path apps/Desktop/src-tauri/Cargo.toml -- -D warnings`.
- `cargo test --manifest-path apps/Desktop/src-tauri/Cargo.toml --lib` valida os
  testes sem executar o aplicativo. `cargo test` completo tenta executar o
  binario Tauri com manifest `requireAdministrator` e falha com erro Windows 740
  em um terminal sem elevacao.
- `npm run tauri build`, que liga `tauri/custom-protocol` e embute `dist`.
- `cargo build --release` sem essa feature continua em modo de desenvolvimento e abre `http://localhost:1420`. Sem o servidor do Vite, o WebView mostra `ERR_CONNECTION_REFUSED`.
- `cargo fmt --manifest-path apps/Desktop/src-tauri/Cargo.toml -- --check`.
- `npm audit --audit-level=high`.
- Manifest da release configurado por `WindowsAppManifest.xml` e validado pelo build Tauri.
