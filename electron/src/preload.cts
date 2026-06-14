const { contextBridge, ipcRenderer } =
  require('electron') as typeof import('electron');

type RendererPaths = {
  base: string;
  database: string;
  export: string;
};
type SelectExportPathOptions = {
  defaultPath?: string;
};

const pathsArgumentPrefix = '--shinden-to-anilist-paths=';
const selectExportPathChannel = 'shinden-to-anilist:select-export-path';

function rendererPaths(): RendererPaths {
  const pathsArgument = process.argv.find((argument) =>
    argument.startsWith(pathsArgumentPrefix)
  );

  if (pathsArgument === undefined) {
    throw new Error('Missing ShindenToAnilist renderer paths');
  }

  return JSON.parse(
    pathsArgument.slice(pathsArgumentPrefix.length)
  ) as RendererPaths;
}

contextBridge.exposeInMainWorld('shindenToAnilist', {
  paths: rendererPaths(),
  selectExportPath: (options?: SelectExportPathOptions) =>
    ipcRenderer.invoke(selectExportPathChannel, options) as Promise<
      string | null
    >
});
