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
- Prioridade de processo deve ser escolha explicita do usuario: `Normal` ou `Alta`.
- Prioridade em tempo real permanece bloqueada.

## Validacao atual

- `npm run build`.
- `npm run tauri build`.
