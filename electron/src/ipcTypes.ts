export type RendererPaths = {
  base: string;
  database: string;
  export: string;
};

export type DesktopIpcError =
  | 'cancelled'
  | 'verificationAlreadyOpen'
  | 'verificationFailed'
  | 'sidecarUnavailable'
  | 'exportDialogFailed'
  | 'appPathsFailed'
  | 'externalLinkFailed';

export type DesktopIpcResult<T> =
  { ok: true; value: T } | { ok: false; error: DesktopIpcError };
