# Codex Tools Desktop

Aplicativo Windows em Tauri para o Codex Tools.

## Preparacao do ambiente

- Instale Node.js com npm (preferencialmente a versao LTS) e Rust com Cargo.
- Instale os [pre-requisitos do Tauri no Windows](https://v2.tauri.app/start/prerequisites/), incluindo as ferramentas de compilacao C++ da Microsoft e WebView2.
- Confirme em um novo terminal: `node --version`, `npm.cmd --version` e `cargo --version`.

Na raiz do repositorio, execute `codex-tools.cmd` e escolha modo dev ou release. O launcher verifica as ferramentas antes de iniciar. Se o terminal estiver com um `PATH` antigo, ele encontra a instalacao oficial em `%ProgramFiles%\nodejs`; se necessario, procura `npm.cmd` tambem em `%APPDATA%\npm`. Em um checkout sem os binarios locais do projeto, executa `npm ci`. A alteracao do `PATH` vale apenas para o processo do launcher e seus filhos, incluindo os comandos de preparacao do Tauri.

## Elevacao e prioridade

O Codex Tools solicita permissao de administrador para ajustar a prioridade dos processos Codex. O botao `Abrir Codex como administrador` usa a entrada `shell:AppsFolder` do pacote instalado com o verbo `runas` e os argumentos `--do-not-de-elevate` e `--user-data-dir`. O perfil administrativo fica em `%LOCALAPPDATA%\CodexTools\CodexAdminProfile`; a primeira abertura pode exigir login nesse perfil. A instancia atual permanece aberta.

O Codex Tools so informa sucesso quando encontra um `ChatGPT.exe` elevado do pacote com `codex.exe app-server` elevado como filho. Depois estabiliza a prioridade alta e mostra o token de cada processo no console. Alguns processos auxiliares do Chromium podem permanecer com token normal.

Na versao MSIX `26.924.2738.0`, a execucao direta de `ChatGPT.exe` com `runas` gerou token elevado sem identidade de pacote e terminou com `O processo nao tem identificador de pacote`. A entrada `shell:AppsFolder` preservou a identidade MSIX e iniciou uma sessao administrativa funcional. O manifesto dessa versao nao declara `allowElevation`; esse campo isolado nao descreve o resultado real da ativacao por `shell:AppsFolder` nesta instalacao.

Para repetir os testes sem encerrar a instancia atual, use `scripts/probe-codex-elevation.ps1` com `-InspectOnly` ou com `-LaunchMode AppsFolderRunAsArgs -IsolatedProfile`. O script mede token e identidade de pacote dos processos novos; `RunAs` usa o executavel direto apenas como comparacao diagnostica.

## Comandos diretos

Dentro de `apps/Desktop`:

- `npm ci`
- `npm run tauri dev`
- `npm run tauri build`
