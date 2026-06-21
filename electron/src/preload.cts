const { contextBridge, ipcRenderer } =
  require('electron') as typeof import('electron');

type RendererPaths = {
  base: string;
  database: string;
  export: string;
};
type AppConfig = {
  paths: RendererPaths;
};
type SelectExportPathOptions = {
  defaultPath?: string;
};

const configArgumentPrefix = '--shinden-to-anilist-config=';
const selectExportPathChannel = 'shinden-to-anilist:select-export-path';
const getGrpcBaseUrlChannel = 'shinden-to-anilist:get-grpc-base-url';
const openExternalUrlChannel = 'shinden-to-anilist:open-external-url';
const openShindenCloudflareVerificationChannel =
  'shinden-to-anilist:open-shinden-cloudflare-verification';

function appConfig(): AppConfig {
  const configArgument = process.argv.find((argument) =>
    argument.startsWith(configArgumentPrefix)
  );

  if (configArgument === undefined) {
    throw new Error('Missing ShindenToAnilist app config');
  }

  return JSON.parse(
    configArgument.slice(configArgumentPrefix.length)
  ) as AppConfig;
}

const config = appConfig();

contextBridge.exposeInMainWorld('shindenToAnilist', {
  paths: config.paths,
  getGrpcBaseUrl: () =>
    ipcRenderer.invoke(getGrpcBaseUrlChannel) as Promise<string>,
  openShindenCloudflareVerification: () =>
    ipcRenderer.invoke(openShindenCloudflareVerificationChannel) as Promise<{
      userAgent: string;
      cfClearance: string;
      domain: string;
      path: string;
      expiresUnixSeconds?: number;
      capturedAtMs: number;
    }>,
  openExternalUrl: (url: string) =>
    ipcRenderer.invoke(openExternalUrlChannel, url) as Promise<void>,
  selectExportPath: (options?: SelectExportPathOptions) =>
    ipcRenderer.invoke(selectExportPathChannel, options) as Promise<
      string | null
    >
});
