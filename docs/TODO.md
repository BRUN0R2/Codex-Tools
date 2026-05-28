# TODO - Codex Tools

## Base

- [x] Confirmar stack atualizada antes de implementar.
- [x] Criar estrutura modular do projeto.
- [x] Configurar build moderno e enxuto.
- [x] Registrar decisoes tecnicas essenciais em `docs/INFO.md`.

## Arquitetura

- [x] Separar frontend Tauri, backend nativo e modulos de plataforma.
- [x] Definir contratos explicitos para comandos, estados e erros.
- [x] Manter cada modulo pequeno, coeso e com responsabilidade unica.
- [x] Evitar fallback silencioso e comportamento implicito.

## Core nativo

- [x] Implementar deteccao do Codex instalado no Windows.
- [x] Implementar abertura do Codex com privilegios administrativos.
- [x] Implementar leitura e aplicacao de prioridade `Normal` ou `Alta`.
- [x] Garantir que a prioridade seja escolhida pelo usuario e nunca automatica.
- [x] Bloquear prioridade em tempo real.

## Interface Tauri

- [x] Criar app Tauri usando a versao mais atualizada no momento da implementacao.
- [x] Criar tela principal direta para controlar o Codex.
- [x] Adicionar seletor explicito de prioridade `Normal` ou `Alta`.
- [x] Adicionar acao real para abrir Codex.
- [ ] Adicionar acoes reais para instalar e remover automacao.
- [x] Evitar textos, controles e estados sem uso real.

## Automacao

- [ ] Criar instalacao em caminho estavel fora de `build`.
- [ ] Criar tarefa do Windows somente com configuracao escolhida pelo usuario.
- [ ] Criar atalho direto para abrir a interface.
- [ ] Adicionar remocao limpa da tarefa, atalho e binario instalado.

## Validacao

- [x] Compilar em Release.
- [ ] Validar deteccao do Codex pela interface.
- [ ] Validar abertura do Codex em runtime.
- [ ] Validar aplicacao de prioridade `Normal`.
- [ ] Validar aplicacao de prioridade `Alta`.
- [ ] Validar instalacao e remocao limpa.
- [ ] Validar tarefa apos reiniciar ou fazer logon.
