export type RendererPaths = {
  base: string;
  database: string;
  export: string;
};

export type DesktopIpcError =
  | 'cancelled'
  | 'verificationAlreadyOpen'
  | 'verificationFailed'
  | 'exportDialogFailed'
  | 'appPathsFailed'
  | 'externalLinkFailed';

export type DesktopIpcResult<T> =
  { ok: true; value: T } | { ok: false; error: DesktopIpcError };

export type NativeReply = { data: Uint8Array; isError: boolean };

export type NativeIpcResult<T> =
  { ok: true; value: T } | { ok: false; error: 'unavailable' | 'failed' };

export type NativeBridge = {
  call: (
    method: string,
    input: Uint8Array
  ) => Promise<NativeIpcResult<NativeReply>>;
  openStream: (
    method: string,
    input: Uint8Array
  ) => Promise<NativeIpcResult<number>>;
  readStream: (id: number) => Promise<NativeIpcResult<NativeReply | null>>;
  closeStream: (id: number) => Promise<void>;
};
