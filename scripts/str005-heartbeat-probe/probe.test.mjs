import './client.test.mjs';
import './evidence.test.mjs';
import './recovery.test.mjs';
import './owner.test.mjs';
import './preparation.test.mjs';
import test from 'node:test';
import assert from 'node:assert/strict';
import { main } from './main.mjs';
test('real heartbeat entrypoint rejects hardware before looking at private inputs', async () => {
  await assert.rejects(main(['preflight', '--private-root', '/unreadable', '--preparation-root', '/unreadable',
    '--gate-root', '/unreadable', '--fixture-binary', '/unreadable']), /heartbeat_hardware_disabled/);
});
