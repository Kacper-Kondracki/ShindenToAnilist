import assert from 'node:assert/strict';
import { test } from 'node:test';
import { Code, ConnectError } from '@connectrpc/connect';
import { setTrailerStatus } from '@connectrpc/connect/protocol-grpc';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import { createServer } from 'vite';

function grpcWebTrailers(error) {
  const headers = setTrailerStatus(new Headers(), error);
  const trailer = Buffer.from(
    Array.from(headers, ([key, value]) => `${key}: ${value}\r\n`).join('')
  );
  const frame = Buffer.alloc(5);
  frame[0] = 0x80;
  frame.writeUInt32BE(trailer.length, 1);
  return new Response(Buffer.concat([frame, trailer]), {
    headers: { 'content-type': 'application/grpc-web+proto' }
  });
}

test('desktop transports preserve failure kinds and developer diagnostics', async (t) => {
  const vite = await createServer({
    configFile: false,
    plugins: [svelte({ prebundleSvelteLibraries: false })],
    optimizeDeps: { noDiscovery: true },
    environments: { ssr: { optimizeDeps: { noDiscovery: true } } },
    server: { middlewareMode: true, ws: false, watch: null }
  });
  t.after(() => vite.close());

  const runtime = await vite.ssrLoadModule('/src/lib/api/runtime.ts');
  const { AppFailure, ErrorKind, userErrorMessage } = await vite.ssrLoadModule(
    '/src/lib/api/errors.ts'
  );
  const { AppErrorSchema } = await vite.ssrLoadModule(
    '/src/lib/gen/shinden_to_anilist/v1/error_pb.ts'
  );
  const grpc = await vite.ssrLoadModule('/src/lib/api/grpcService.ts');

  const failure = { kind: ErrorKind.DATABASE_MISSING_RELEASE_ASSET };
  const grpcError = new ConnectError(
    'ERROR_KIND_DATABASE_MISSING_RELEASE_ASSET',
    Code.FailedPrecondition,
    undefined,
    [{ desc: AppErrorSchema, value: failure }]
  );

  await t.test(
    'Tauri and gRPC-Web produce the same database message',
    async (t) => {
      t.mock.method(globalThis, 'fetch', async () =>
        grpcWebTrailers(grpcError)
      );
      globalThis.window = {
        __TAURI_INTERNALS__: {
          invoke: async () => {
            throw failure;
          }
        }
      };
      t.after(() => delete globalThis.window);

      const rejected = async (promise) => {
        const [result] = await Promise.allSettled([promise]);
        assert.equal(result.status, 'rejected');
        assert.ok(result.reason instanceof AppFailure);
        assert.equal(result.reason.kind, failure.kind);
        return result.reason;
      };

      const rpcFailure = await rejected(grpc.downloadDatabase('/tmp/example'));
      const tauriFailure = await rejected(
        runtime.callTauri('download_database')
      );
      assert.equal(rpcFailure.cause.code, Code.FailedPrecondition);
      assert.equal(tauriFailure.cause, failure);
      assert.equal(
        userErrorMessage(rpcFailure, ''),
        userErrorMessage(tauriFailure, '')
      );
      assert.match(userErrorMessage(rpcFailure, ''), /pliku bazy anime/);
    }
  );

  await t.test(
    'Electron directory failures use the same public contract',
    async (t) => {
      globalThis.shindenToAnilist = {
        getAppPaths: async () => ({ ok: false, error: 'appPathsFailed' })
      };
      t.after(() => delete globalThis.shindenToAnilist);

      await assert.rejects(runtime.resolveAppPaths(), (error) => {
        assert.ok(error instanceof AppFailure);
        assert.equal(error.kind, ErrorKind.APP_PATHS_FAILED);
        return true;
      });
    }
  );

  await t.test(
    'renderer diagnostics forward new failures and redact credentials',
    async (t) => {
      const { reportFailure } = await vite.ssrLoadModule(
        '/src/lib/api/diagnostics.ts'
      );
      const entries = [];
      globalThis.shindenToAnilist = {
        reportFailure: (operation, detail) =>
          entries.push({ operation, detail })
      };
      t.after(() => delete globalThis.shindenToAnilist);

      reportFailure(
        'load_database',
        new AppFailure(ErrorKind.DATABASE_IO, undefined, undefined, true)
      );
      reportFailure('source_import', new AppFailure(ErrorKind.CANCELLED));
      reportFailure(
        'source_import',
        new Error('https://example.test/list?secret=token', {
          cause: new Error('cf_clearance=sensitive-value')
        })
      );

      assert.equal(entries.length, 1);
      assert.equal(entries[0].operation, 'source_import');
      assert.match(entries[0].detail, /Caused by:/);
      assert.match(entries[0].detail, /cf_clearance=\[redacted\]/);
      assert.doesNotMatch(entries[0].detail, /secret=token|sensitive-value/);
    }
  );

  await t.test(
    'local exceptions stay intact and use a safe fallback',
    async () => {
      const error = new TypeError('EACCES: /tmp/example', {
        cause: new Error('permission denied')
      });
      await assert.rejects(
        runtime.callRpc(async () => {
          throw error;
        }),
        (caught) => caught === error
      );
      assert.equal(
        userErrorMessage(error, 'Operacja nie powiodła się.'),
        'Operacja nie powiodła się.'
      );
    }
  );

  await t.test(
    'an incomplete stream cannot report a successful import',
    async (t) => {
      t.mock.method(globalThis, 'fetch', async () => grpcWebTrailers());
      await assert.rejects(
        grpc.fetchSourceList(1, '123'),
        /ended without a valid completion event/
      );
    }
  );

  await t.test(
    'failed import after verification keeps its own recovery path',
    async () => {
      const { createShindenCloudflareController } = await vite.ssrLoadModule(
        '/src/lib/features/shinden/cloudflareController.svelte.ts'
      );
      const clearance = { cfClearance: 'test' };
      let saved = false;
      let outcome = 'failed';
      let closeVerification = false;
      const controller = createShindenCloudflareController({
        openVerification: async () => {
          if (closeVerification) throw new AppFailure(ErrorKind.CANCELLED);
          return clearance;
        },
        applyClearance: async () => ({ accepted: true }),
        retry: async () => outcome,
        saveClearance: () => {
          saved = true;
        }
      });
      const request = {
        provider: 'shinden',
        query: '123',
        sourceUser: {},
        message: 'Weryfikacja Shindena'
      };

      controller.block(request);
      await controller.resolve();
      assert.equal(controller.state.status, 'idle');
      assert.equal(saved, true);

      outcome = 'challenge';
      saved = false;
      controller.block(request);
      await controller.resolve();
      assert.equal(controller.state.status, 'failed');
      assert.match(controller.state.message, /nadal wymaga weryfikacji/);
      assert.equal(saved, false);

      closeVerification = true;
      await controller.resolve();
      assert.equal(controller.state.status, 'blocked');
      assert.match(controller.state.message, /weryfikacja została przerwana/i);
      assert.equal(saved, false);
    }
  );
  await t.test(
    'Shinden retries preserve database failures and skip stale clearance',
    async (t) => {
      let rejectedByShinden = false;
      const tauri = {
        transformCallback: () => 1,
        invoke: async (command) => {
          assert.equal(command, 'fetch_source_list');
          if (rejectedByShinden) throw { kind: ErrorKind.CLOUDFLARE_CHALLENGE };
          return { sourceVersion: '1' };
        }
      };
      globalThis.window = { __TAURI_INTERNALS__: tauri };
      globalThis.__TAURI_INTERNALS__ = tauri;
      t.after(() => {
        delete globalThis.window;
        delete globalThis.__TAURI_INTERNALS__;
      });

      const { createSourceImportController } = await vite.ssrLoadModule(
        '/src/lib/features/source/sourceImportController.svelte.ts'
      );
      let challengeCount = 0;
      let appliedSavedClearance = 0;
      const databaseMessage = 'Plik bazy anime jest uszkodzony.';
      const controller = createSourceImportController({
        animeData: {},
        workspace: {},
        notifications: { error: () => {} },
        waitForReadyDatabase: async () => ({
          status: 'error',
          message: databaseMessage
        }),
        applySavedShindenVerification: async () => {
          appliedSavedClearance += 1;
        },
        onShindenChallenge: () => {
          challengeCount += 1;
        }
      });
      const user = { provider: 1, user: '123' };

      assert.equal(
        await controller.loadParsedSourceUser('shinden', '123', user, {
          afterVerification: true
        }),
        'failed'
      );
      assert.equal(controller.state.message, databaseMessage);
      assert.equal(appliedSavedClearance, 0);
      assert.equal(challengeCount, 0);

      rejectedByShinden = true;
      assert.equal(
        await controller.loadParsedSourceUser('shinden', '123', user),
        'challenge'
      );
      assert.equal(appliedSavedClearance, 1);
      assert.equal(challengeCount, 1);
    }
  );
});
