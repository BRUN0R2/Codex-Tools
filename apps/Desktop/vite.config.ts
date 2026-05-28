import { defineConfig, type ServerOptions } from "vite";

type NodeProcessEnvironment = {
  readonly TAURI_DEV_HOST?: string;
};

type NodeGlobal = typeof globalThis & {
  readonly process?: {
    readonly env: NodeProcessEnvironment;
  };
};

const TAURI_DEV_SERVER_PORT = 1420;
const TAURI_HOT_RELOAD_PORT = 1421;
const TAURI_SOURCE_WATCH_PATTERN = "**/src-tauri/**";
const tauriDevHost = (globalThis as NodeGlobal).process?.env.TAURI_DEV_HOST;

const serverOptions: ServerOptions = {
  port: TAURI_DEV_SERVER_PORT,
  strictPort: true,
  host: tauriDevHost ?? false,
  hmr:
    tauriDevHost === undefined
      ? undefined
      : {
          protocol: "ws",
          host: tauriDevHost,
          port: TAURI_HOT_RELOAD_PORT,
        },
  watch: {
    ignored: [TAURI_SOURCE_WATCH_PATTERN],
  },
};

export default defineConfig({
  clearScreen: false,
  server: serverOptions,
});
