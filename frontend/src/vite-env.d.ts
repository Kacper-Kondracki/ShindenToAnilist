/// <reference types="svelte" />
/// <reference types="vite/client" />

interface Window {
  isTauri?: boolean;
  __TAURI__?: unknown;
  __TAURI_INTERNALS__?: unknown;
  shindenToAnilist?: {
    paths: {
      base: string;
      database: string;
      export: string;
    };
    getGrpcBaseUrl?: () => Promise<string>;
    openShindenCloudflareVerification?: () => Promise<{
      userAgent: string;
      cfClearance: string;
      domain: string;
      path: string;
      expiresUnixSeconds?: number;
      capturedAtMs: number;
    }>;
    openExternalUrl?: (url: string) => Promise<void>;
    selectExportPath?: (options?: {
      defaultPath?: string;
    }) => Promise<string | null>;
  };
}

declare var isTauri: Window['isTauri'];
declare var __TAURI__: Window['__TAURI__'];
declare var __TAURI_INTERNALS__: Window['__TAURI_INTERNALS__'];
declare var shindenToAnilist: Window['shindenToAnilist'];
