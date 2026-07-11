# INFO - Codex Tools

Codex Tools sera reconstruido como um utilitario moderno para Windows.

## Stack confirmada

- Tauri `2.11.5`.
- Tauri API `2.11.1`.
- Tauri Build `2.6.3`.
- Vite `8.1.4`.
- TypeScript `7.0.2`.
- Rust `1.96.0`.
- Node.js `26.3.0`.
- npm `11.17.0`.
- CMake `4.3.2`.
- MSVC `14.51`.

## Decisoes

- Usar Tauri 2 com Vanilla TypeScript para evitar framework sem necessidade real.
- Usar npm porque ja esta instalado e evita adicionar um gerenciador extra.
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
- Abrir Codex com elevacao via API nativa do Windows.
- Reaplicar prioridade alta por alguns ciclos curtos para cobrir os processos que surgem apos o carregamento inicial.
- A estabilizacao de prioridade roda em segundo plano para manter a interface responsiva.
- O status da estabilizacao deve expor `Idle`, `Running`, `Succeeded` e `Failed`.
- O status de runtime `pronto` exige ao menos um processo Codex e todos os
  processos detectados em prioridade alta.
- Mostrar processos Codex em execucao com PID, prioridade atual e estado de administrador.
- Elevação `normal` no console indica processo existente ou processo que nao foi reaberto com token administrativo.
- Manter automacao generica fora da interface simplificada.
- Expor apenas uma acao de Registry: salvar `~ RUNASADMIN` em
  `HKCU\Software\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Layers`
  somente para os binarios primarios `ChatGPT.exe`, `Codex.exe` e `codex.exe` encontrados
  nas pastas Codex conhecidas.
- Nao registrar ferramentas auxiliares como `node.exe`, `rg.exe`, executaveis
  de plugins ou helpers do Chromium como `RUNASADMIN`.
- Ao reiniciar o desktop elevado, encerrar apenas processos pertencentes ao
  pacote desktop selecionado para preservar sessoes CLI independentes.
- Reconhecer app-servers executados a partir do pacote desktop e dos runtimes
  hashados em `LOCALAPPDATA`.
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

## Validacao atual

- `npm run build`.
- `cargo check --manifest-path apps/Desktop/src-tauri/Cargo.toml`.
- `cargo clippy --manifest-path apps/Desktop/src-tauri/Cargo.toml -- -D warnings`.
- `npm run tauri build`.
- `cargo fmt --manifest-path apps/Desktop/src-tauri/Cargo.toml -- --check`.
- `npm audit --audit-level=high`.
- Manifest da release configurado por `WindowsAppManifest.xml` e validado pelo build Tauri.
