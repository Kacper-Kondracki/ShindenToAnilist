const { contextBridge, ipcRenderer } =
  require('electron') as typeof import('electron');

type RendererPaths = {
  base: string;
  database: string;
  export: string;
};
type AppConfig = {
  paths: RendererPaths;
  grpcBaseUrl: string;
};
type SelectExportPathOptions = {
  defaultPath?: string;
};

const configArgumentPrefix = '--shinden-to-anilist-config=';
const selectExportPathChannel = 'shinden-to-anilist:select-export-path';

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
  grpcBaseUrl: config.grpcBaseUrl,
  selectExportPath: (options?: SelectExportPathOptions) =>
    ipcRenderer.invoke(selectExportPathChannel, options) as Promise<
      string | null
    >
});
