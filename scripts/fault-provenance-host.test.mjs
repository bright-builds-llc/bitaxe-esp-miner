import test from 'node:test';
import assert from 'node:assert/strict';
import { execFile } from 'node:child_process';
import { promisify } from 'node:util';
import { mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
const run = promisify(execFile);
const component = resolve(dirname(fileURLToPath(import.meta.url)), '../firmware/bitaxe/components/bitaxe_fault_provenance');
for (const source of ['panic_model_test.c', 'allocation_test.c']) {
  test(`production native fault model: ${source}`, async t => {
    // Arrange: only CPU/task-register access is simulated by the C fixture.
    const root = await mkdtemp(resolve(tmpdir(), 'bitaxe-fault-model-'));
    t.after(() => rm(root, { recursive: true, force: true }));
    const binary = resolve(root, 'model');
    await run('cc', ['-std=c11', '-Wall', '-Wextra', '-Werror', '-O2', resolve(component, source), '-o', binary], { timeout: 30000 });
    // Act
    const result = await run(binary, [], { timeout: 10000 });
    // Assert
    assert.equal(result.stderr, '');
  });
}
