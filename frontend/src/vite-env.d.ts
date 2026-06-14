/// <reference types="svelte" />
/// <reference types="vite/client" />

interface Window {
  shindenToAnilist?: {
    paths: {
      base: string;
      database: string;
      export: string;
    };
    grpcBaseUrl: string;
    selectExportPath?: (options?: {
      defaultPath?: string;
    }) => Promise<string | null>;
  };
}

declare var shindenToAnilist: Window['shindenToAnilist'];
