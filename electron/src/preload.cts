const { contextBridge } = require('electron') as typeof import('electron');

type RendererPaths = {
  base: string;
  database: string;
  export: string;
};

const pathsArgumentPrefix = '--shinden-to-anilist-paths=';

function rendererPaths(): RendererPaths {
  const pathsArgument = process.argv.find((argument) =>
    argument.startsWith(pathsArgumentPrefix)
  );

  if (pathsArgument === undefined) {
    throw new Error('Missing ShindenToAnilist renderer paths');
  }

  return JSON.parse(pathsArgument.slice(pathsArgumentPrefix.length)) as RendererPaths;
}

contextBridge.exposeInMainWorld('shindenToAnilist', {
  paths: rendererPaths()
});
