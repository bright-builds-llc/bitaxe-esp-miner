import test from 'node:test';
import assert from 'node:assert/strict';
import { once } from 'node:events';
import { mkdtemp, readFile, realpath, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { createServerOwner } from '../str005-startup-probe/server.mjs';
import { routePolicy } from './policy.mjs';
const here = dirname(fileURLToPath(import.meta.url));
test('production server serves the focused page and its shared browser modules without effect access', async t => {
  // Arrange
  const root = await realpath(await mkdtemp(resolve(tmpdir(), 'status-repro-http-')));
  t.after(() => rm(root, { recursive: true, force: true }));
  const extras = new Map();
  for (const [path, file] of [['/base-startup-page.mjs','../str005-startup-probe/page.mjs'],
    ['/base-client.mjs','../str005-startup-probe/client.mjs'], ['/client-core.mjs','client-core.mjs']])
    extras.set(path, await readFile(resolve(here, file)));
  const server = await createServerOwner({ root, context: { firmware_root: '/unused' },
    assets: { page: '', bundle: Buffer.from(''), trust: {}, pageClient: await readFile(resolve(here, 'page.mjs')),
      coordinator: await readFile(resolve(here, 'client.mjs')) }, authorityDirectory: '/unused', verify: async () => {} },
  { routePolicy: routePolicy(), extraAssets: extras, admitContext: () => {} });
  server.listen(0, '127.0.0.1'); await once(server, 'listening');
  const origin = `http://127.0.0.1:${server.address().port}`;
  try {
    // Act / Assert
    for (const path of ['/', '/startup-page.mjs', '/client.mjs', '/client-core.mjs', '/base-startup-page.mjs', '/base-client.mjs']) {
      const response = await fetch(`${origin}${path}`); assert.equal(response.status, 200, path);
      assert.ok((await response.text()).length > 0, path);
    }
    const rejected = await fetch(`${origin}/status-repro/observe`, { method: 'POST',
      headers: { Origin: origin, 'Content-Type': 'application/json' }, body: '{}' });
    assert.equal(rejected.status, 400);
  } finally { await server.release(); }
});
