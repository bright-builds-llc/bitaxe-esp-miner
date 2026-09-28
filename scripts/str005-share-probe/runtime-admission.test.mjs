import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, mkdir, rm, writeFile, realpath } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { resolve } from 'node:path';
import { beginAdmissionVerification, verifyPinnedInventories } from './runtime-admission.mjs';
import { inventory, writeNew, proof } from '../str005-noise-serial/files.mjs';
import { STARTUP_SEAL } from './contract.mjs';

test('runtime inventory checks permit current ownership without re-running offline admission and reject changed bytes', async () => {
  // Arrange: real private producer files; only the published historical pin and offline hardware admission are doubled.
  const root = await realpath(await mkdtemp(resolve(tmpdir(), 'share-runtime-'))); let held = false, admissions = 0;
  try {
    const pins = [];
    for (const name of ['install', 'startup', 'capture', 'clear', 'recovery']) {
      const child = resolve(root, name); await mkdir(child, { mode: 0o700 });
      await writeNew(resolve(child, 'result.json'), { name, complete: true });
      await writeNew(resolve(child, 'sealed-inventory.json'), { files: await inventory(child) });
      pins.push([child, (await proof(child, 'sealed-inventory.json')).sha256]);
    }
    const input = { installationRoot: pins[0][0], installationSeal: pins[0][1], startupRoot: pins[1][0],
      captureBindings: { captureRoot: pins[2][0] }, captureSeal: pins[2][1], clearRoot: pins[3][0], clearSeal: pins[3][1],
      recoveryRoot: pins[4][0], recoverySeal: pins[4][1] };
    const verify = await beginAdmissionVerification(root, input, {
      admit: async () => { admissions++; assert.equal(held, false, 'offline no-holder admission must precede ownership'); },
      verifyInventories: async values => {
        assert.equal(values[1][1], STARTUP_SEAL);
        await verifyPinnedInventories(values.map((entry, index) => index === 1 ? pins[1] : entry));
      },
    });
    // Act: the live page now owns USB; immutable checks still succeed without offline admission.
    held = true; await verify(); await verify();
    input.installationSeal = 'f'.repeat(64); await verify(); // Caller mutation cannot replace the captured pins.
    await writeFile(resolve(pins[4][0], 'result.json'), '{"complete":false}\n', { mode: 0o600 });
    // Assert
    await assert.rejects(verify(), { code: 'noise_inventory_changed' });
    assert.equal(admissions, 1);
  } finally { await rm(root, { recursive: true, force: true }); }
});
