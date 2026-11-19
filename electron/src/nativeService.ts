import type { IpcMainInvokeEvent, WebContents } from 'electron';
import { createRequire } from 'node:module';
import { join } from 'node:path';
import { logFailure } from './diagnostics.js';
import type { NativeIpcResult, NativeReply } from './ipcTypes.js';

interface NativeStream {
  next(): Promise<NativeReply | null | undefined>;
  close(): void;
}

interface NativeService {
  call(method: string, input: Buffer): Promise<NativeReply>;
  openStream(method: string, input: Buffer): NativeStream;
}

type NativeModule = { NativeService: new () => NativeService };

export function loadNativeService(
  resourceDir: string
): NativeService | undefined {
  const path = join(resourceDir, 'native', 'shinden_to_anilist.node');
  try {
    const { NativeService } = createRequire(import.meta.url)(
      path
    ) as NativeModule;
    return new NativeService();
  } catch (error) {
    logFailure(`Could not load native service at ${path}`, error);
    return undefined;
  }
}

export function registerNativeServiceIpc(
  contents: WebContents,
  rendererUrl: string,
  service: NativeService | undefined
): void {
  const expected = new URL(rendererUrl);
  expected.hash = '';
  const streams = new Map<number, { stream: NativeStream; reading: boolean }>();
  let nextId = 0;

  function trusted(event: IpcMainInvokeEvent): boolean {
    const sender = event.senderFrame;
    if (
      event.sender !== contents ||
      sender === null ||
      sender.parent !== null ||
      sender.url === ''
    ) {
      return false;
    }

    const actual = new URL(sender.url);
    actual.hash = '';
    return actual.href === expected.href;
  }

  async function handle<T>(
    event: IpcMainInvokeEvent,
    run: (service: NativeService) => T | Promise<T>
  ): Promise<NativeIpcResult<T>> {
    if (!trusted(event)) return { ok: false, error: 'failed' };
    if (service === undefined) return { ok: false, error: 'unavailable' };
    try {
      return { ok: true, value: await run(service) };
    } catch (error) {
      logFailure('Native service IPC failed', error);
      return { ok: false, error: 'failed' };
    }
  }

  function validateRequest(method: unknown, input: unknown) {
    if (typeof method !== 'string' || !(input instanceof Uint8Array)) {
      throw new TypeError('Invalid native service request');
    }
    return {
      method,
      input: Buffer.from(input.buffer, input.byteOffset, input.byteLength)
    };
  }

  function closeStream(id: number) {
    const session = streams.get(id);
    streams.delete(id);
    session?.stream.close();
  }

  function closeStreams() {
    for (const id of streams.keys()) closeStream(id);
  }

  contents.ipc.handle(
    'shinden-to-anilist:native-call',
    (event, method: unknown, input: unknown) =>
      handle(event, (native) => {
        const request = validateRequest(method, input);
        return native.call(request.method, request.input);
      })
  );
  contents.ipc.handle(
    'shinden-to-anilist:native-open-stream',
    (event, method: unknown, input: unknown) =>
      handle(event, (native) => {
        const request = validateRequest(method, input);
        const id = ++nextId;
        streams.set(id, {
          stream: native.openStream(request.method, request.input),
          reading: false
        });
        return id;
      })
  );
  contents.ipc.handle(
    'shinden-to-anilist:native-read-stream',
    (event, id: number) =>
      handle(event, async () => {
        const session = streams.get(id);
        if (session === undefined) return null;
        if (session.reading)
          throw new Error('A read is already pending for this native stream');

        session.reading = true;
        try {
          const reply = await session.stream.next();
          if (reply == null || reply.isError) closeStream(id);
          return reply ?? null;
        } catch (error) {
          closeStream(id);
          throw error;
        } finally {
          session.reading = false;
        }
      })
  );
  contents.ipc.handle(
    'shinden-to-anilist:native-close-stream',
    (event, id: number) => {
      if (trusted(event)) closeStream(id);
    }
  );

  contents.on('did-start-navigation', (event) => {
    if (event.isMainFrame && !event.isSameDocument) closeStreams();
  });
  contents.on(
    'did-frame-navigate',
    (_event, _url, _code, _status, isMainFrame) => {
      if (isMainFrame) closeStreams();
    }
  );
  contents.on('render-process-gone', closeStreams);
  contents.once('destroyed', closeStreams);
}
