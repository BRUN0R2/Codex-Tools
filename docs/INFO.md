# INFO - Codex Tools

Codex Tools sera reconstruido como um utilitario moderno para Windows.

## Stack confirmada

- Tauri `2.11.2`.
- Tauri API `2.11.0`.
- Tauri Build `2.6.2`.
- Vite `8.0.14`.
- TypeScript `6.0.3`.
- Rust `1.95.0`.
- Node.js `26.1.0`.
- npm `11.13.0`.
- CMake `4.3.2`.
- MSVC `14.51`.

## Decisoes

- Usar Tauri 2 com Vanilla TypeScript para evitar framework sem necessidade real.
- Usar npm porque ja esta instalado e evita adicionar um gerenciador extra.
- Manter o app desktop em `apps/Desktop`.
- Manter frontend, backend Tauri e futuro core nativo isolados por responsabilidade.
- Usar convencoes de modulo do Rust no backend Tauri quando exigidas pelo toolchain.
- O fluxo principal usa prioridade alta fixa para todos os processos Codex.
- Prioridade em tempo real permanece fora do produto.
- Detectar Codex por caminhos conhecidos em `LOCALAPPDATA` e pelo `PATH`, retornando os caminhos verificados.
- Codex Tools deve pedir elevacao de administrador antes da janela abrir.
- Abrir Codex com elevacao via API nativa do Windows.
- Reaplicar prioridade alta por alguns ciclos curtos para cobrir os processos que surgem apos o carregamento inicial.
- Mostrar processos Codex em execucao com PID, prioridade atual e estado de administrador.
- Manter automacao e Registry fora da interface simplificada.

## Validacao atual

- `npm run build`.
- `cargo check --manifest-path apps/Desktop/src-tauri/Cargo.toml`.
- `cargo clippy --manifest-path apps/Desktop/src-tauri/Cargo.toml -- -D warnings`.
