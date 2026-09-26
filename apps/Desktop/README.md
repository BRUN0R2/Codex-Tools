# Codex Tools Desktop

Windows Tauri application for Codex Tools.

## Environment setup

- Install Node.js with npm (preferably the LTS version) and Rust with Cargo.
- Install the [Tauri prerequisites for Windows](https://v2.tauri.app/start/prerequisites/), including Microsoft C++ build tools and WebView2.
- In a new terminal, verify `node --version`, `npm.cmd --version`, and `cargo --version`.

From the repository root, run `codex-tools.cmd` and choose development or release mode. The launcher checks the required tools before starting. If the terminal has an outdated `PATH`, it finds the official installation under `%ProgramFiles%\nodejs`; if needed, it also looks for `npm.cmd` under `%APPDATA%\npm`. In a checkout without the project's local binaries, it runs `npm ci`. The `PATH` change applies only to the launcher process and its children, including Tauri setup commands.

## Codex Desktop and CLI

The `Processes` tab detects the Codex Desktop package and the standalone Codex CLI installation. One `App` selector switches between Desktop and CLI; the `Verify` and `Open as administrator` buttons act on the current selection. The CLI is searched for at `%LOCALAPPDATA%\Programs\OpenAI\Codex\bin\codex.exe` and in `PATH` directories. When opened, the CLI starts in a new Windows console with its own input and output channels, from the user's profile directory, and receives high priority as soon as the process is created. The CLI receives `--no-daemon` because the shared daemon rejects clients started with administrator privileges. The launcher also removes `TERM=dumb` from the child environment, when present, so the terminal interface can detect the Windows console. The CLI inherits Codex Tools' elevated token. The monitor tracks the started PID and confirms that it remains active and at high priority.

General cleanup operates on shared data under `.codex` and is blocked while any Codex process, including the CLI, is open. Uninstallation removes the Desktop package and shared Codex data; it does not remove the standalone executable installed by Codex CLI.

## Languages

Under `Settings > Language`, choose `Automatic (system language)`, `English`, or `Brazilian Portuguese`. The change takes effect immediately and is saved in the WebView's local storage. In automatic mode, the interface selects the best available language from the WebView preferences. It tries an exact match first, then another variant of the same language, and uses English when no compatible catalog is available. A saved preference for a removed catalog returns to automatic mode. Missing keys in a catalog use the English translation.

To add another language, create only `src/i18n/locales/<canonical-locale>.json`. The file contains `locale`, `name`, `direction`, and `messages`; `locale` must match the file's canonical locale name, such as `fr-CA`. The loader discovers JSON files automatically and validates keys and placeholders against `en.json`. There is no need to register the language in another file or change code.

## Elevation and priority

Codex Tools requests administrator permission to adjust the priority of Codex processes. When `Codex Desktop` is selected, the `Open as administrator` button uses the installed package's `shell:AppsFolder` entry with the `runas` verb and the `--do-not-de-elevate` and `--user-data-dir` arguments. The administrative profile is stored at `%LOCALAPPDATA%\CodexTools\CodexAdminProfile`; the first launch may require signing in to that profile. The current instance remains open.

Codex Tools reports success only when it finds an elevated `ChatGPT.exe` from the package with an elevated `codex.exe app-server` child. It then stabilizes high priority and shows each process token in the console. Some Chromium helper processes may retain a normal token.

With MSIX version `26.924.2738.0`, directly running `ChatGPT.exe` with `runas` created an elevated token without package identity and ended with the error `The process has no package identity`. The `shell:AppsFolder` entry preserved the MSIX identity and started a working administrative session. That version's manifest does not declare `allowElevation`; this field alone does not describe the actual result of activation through `shell:AppsFolder` on this installation.

To repeat the tests without closing the current instance, use `scripts/probe-codex-elevation.ps1` with `-InspectOnly` or with `-LaunchMode AppsFolderRunAsArgs -IsolatedProfile`. The script measures the token and package identity of new processes; `RunAs` uses the direct executable only as a diagnostic comparison.

## Direct commands

From `apps/Desktop`:

```powershell
npm ci
npm run test:i18n
npm run build
npm run tauri dev
npm run tauri build
```

`npm run build` also validates TypeScript types and builds the frontend with all discovered JSON catalogs.

Opening the Vite server directly in a browser shows a UI preview. The preview does not run native commands: verification, launch, cleanup, and uninstallation are unavailable. Use `npm run tauri dev` or the Windows build to validate these actions.
