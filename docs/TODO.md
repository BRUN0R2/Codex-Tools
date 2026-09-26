# TODO — Codex Tools

## Foundation

- [x] Keep project rules in `docs/RULES.md`.
- [x] Align project rules with Rust, TypeScript, Tauri, and isolated Windows APIs.
- [x] Create the modular project structure.
- [x] Configure the Tauri desktop app.
- [x] Record technical decisions in `docs/INFO.md`.

## Core

- [x] Detect Codex installation on Windows.
- [x] Open the Codex package as administrator with a separate profile and verify the elevated app-server at runtime.
- [x] Activate Desktop through the `shell:AppsFolder` entry without closing the current session.
- [x] Show `allowElevation` only as a manifest diagnostic; do not infer the process token from that field.
- [x] Embed a `requireAdministrator` manifest in the release.
- [x] Enable safe elevated-token privileges for process management.
- [x] Define an explicit CSP for the elevated app.
- [x] Apply high priority to Codex processes.
- [x] Stabilize high priority while Codex finishes loading its processes.
- [x] Expose priority-stabilization success or failure.
- [x] Run priority stabilization without blocking the interface.
- [x] Require Codex Tools to elevate before its window opens.
- [x] List Codex processes with their current priority and administrator status.
- [x] Separate detection of Desktop and Codex CLI binaries.
- [x] Detect and open the standalone Codex CLI in a new console with its own input and output and high priority from process creation.
- [x] Stabilize CLI priority by the started PID without requiring Desktop.
- [x] Detect hashed Codex runtimes under `LOCALAPPDATA`.
- [x] Recognize `ChatGPT.exe` as Desktop only inside the Codex package.
- [x] Preserve the current Desktop instance and independent CLI sessions when opening Codex.
- [x] Clean Codex chats, sessions, attachments, global state, legacy databases, temporary files, and local caches in one general action.
- [x] Remove legacy AppCompat entries marked `RUNASADMIN` by previous versions during uninstallation.

## Interface

- [x] Simplify the screen to focus on opening Codex.
- [x] Remove extra automation and Registry controls.
- [x] Move `Open Codex` to the lower-right corner.
- [x] Show actual priority and elevation separately for Codex processes.
- [x] Move `Verify Codex` to the left of `Open Codex`.
- [x] Replace the process list with a simple console that supports copy and clear.
- [x] Remove the duplicate subtitle below `Codex Tools`.
- [x] Show status with `Waiting`, `Opening Codex`, and `Ready` states.
- [x] Display operational messages and elevation diagnostics in the interface.
- [x] Split the interface into `Processes`, `Cleanup`, `Uninstall`, and `Settings` sidebar tabs.
- [x] Unify Desktop/CLI launch behind one app selector and one launch button.
- [x] Let users choose and save the interface language in `Settings`.
- [x] Distinguish the browser preview from the Tauri runtime and disable native actions outside the Windows app.
- [x] Provide general cleanup through one button with an operational summary.
- [x] Add an `Uninstall` tab for full removal of Codex data and package.
- [x] Require two-step confirmation before full uninstallation.
- [x] Modernize visual hierarchy, navigation, states, and responsiveness.
- [x] Show `Ready` only when every process has high priority.
- [x] Detect Portuguese/English automatically and use English as the fallback.
- [x] Discover one JSON file per language without per-file manual registration.
- [x] Localize interface labels and operational messages in English and Brazilian Portuguese.

## Dependencies

- [x] Update npm and Cargo dependencies to current stable versions.
- [x] Update TypeScript to `7.0.2` and keep strict mode enabled.
- [x] Keep the npm audit free of known vulnerabilities.

## Validation

- [x] Build the frontend.
- [x] Validate the Rust backend.
- [x] Validate sidebar tabs and the cleanup screen in the local preview.
- [x] Build the frontend and backend with the uninstall tab.
- [x] Build the frontend with JSON catalog discovery and validation.
- [x] Test exact locale, language-family matching, and English fallback.
- [x] Test manual language preference and fallback when the saved preference is unavailable.
- [x] Build the native CLI launch with a console and initial high priority.
- [x] Run Rust tests for CLI detection and priority stabilization.
- [x] Confirm the CLI remains active in an elevated console with `--no-daemon`.
- [ ] Validate opening Codex at runtime.
- [ ] Validate opening the CLI from the updated build's panel.
- [ ] Validate the process list after Codex finishes loading.
- [ ] Validate full uninstallation at runtime with Codex closed.

## Next steps

- [ ] Evaluate `codex://` deep links in an isolated desktop navigation module.
- [ ] Evaluate optional integration with `codex doctor` for local diagnostics.
- [ ] Evaluate an optional `codex app-server` client without coupling the experimental protocol to the launcher.
