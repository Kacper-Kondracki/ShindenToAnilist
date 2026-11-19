import { execFileSync } from 'node:child_process';
import { writeFileSync } from 'node:fs';

let revision = 'unknown';
try {
  revision = execFileSync('git', ['rev-parse', '--short=12', 'HEAD'], {
    encoding: 'utf8',
    stdio: ['ignore', 'pipe', 'ignore']
  }).trim();
} catch {
  // Source archives can be built without Git.
}
writeFileSync(
  new URL('../electron/dist/build-info.json', import.meta.url),
  JSON.stringify({ revision })
);
