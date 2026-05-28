# TODO - Codex Tools

## Base

- [ ] Confirmar stack atualizada antes de implementar.
- [ ] Criar estrutura modular do projeto.
- [ ] Configurar build moderno e enxuto.
- [ ] Registrar decisoes tecnicas essenciais em `docs/INFO.md`.

## Arquitetura

- [ ] Separar frontend Tauri, backend nativo e modulos de plataforma.
- [ ] Definir contratos explicitos para comandos, estados e erros.
- [ ] Manter cada modulo pequeno, coeso e com responsabilidade unica.
- [ ] Evitar fallback silencioso e comportamento implicito.

## Core nativo

- [ ] Implementar deteccao do Codex instalado no Windows.
- [ ] Implementar abertura do Codex com privilegios administrativos.
- [ ] Implementar leitura e aplicacao de prioridade `Normal` ou `Alta`.
- [ ] Garantir que a prioridade seja escolhida pelo usuario e nunca automatica.
- [ ] Bloquear prioridade em tempo real.

## Interface Tauri

- [ ] Criar app Tauri usando a versao mais atualizada no momento da implementacao.
- [ ] Criar tela principal direta para controlar o Codex.
- [ ] Adicionar seletor explicito de prioridade `Normal` ou `Alta`.
- [ ] Adicionar acoes para abrir Codex, instalar automacao e remover automacao.
- [ ] Evitar textos, controles e estados sem uso real.

## Automacao

- [ ] Criar instalacao em caminho estavel fora de `build`.
- [ ] Criar tarefa do Windows somente com configuracao escolhida pelo usuario.
- [ ] Criar atalho direto para abrir a interface.
- [ ] Adicionar remocao limpa da tarefa, atalho e binario instalado.

## Validacao

- [ ] Compilar em Release.
- [ ] Validar abertura do Codex em runtime.
- [ ] Validar aplicacao de prioridade `Normal`.
- [ ] Validar aplicacao de prioridade `Alta`.
- [ ] Validar instalacao e remocao limpa.
- [ ] Validar tarefa apos reiniciar ou fazer logon.
