# TODO - Codex Tools

## Base

- [x] Manter regras do projeto em `docs/RULES.md`.
- [x] Alinhar regras do projeto a Rust, TypeScript, Tauri e APIs Windows isoladas.
- [x] Criar estrutura modular do projeto.
- [x] Configurar app desktop Tauri.
- [x] Registrar decisoes tecnicas em `docs/INFO.md`.

## Core

- [x] Detectar instalacao do Codex no Windows.
- [x] Abrir o pacote Codex como administrador com perfil separado e verificar
      o app-server elevado em runtime.
- [x] Ativar o desktop pela entrada `shell:AppsFolder` sem encerrar a sessao atual.
- [x] Mostrar `allowElevation` apenas como diagnostico do manifesto, sem
      deduzir o token do processo a partir desse campo.
- [x] Embutir manifest `requireAdministrator` na release.
- [x] Ativar privilegios seguros do token elevado para gerenciar processos.
- [x] Definir CSP explicito para o app elevado.
- [x] Aplicar prioridade alta nos processos Codex.
- [x] Estabilizar prioridade alta durante o carregamento completo do Codex.
- [x] Expor sucesso ou falha da estabilizacao de prioridade.
- [x] Executar estabilizacao de prioridade sem bloquear a interface.
- [x] Exigir elevacao do Codex Tools antes da janela abrir.
- [x] Listar processos Codex com prioridade atual e estado de administrador.
- [x] Separar deteccao do desktop e dos binarios CLI do Codex.
- [x] Detectar e abrir Codex CLI independente em um terminal novo.
- [x] Estabilizar prioridade da CLI pelo PID iniciado sem exigir o Desktop.
- [x] Detectar runtimes Codex hashados em `LOCALAPPDATA`.
- [x] Reconhecer `ChatGPT.exe` como desktop apenas dentro do pacote Codex.
- [x] Preservar a instancia desktop atual e sessoes CLI independentes ao abrir Codex.
- [x] Limpar chats, sessoes, anexos, estado global, bancos legados, temporarios
      e cache local do Codex em uma acao geral.
- [x] Limpar na desinstalacao entradas AppCompat legadas que versoes anteriores
      marcaram como `RUNASADMIN`.

## Interface

- [x] Simplificar tela para foco unico em abrir Codex.
- [x] Remover controles extras de automacao e Registry.
- [x] Mover `Abrir Codex` para o canto inferior direito.
- [x] Mostrar prioridade e elevacao reais dos processos Codex separadamente.
- [x] Mover `Verificar Codex` para o lado esquerdo de `Abrir Codex`.
- [x] Trocar lista de processos por console simples com copiar e limpar.
- [x] Remover subtitulo duplicado abaixo de `Codex Tools`.
- [x] Mostrar `Estatus` com estados `esperando`, `abrindo codex` e `pronto`.
- [x] Exibir mensagens operacionais e diagnosticos de elevacao na interface.
- [x] Separar a interface em abas laterais `Processos`, `Limpeza` e `Desinstalacao`.
- [x] Expor limpeza geral em um botao unico com resumo operacional.
- [x] Adicionar aba `Desinstalacao` com remocao total de dados e pacote Codex.
- [x] Exigir confirmacao em dois passos antes da desinstalacao total.
- [x] Modernizar hierarquia visual, navegacao, estados e responsividade.
- [x] Exibir `pronto` somente com todos os processos em prioridade alta.
- [x] Detectar automaticamente portugues/ingles e usar ingles como fallback.
- [x] Descobrir um JSON por idioma sem registro manual por arquivo.
- [x] Localizar rotulos e mensagens operacionais da interface em ingles e pt-BR.

## Dependencias

- [x] Atualizar dependencias npm e Cargo para versoes estaveis atuais.
- [x] Atualizar TypeScript para `7.0.2` e manter modo estrito.
- [x] Manter auditoria npm sem vulnerabilidades conhecidas.

## Validacao

- [x] Compilar frontend.
- [x] Validar backend Rust.
- [x] Validar abas laterais e tela de limpeza no preview local.
- [x] Compilar frontend e backend com a aba de desinstalacao.
- [x] Compilar frontend com descoberta e validacao de catalogos JSON.
- [x] Testar idioma exato, familia de idioma e fallback para ingles.
- [x] Executar testes Rust de deteccao e estabilizacao da CLI.
- [ ] Validar abertura do Codex em runtime.
- [ ] Validar lista de processos apos o Codex carregar totalmente.
- [ ] Validar desinstalacao total em runtime com Codex fechado.

## Proximos passos

- [ ] Avaliar deep links `codex://` em um modulo isolado de navegacao desktop.
- [ ] Avaliar integracao opcional com `codex doctor` para diagnosticos locais.
- [ ] Avaliar cliente opcional do `codex app-server` sem acoplar o protocolo
      experimental ao launcher.
