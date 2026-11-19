import { create, fromBinary, toBinary } from '@bufbuild/protobuf';
import type { Transport } from '@connectrpc/connect';
import type {
  NativeBridge,
  NativeIpcResult,
  NativeReply
} from '../../../../electron/src/ipcTypes';
import {
  AppErrorSchema,
  ErrorKind
} from '../gen/shinden_to_anilist/v1/error_pb';
import { AppFailure } from './errors';

export function createElectronTransport(bridge: NativeBridge): Transport {
  return {
    async unary(method, signal, timeoutMs, _header, input) {
      const abort = withTimeout(signal, timeoutMs);
      checkAbort(abort);
      const bytes = toBinary(method.input, create(method.input, input));
      const response = await invoke(() => bridge.call(method.name, bytes));
      checkAbort(abort);
      return {
        stream: false,
        service: method.parent,
        method,
        header: new Headers(),
        trailer: new Headers(),
        message: fromBinary(method.output, readReply(response))
      };
    },

    async stream(method, signal, timeoutMs, _header, input) {
      if (method.methodKind !== 'server_streaming') {
        throw new AppFailure(ErrorKind.UNKNOWN);
      }

      const abort = withTimeout(signal, timeoutMs);
      async function* messages() {
        checkAbort(abort);
        for await (const request of input) {
          const bytes = toBinary(method.input, create(method.input, request));
          const id = await invoke(() => bridge.openStream(method.name, bytes));
          let closing: Promise<void> | undefined;
          const close = () => {
            closing ??= bridge.closeStream(id).catch(() => {
              // The IPC endpoint can disappear during window disposal.
            });
            return closing;
          };

          abort?.addEventListener('abort', close, { once: true });
          try {
            checkAbort(abort);
            while (true) {
              const reply = await invoke(() => bridge.readStream(id));
              checkAbort(abort);
              if (reply === null) return;
              yield fromBinary(method.output, readReply(reply));
              checkAbort(abort);
            }
          } finally {
            abort?.removeEventListener('abort', close);
            await close();
          }
        }
      }

      return {
        stream: true,
        service: method.parent,
        method,
        header: new Headers(),
        trailer: new Headers(),
        message: messages()
      };
    }
  };
}

function withTimeout(
  signal: AbortSignal | undefined,
  timeoutMs: number | undefined
) {
  if (timeoutMs === undefined || timeoutMs <= 0) return signal;
  const timeout = AbortSignal.timeout(timeoutMs);
  return signal === undefined ? timeout : AbortSignal.any([signal, timeout]);
}

function checkAbort(signal: AbortSignal | undefined): void {
  if (!signal?.aborted) return;
  const kind =
    signal.reason instanceof DOMException &&
    signal.reason.name === 'TimeoutError'
      ? ErrorKind.DEADLINE_EXCEEDED
      : ErrorKind.CANCELLED;
  throw new AppFailure(kind, { cause: signal.reason });
}

async function invoke<T>(run: () => Promise<NativeIpcResult<T>>): Promise<T> {
  let response: NativeIpcResult<T>;
  try {
    response = await run();
  } catch (cause) {
    throw new AppFailure(ErrorKind.SERVICE_UNAVAILABLE, { cause });
  }
  if (!response.ok) {
    const kind =
      response.error === 'unavailable'
        ? ErrorKind.SERVICE_UNAVAILABLE
        : ErrorKind.UNKNOWN;
    throw new AppFailure(kind, undefined, undefined, true);
  }
  return response.value;
}

function readReply(reply: NativeReply): Uint8Array {
  if (reply.isError) {
    const detail = fromBinary(AppErrorSchema, reply.data);
    throw new AppFailure(
      detail.kind || ErrorKind.UNKNOWN,
      undefined,
      detail.httpStatus,
      true
    );
  }
  return reply.data;
}
