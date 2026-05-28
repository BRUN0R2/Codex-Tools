# TODO - Codex Tools

## Base

- [x] Manter regras do projeto em `docs/RULES.md`.
- [x] Alinhar regras do projeto a Rust, TypeScript, Tauri e APIs Windows isoladas.
- [x] Criar estrutura modular do projeto.
- [x] Configurar app desktop Tauri.
- [x] Registrar decisoes tecnicas em `docs/INFO.md`.

## Core

- [x] Detectar instalacao do Codex no Windows.
- [x] Abrir Codex com elevacao de administrador.
- [x] Embutir manifest `requireAdministrator` na release.
- [x] Ativar privilegios seguros do token elevado para gerenciar processos.
- [x] Definir CSP explicito para o app elevado.
- [x] Aplicar prioridade alta nos processos Codex.
- [x] Estabilizar prioridade alta durante o carregamento completo do Codex.
- [x] Expor sucesso ou falha da estabilizacao de prioridade.
- [x] Executar estabilizacao de prioridade sem bloquear a interface.
- [x] Exigir elevacao do Codex Tools antes da janela abrir.
- [x] Listar processos Codex com prioridade atual e estado de administrador.

## Interface

- [x] Simplificar tela para foco unico em abrir Codex.
- [x] Remover controles extras de automacao e Registry.
- [x] Mover `Abrir Codex` para o canto inferior direito.
- [x] Mostrar processos Codex em prioridade alta e como administrador.
- [x] Mover `Verificar Codex` para o lado esquerdo de `Abrir Codex`.
- [x] Trocar lista de processos por console simples com copiar e limpar.
- [x] Remover subtitulo duplicado abaixo de `Codex Tools`.
- [x] Mostrar `Estatus` com estados `esperando`, `abrindo codex` e `pronto`.

## Validacao

- [x] Compilar frontend.
- [x] Validar backend Rust.
- [ ] Validar abertura do Codex em runtime.
- [ ] Validar lista de processos apos o Codex carregar totalmente.
