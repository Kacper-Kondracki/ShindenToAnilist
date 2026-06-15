/// <reference types="svelte" />
/// <reference types="vite/client" />

interface Window {
  shindenToAnilist?: {
    paths: {
      base: string;
      database: string;
      export: string;
    };
    getGrpcBaseUrl?: () => Promise<string>;
    openExternalUrl?: (url: string) => Promise<void>;
    selectExportPath?: (options?: {
      defaultPath?: string;
    }) => Promise<string | null>;
  };
}

declare var shindenToAnilist: Window['shindenToAnilist'];
