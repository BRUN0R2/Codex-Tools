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
- [x] Salvar modo administrador persistente para executaveis Codex via
      AppCompat Registry.
- [x] Separar deteccao do desktop e dos binarios CLI do Codex.
- [x] Detectar runtimes Codex hashados em `LOCALAPPDATA`.
- [x] Reconhecer `ChatGPT.exe` como desktop apenas dentro do pacote Codex.
- [x] Limitar modo administrador persistente aos binarios primarios Codex.
- [x] Preservar sessoes CLI independentes ao reiniciar o desktop elevado.
- [x] Limpar chats, sessoes, anexos, estado global, bancos legados, temporarios
      e cache local do Codex em uma acao geral.

## Interface

- [x] Simplificar tela para foco unico em abrir Codex.
- [x] Remover controles extras de automacao e Registry.
- [x] Adicionar controle enxuto para salvar modo administrador persistente.
- [x] Mover `Abrir Codex` para o canto inferior direito.
- [x] Mostrar processos Codex em prioridade alta e como administrador.
- [x] Mover `Verificar Codex` para o lado esquerdo de `Abrir Codex`.
- [x] Trocar lista de processos por console simples com copiar e limpar.
- [x] Remover subtitulo duplicado abaixo de `Codex Tools`.
- [x] Mostrar `Estatus` com estados `esperando`, `abrindo codex` e `pronto`.
- [x] Exibir mensagens operacionais e diagnosticos de fallback na interface.
- [x] Separar a interface em abas laterais `Admin` e `Limpeza`.
- [x] Expor limpeza geral em um botao unico com resumo operacional.
- [x] Modernizar hierarquia visual, navegacao, estados e responsividade.

## Dependencias

- [x] Atualizar dependencias npm e Cargo para versoes estaveis atuais.
- [x] Atualizar TypeScript para `7.0.2` e manter modo estrito.
- [x] Manter auditoria npm sem vulnerabilidades conhecidas.

## Validacao

- [x] Compilar frontend.
- [x] Validar backend Rust.
- [x] Validar abas laterais e tela de limpeza no preview local.
- [ ] Validar abertura do Codex em runtime.
- [ ] Validar lista de processos apos o Codex carregar totalmente.

## Proximos passos

- [ ] Adicionar limpeza explicita de registros AppCompat legados para helpers
      que versoes anteriores marcaram como `RUNASADMIN`.
- [ ] Avaliar deep links `codex://` em um modulo isolado de navegacao desktop.
- [ ] Avaliar integracao opcional com `codex doctor` para diagnosticos locais.
- [ ] Avaliar cliente opcional do `codex app-server` sem acoplar o protocolo
      experimental ao launcher.
