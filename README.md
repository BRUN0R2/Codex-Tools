# Codex Tools

Codex Tools is a Windows desktop utility for managing the Codex Desktop and Codex CLI processes, cleaning local Codex data, and removing the desktop app and its shared data.

## Features

- Choose Codex Desktop or Codex CLI from one app selector, then verify or open the selected app as an administrator.
- Inspect detected Codex processes, their current priority, and their administrator status.
- Apply high priority while Codex starts and loads its processes.
- Clean local chats, sessions, attachments, temporary files, caches, and legacy databases while preserving credentials, settings, skills, and plugins.
- Select English, Brazilian Portuguese, or automatic language detection in Settings.
- Uninstall Codex Desktop and its shared data with a two-step confirmation.

## Download

Download the latest Windows release from [GitHub Releases](https://github.com/BRUN0R2/Codex-Tools/releases/latest). Review the release notes before installing.

## Run from source

Codex Tools requires Windows, Node.js with npm, and Rust with Cargo. Install the [Tauri Windows prerequisites](https://v2.tauri.app/start/prerequisites/), including Microsoft C++ build tools and WebView2.

From the repository root, run `codex-tools.cmd` and choose development or release mode. The launcher checks the required tools and prepares npm dependencies when needed.

To run commands directly, open `apps/Desktop` and use:

```powershell
npm ci
npm run test:i18n
npm run build
npm run tauri dev
```

Create a Windows release build with:

```powershell
npm run tauri build
```

The Codex Desktop and CLI launch actions require administrator permission. The CLI opens in a separate console. Running the Vite server in a browser shows a UI preview; native Windows actions are available only in the Tauri app.

## Documentation

- [Desktop app setup and usage](apps/Desktop/README.md)
- [Technical notes and implementation decisions](docs/INFO.md)
- [Project engineering rules](docs/RULES.md)
- [Project status and next steps](docs/TODO.md)

