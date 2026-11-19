import assert from 'node:assert/strict';
import { mkdtemp, rm, writeFile } from 'node:fs/promises';
import { createRequire } from 'node:module';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { test } from 'node:test';
import { fileURLToPath } from 'node:url';
import { create, fromBinary, toBinary } from '@bufbuild/protobuf';
import { createServer } from 'vite';

test('native addon loads database snapshots and streams protobuf over Node-API', async (t) => {
  const dir = await mkdtemp(join(tmpdir(), 'shinden-native-test-'));
  const vite = await createServer({
    root: fileURLToPath(new URL('..', import.meta.url)),
    configFile: false,
    optimizeDeps: { noDiscovery: true },
    environments: { ssr: { optimizeDeps: { noDiscovery: true } } },
    server: { middlewareMode: true, ws: false, watch: null }
  });
  t.after(async () => {
    await vite.close();
    await rm(dir, { recursive: true, force: true });
  });

  const { ShindenToAnilistService } = await vite.ssrLoadModule(
    '/src/lib/gen/shinden_to_anilist/v1/service_pb.ts'
  );
  const { AppErrorSchema, ErrorKind } = await vite.ssrLoadModule(
    '/src/lib/gen/shinden_to_anilist/v1/error_pb.ts'
  );
  const methods = ShindenToAnilistService.method;
  const addon =
    process.argv[2] ??
    fileURLToPath(
      new URL(
        '../../electron/build/native/shinden_to_anilist.node',
        import.meta.url
      )
    );
  process.env.XDG_STATE_HOME = dir;
  const { NativeService } = createRequire(import.meta.url)(resolve(addon));
  const service = new NativeService();
  const encode = (method, fields) =>
    Buffer.from(toBinary(method.input, create(method.input, fields)));
  const call = async (method, fields) => {
    const reply = await service.call(method.name, encode(method, fields));
    assert.equal(reply.isError, false);
    return fromBinary(method.output, reply.data);
  };
  const open = (method, fields) =>
    service.openStream(method.name, encode(method, fields));
  const read = async (stream, method) => {
    const reply = await stream.next();
    if (reply == null) return null;
    assert.equal(reply.isError, false);
    return fromBinary(method.output, reply.data);
  };

  const failure = await service.call(
    methods.getDatabaseIds.name,
    Buffer.alloc(0)
  );
  assert.equal(failure.isError, true);
  assert.equal(
    fromBinary(AppErrorSchema, failure.data).kind,
    ErrorKind.DATABASE_NOT_LOADED
  );

  const maxId = (1n << 64n) - 1n;
  const path = join(dir, 'database.jsonl');
  const header = JSON.stringify({ lastUpdate: '2026-11-01' });
  function entry(id) {
    return JSON.stringify({
      sources: id === undefined ? [] : [`https://myanimelist.net/anime/${id}`],
      title: `Anime ${id ?? 'without MAL ID'}`,
      type: 'TV',
      episodes: 12,
      status: 'FINISHED',
      animeSeason: { season: 'UNDEFINED' },
      picture: '',
      thumbnail: '',
      synonyms: [],
      studios: [],
      producers: [],
      relatedAnime: [],
      tags: []
    });
  }
  await writeFile(
    path,
    [
      header,
      ...Array.from({ length: 500 }, (_, i) => entry(i + 1)),
      entry(maxId)
    ].join('\n')
  );
  const { databaseVersion } = await call(methods.loadDatabase, { path });
  assert.equal(databaseVersion, 1n);
  const full = open(methods.getDatabaseFull);
  const first = await read(full, methods.getDatabaseFull);
  const last = await read(full, methods.getDatabaseFull);
  assert.equal(first.entries.length, 500);
  assert.equal(last.entries.length, 1);
  assert.equal(last.entries[0].id, maxId);
  assert.equal(first.databaseVersion, databaseVersion);
  assert.equal(last.databaseVersion, databaseVersion);
  assert.equal(await read(full, methods.getDatabaseFull), null);

  const cancelled = open(methods.getDatabaseFull);
  assert.equal(
    (await read(cancelled, methods.getDatabaseFull)).entries.length,
    500
  );
  cancelled.close();
  assert.equal(await read(cancelled, methods.getDatabaseFull), null);

  const invalid = open(methods.fetchSourceList, {});
  const error = await invalid.next();
  assert.equal(error.isError, true);
  assert.equal(
    fromBinary(AppErrorSchema, error.data).kind,
    ErrorKind.INVALID_SOURCE
  );
  assert.equal(await read(invalid, methods.fetchSourceList), null);

  await writeFile(path, [header, entry()].join('\n'));
  const emptyVersion = (await call(methods.loadDatabase, { path }))
    .databaseVersion;
  const empty = open(methods.getDatabaseFull);
  const emptyBatch = await read(empty, methods.getDatabaseFull);
  assert.equal(emptyBatch.databaseVersion, emptyVersion);
  assert.deepEqual(emptyBatch.entries, []);
  assert.equal(await read(empty, methods.getDatabaseFull), null);
});
