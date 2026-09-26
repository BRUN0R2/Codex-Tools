# Codex Tools Desktop

Aplicativo Windows em Tauri para o Codex Tools.

## Preparacao do ambiente

- Instale Node.js com npm (preferencialmente a versao LTS) e Rust com Cargo.
- Instale os [pre-requisitos do Tauri no Windows](https://v2.tauri.app/start/prerequisites/), incluindo as ferramentas de compilacao C++ da Microsoft e WebView2.
- Confirme em um novo terminal: `node --version`, `npm.cmd --version` e `cargo --version`.

Na raiz do repositorio, execute `codex-tools.cmd` e escolha modo dev ou release. O launcher verifica as ferramentas antes de iniciar. Se o terminal estiver com um `PATH` antigo, ele encontra a instalacao oficial em `%ProgramFiles%\nodejs`; se necessario, procura `npm.cmd` tambem em `%APPDATA%\npm`. Em um checkout sem os binarios locais do projeto, executa `npm ci`. A alteracao do `PATH` vale apenas para o processo do launcher e seus filhos, incluindo os comandos de preparacao do Tauri.

## Codex Desktop e CLI

A aba `Processos` detecta o pacote Codex Desktop e a instalacao independente do Codex CLI. A CLI e procurada em `%LOCALAPPDATA%\Programs\OpenAI\Codex\bin\codex.exe` e nos diretorios do `PATH`. `Abrir CLI como administrador` inicia `codex.exe` em um novo terminal, usando a pasta do perfil do usuario como diretorio inicial. Como o Codex Tools precisa de elevacao para ajustar prioridades, a janela de terminal e a CLI iniciada tambem recebem o token elevado. O monitor acompanha o PID da CLI iniciada e ajusta a prioridade alta dos processos Codex.

A limpeza geral usa os dados compartilhados em `.codex` e bloqueia enquanto qualquer processo Codex estiver aberto, incluindo a CLI. A desinstalacao existente remove o pacote Desktop e dados Codex compartilhados; ela nao remove o executavel independente instalado pelo Codex CLI.

## Idiomas

A interface escolhe automaticamente o melhor idioma disponivel a partir das preferencias de idioma do WebView. Os catalogos incluidos sao `en` e `pt-BR`. A busca tenta uma correspondencia exata, depois outra variante do mesmo idioma e usa ingles quando nao ha catalogo compativel. Chaves ausentes em um catalogo tambem usam a traducao inglesa.

Para adicionar outro idioma, crie somente `src/i18n/locales/<locale-canonico>.json`. O arquivo inclui `locale`, `name`, `direction` e `messages`; `locale` precisa corresponder ao nome canonico do arquivo, como `fr-CA`. O carregador descobre os JSON automaticamente e valida as chaves e os placeholders contra `en.json`. Nao e necessario registrar o idioma em outro arquivo ou alterar codigo.

## Elevacao e prioridade

O Codex Tools solicita permissao de administrador para ajustar a prioridade dos processos Codex. O botao `Abrir Codex como administrador` usa a entrada `shell:AppsFolder` do pacote instalado com o verbo `runas` e os argumentos `--do-not-de-elevate` e `--user-data-dir`. O perfil administrativo fica em `%LOCALAPPDATA%\CodexTools\CodexAdminProfile`; a primeira abertura pode exigir login nesse perfil. A instancia atual permanece aberta.

O Codex Tools so informa sucesso quando encontra um `ChatGPT.exe` elevado do pacote com `codex.exe app-server` elevado como filho. Depois estabiliza a prioridade alta e mostra o token de cada processo no console. Alguns processos auxiliares do Chromium podem permanecer com token normal.

Na versao MSIX `26.924.2738.0`, a execucao direta de `ChatGPT.exe` com `runas` gerou token elevado sem identidade de pacote e terminou com `O processo nao tem identificador de pacote`. A entrada `shell:AppsFolder` preservou a identidade MSIX e iniciou uma sessao administrativa funcional. O manifesto dessa versao nao declara `allowElevation`; esse campo isolado nao descreve o resultado real da ativacao por `shell:AppsFolder` nesta instalacao.

Para repetir os testes sem encerrar a instancia atual, use `scripts/probe-codex-elevation.ps1` com `-InspectOnly` ou com `-LaunchMode AppsFolderRunAsArgs -IsolatedProfile`. O script mede token e identidade de pacote dos processos novos; `RunAs` usa o executavel direto apenas como comparacao diagnostica.

## Comandos diretos

Dentro de `apps/Desktop`:

- `npm ci`
- `npm run test:i18n`
- `npm run build`
- `npm run tauri dev`
- `npm run tauri build`

`npm run build` tambem valida os tipos TypeScript e gera o frontend com todos os catalogos JSON descobertos.
