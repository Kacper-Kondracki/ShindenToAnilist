import type {
  DesktopIpcResult,
  NativeBridge,
  RendererPaths
} from './ipcTypes.js' with {
  'resolution-mode': 'import'
};

const { contextBridge, ipcRenderer } =
  require('electron') as typeof import('electron');

type SelectExportPathOptions = {
  defaultPath?: string;
};
type ShindenCloudflareVerificationOptions = {
  mode?: 'clearance' | 'autocloseTest';
};

const getAppPathsChannel = 'shinden-to-anilist:get-app-paths';
const selectExportPathChannel = 'shinden-to-anilist:select-export-path';
const openExternalUrlChannel = 'shinden-to-anilist:open-external-url';
const openShindenCloudflareVerificationChannel =
  'shinden-to-anilist:open-shinden-cloudflare-verification';

contextBridge.exposeInMainWorld('shindenToAnilist', {
  reportFailure: (operation: string, detail: string) => {
    ipcRenderer.send('shinden-to-anilist:renderer-failure', operation, detail);
  },
  getAppPaths: () =>
    ipcRenderer.invoke(getAppPathsChannel) as Promise<
      DesktopIpcResult<RendererPaths>
    >,
  native: {
    call: (method, input) =>
      ipcRenderer.invoke('shinden-to-anilist:native-call', method, input),
    openStream: (method, input) =>
      ipcRenderer.invoke(
        'shinden-to-anilist:native-open-stream',
        method,
        input
      ),
    readStream: (id) =>
      ipcRenderer.invoke('shinden-to-anilist:native-read-stream', id),
    closeStream: (id) =>
      ipcRenderer.invoke('shinden-to-anilist:native-close-stream', id)
  } satisfies NativeBridge,
  openShindenCloudflareVerification: (
    options?: ShindenCloudflareVerificationOptions
  ) =>
    ipcRenderer.invoke(
      openShindenCloudflareVerificationChannel,
      options
    ) as Promise<
      DesktopIpcResult<{
        userAgent: string;
        cfClearance: string;
        domain: string;
        path: string;
        expiresUnixSeconds?: number;
        capturedAtMs: number;
      }>
    >,
  openExternalUrl: (url: string) =>
    ipcRenderer.invoke(openExternalUrlChannel, url) as Promise<
      DesktopIpcResult<void>
    >,
  selectExportPath: (options?: SelectExportPathOptions) =>
    ipcRenderer.invoke(selectExportPathChannel, options) as Promise<
      DesktopIpcResult<string | null>
    >
});
