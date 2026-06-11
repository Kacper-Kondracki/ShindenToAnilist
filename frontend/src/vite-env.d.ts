/// <reference types="svelte" />
/// <reference types="vite/client" />

interface Window {
  shindenToAnilist?: {
    paths: {
      base: string;
      database: string;
      export: string;
    };
  };
}

declare var shindenToAnilist: Window['shindenToAnilist'];
