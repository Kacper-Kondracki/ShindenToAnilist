/// <reference types="svelte" />
/// <reference types="vite/client" />

type DesktopIpcResult<T> =
  import('../../electron/src/ipcTypes.js').DesktopIpcResult<T>;

interface Window {
  isTauri?: boolean;
  __TAURI__?: unknown;
  __TAURI_INTERNALS__?: unknown;
  shindenToAnilist?: {
    reportFailure?: (operation: string, detail: string) => void;
    getAppPaths?: () => Promise<
      DesktopIpcResult<import('../../electron/src/ipcTypes.js').RendererPaths>
    >;
    native?: import('../../electron/src/ipcTypes.js').NativeBridge;
    openShindenCloudflareVerification?: (options?: {
      mode?: 'clearance' | 'autocloseTest';
    }) => Promise<
      DesktopIpcResult<{
        userAgent: string;
        cfClearance: string;
        domain: string;
        path: string;
        expiresUnixSeconds?: number;
        capturedAtMs: number;
      }>
    >;
    openExternalUrl?: (url: string) => Promise<DesktopIpcResult<void>>;
    selectExportPath?: (options?: {
      defaultPath?: string;
    }) => Promise<DesktopIpcResult<string | null>>;
  };
}

declare var isTauri: Window['isTauri'];
declare var __TAURI__: Window['__TAURI__'];
declare var __TAURI_INTERNALS__: Window['__TAURI_INTERNALS__'];
declare var shindenToAnilist: Window['shindenToAnilist'];
