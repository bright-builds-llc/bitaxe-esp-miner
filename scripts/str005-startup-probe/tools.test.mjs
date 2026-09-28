import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, mkdir, writeFile, rm, realpath } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { resolve, dirname } from 'node:path';
import { canonicalTools } from './tools.mjs';
import { sha256 } from '../str005-v2-serial/values.mjs';
test('caller executable hashes cannot substitute for canonical source-bound tools', async t => {
  const root = await realpath(await mkdtemp(resolve(tmpdir(), 'startup-tools-test-'))); t.after(() => rm(root, { recursive: true, force: true }));
  const put = async (relative, value) => { const path = resolve(root, relative); await mkdir(dirname(path), { recursive: true }); await writeFile(path, value); return path; };
  const flash = await put('bazel-bin/tools/flash/flash', 'synthetic-flash');
  const fixture = await put('bazel-bin/tools/stratum-v2-fixture/stratum_v2_fixture', 'synthetic-fixture');
  await put('scripts/str005-v2-serial/build-identity.mjs', 'synthetic-writer');
  const receipt = { schema: 'str005-v2-fixture-build-v1', sourceCommit: 'a'.repeat(40), sourceDirty: false,
    fixtureSha256: sha256('synthetic-fixture'), writerSha256: sha256('synthetic-writer') };
  await put('bazel-bin/tools/stratum-v2-fixture/v2-serial-build-identity.json', JSON.stringify(receipt));
  const options = { '--flash-binary': flash, '--fixture-binary': fixture };
  assert.equal((await canonicalTools(root, options, receipt.sourceCommit)).fixture_sha256, receipt.fixtureSha256);
  const fake = await put('caller-executable', 'synthetic-flash');
  await assert.rejects(canonicalTools(root, { ...options, '--flash-binary': fake }, receipt.sourceCommit), { code: 'startup_noncanonical_tool' });
  await assert.rejects(canonicalTools(root, options, 'b'.repeat(40)), { code: 'startup_fixture_provenance' });
});
