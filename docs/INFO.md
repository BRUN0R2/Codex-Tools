# Technical Notes — Codex Tools

Codex Tools is being rebuilt as a modern Windows utility.

## Confirmed stack

- Tauri `2.11.6`.
- Tauri API `2.11.1`.
- Tauri CLI `2.11.5`.
- Tauri Build `2.6.3`.
- Vite `8.3.1`.
- TypeScript `7.0.2`.
- Rust `1.98.0` (Windows installation used for current validation).
- Node.js `26.10.0` (Windows installation used for validation).
- npm `12.1.0` (Windows installation used for validation).
- CMake `4.3.2`.
- MSVC `14.51`.

## Decisions

- Use Tauri 2 with Vanilla TypeScript to avoid an unnecessary framework.
- Keep the Rust library in its default `rlib` form, linked to the Windows executable; do not generate `cdylib` or `staticlib` libraries for other targets.
- Use npm and `package-lock.json` for reproducible dependency installation.
- The launcher validates Node.js, npm, and Cargo in both modes. If the terminal's `PATH` is stale, it looks for Node.js with npm in `%ProgramFiles%\nodejs`; if needed, it also looks for `npm.cmd` in `%APPDATA%\npm`. These directories are added only to the current process and its Tauri child processes.
- In a checkout without the project's local binaries, the launcher runs `npm ci`.
- Keep the desktop app in `apps/Desktop`.
- Keep the frontend, Tauri backend, and future native core isolated by responsibility.
- Detect the native bridge with `isTauri()` before running commands. Vite opened in a browser provides a UI-only preview; native actions are disabled, and the app does not call `invoke` without Tauri.
- Use Rust module conventions in the Tauri backend where required by the toolchain.
- Keep `docs/RULES.md` aligned with Rust, TypeScript, Tauri, and isolated Windows APIs.
- The main flow uses fixed high priority for all Codex processes.
- Realtime priority remains outside the product.
- Detect Codex Desktop in versioned packages under `Program Files\WindowsApps`, including the current `ChatGPT.exe` executable and the legacy name `Codex.exe`.
- Detect the standalone CLI at `%LOCALAPPDATA%\Programs\OpenAI\Codex\bin\codex.exe` and in `PATH` directories, without requiring Desktop to be installed.
- Open the CLI with `CreateProcessW`, `CREATE_NEW_CONSOLE`, and `HIGH_PRIORITY_CLASS`, without `STARTF_USESTDHANDLES`. Codex Tools is a Windows GUI application and has no reliable console channels for relaying the CLI. The new console provides its own input and output; it inherits the elevated token, and its priority is high before the first PID check. Passing `--no-daemon` avoids rejection by the shared daemon when elevated clients connect. Filtering `TERM=dumb` from the child environment lets the CLI detect the Windows console when it inherited that variable from a Codex session. Start in the user's profile and keep monitoring the PID to confirm that the CLI remains active.
- Uninstallation remains limited to the Desktop package and shared data; it does not remove the standalone CLI executable.
- Detect primary Codex binaries in versioned packages, hashed runtimes under `LOCALAPPDATA`, and through `PATH`.
- Codex Tools must request administrator elevation before its window opens.
- The release embeds a Windows manifest with `requireAdministrator`.
- The elevated token enables `SeDebugPrivilege` and `SeIncreaseBasePriorityPrivilege` to manage Codex processes.
- The window uses an explicit CSP because the app runs elevated.
- Open Desktop through the `shell:AppsFolder` entry for AUMID `OpenAI.Codex_2p2nqsd0c76g0!App`, using `ShellExecuteExW`, the `runas` verb, `--do-not-de-elevate`, and a persistent administrative profile at `%LOCALAPPDATA%\CodexTools\CodexAdminProfile`, without closing the existing session.
- The `OpenAI.Codex` MSIX package installed on 2026-09-26 (version `26.924.2738.0`) declares `runFullTrust`, but not `allowElevation`. Tests using `shell:AppsFolder` created `ChatGPT.exe` with MSIX identity and an elevated `codex.exe app-server`, while direct execution of the `.exe` failed without package identity. Therefore, do not infer that elevation is impossible from the manifest alone; inspect the actual process tokens.
  References: [Restricted capabilities](https://learn.microsoft.com/windows/apps/package-and-deploy/app-capability-declarations#restricted-capabilities) and [GetPackageFullName function](https://learn.microsoft.com/windows/win32/api/appmodel/nf-appmodel-getpackagefullname).
- Inspect the selected package's `AppxManifest.xml` for diagnostics, without using `allowElevation` as a blocker. Show the actual elevation state for each process individually, and require an elevated app-server for launch confirmation.
- Do not run the binary under `WindowsApps` with `CreateProcess` or a scheduled task. That path ends with `0x80070005` and starts no process, so there is no session whose priority can be stabilized. The scheduled task also started at priority 7, below normal.
- The elevated scheduled task is no longer created. Uninstallation still removes `OpenAI Codex Desktop Elevated` and `OpenAI Codex Elevated`.
- Priority stabilization is ready when Desktop and app-server processes exist and all detected Codex processes have high priority. Report elevation separately; it is not required to apply high priority.
- For the CLI, stabilization is ready when the started PID remains active and all detected Codex processes have high priority.
- Priority failures name the unmet condition or the PID that rejected the priority change.
- Run priority stabilization in the background to keep the interface responsive.
- Stabilization status must expose `Idle`, `Running`, `Succeeded`, and `Failed`.
- Runtime status `Ready` requires at least one Codex process and high priority for every detected process.
- Show running Codex processes with PID, current priority, and administrator status.
- `Normal` elevation in the console means the process has a normal token. Chromium helper processes may be normal even when Desktop and app-server are elevated.
- Keep generic automation out of the simplified interface.
- The `Processes` tab selector chooses Desktop or CLI for one `Open as administrator` button and one `Verify` button.
- The `Open as administrator` action does not close existing Codex processes. A separate profile avoids the single-instance lock of the current session.
- Inspect app-servers running from the Desktop package and hashed runtimes under `LOCALAPPDATA` as `codex.exe` processes.
- Maintain compatibility with the dynamic versioned folder `OpenAI.Codex_*__2p2nqsd0c76g0`.
- Discover language catalogs with `import.meta.glob` in `apps/Desktop/src/i18n/locales/*.json`. The `en.json` catalog is the reference and fallback; `pt-BR.json` is selected automatically when the WebView language is Portuguese.
- Validate canonical locale names, known keys, and placeholders when loading catalogs. Missing keys use English; adding another language requires only a new JSON file named for its locale.
- Format numbers and plurals with `Intl` for the selected locale.
- The `Settings` tab lets users select an installed catalog or automatic detection. Apply the choice immediately and save it in `localStorage` under `codex-tools.locale`. Removed catalogs revert to automatic detection.
- Use `rusqlite` with embedded SQLite to clean the local Codex database without requiring `sqlite3.exe` on Windows.
- Use `serde_json` to clean global Codex state with a real JSON parser, avoiding fragile text manipulation.
- Block general cleanup while Codex processes are open to avoid write conflicts in SQLite and local state.
- Preserve credentials, settings, installed skills, and plugins; clean conversations, sessions, attachments, temporary files, local caches, and legacy databases.
- Separate the `Uninstall` tab from partial cleanup: full, irreversible removal of Codex/ChatGPT Desktop data, with two-step confirmation in the interface.
- Uninstallation removes the `.codex` home, `LOCALAPPDATA\OpenAI\Codex`, `OpenAI.Codex_*` MSIX data, `ProgramData\OpenAI\Codex`, `.cache\codex-runtimes`, the `Documents\Codex` workspace, the `OpenAI Codex Elevated` task, `RUNASADMIN` AppCompat entries for Codex paths, and the MSIX package through `Remove-AppxPackage`.
- Block uninstallation while Codex processes are open.
- Preserve parent `OpenAI` folders only when they still contain other data.

## Current validation

- `npm run build`.
- `npm run test:i18n` validates exact selection, language-family matching, and English fallback.
- `cargo check --manifest-path apps/Desktop/src-tauri/Cargo.toml`.
- `cargo clippy --manifest-path apps/Desktop/src-tauri/Cargo.toml -- -D warnings`.
- `cargo test --manifest-path apps/Desktop/src-tauri/Cargo.toml --lib` validates tests without running the app. A full `cargo test` attempts to run the Tauri binary with a `requireAdministrator` manifest and fails with Windows error 740 from a non-elevated terminal.
- `npm run tauri build`, which enables `tauri/custom-protocol` and embeds `dist`.
- `cargo build --release` without that feature remains in development mode and opens `http://localhost:1420`. Without the Vite server, the WebView shows `ERR_CONNECTION_REFUSED`.
- `cargo fmt --manifest-path apps/Desktop/src-tauri/Cargo.toml -- --check`.
- `npm audit --audit-level=high`.
- The release manifest is configured through `WindowsAppManifest.xml` and validated by the Tauri build.
