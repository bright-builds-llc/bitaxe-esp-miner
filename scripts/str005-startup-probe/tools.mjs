import { readFile, realpath } from 'node:fs/promises';
import { resolve } from 'node:path';
import { fileDigest } from '../fixed-usb-qualification/contract.mjs';
import { check, object } from '../str005-v2-serial/values.mjs';
/** Resolve only the fixture capability needed by prepared startup; no flash capability is acquired. */
export async function canonicalFixture(root, requested, source) {
  const fixture = await realpath(resolve(root, 'bazel-bin/tools/stratum-v2-fixture/stratum_v2_fixture'));
  check(await realpath(requested) === fixture, 'startup_noncanonical_tool');
  const receiptPath = resolve(root, 'bazel-bin/tools/stratum-v2-fixture/v2-serial-build-identity.json');
  const receipt = JSON.parse(await readFile(receiptPath, 'utf8'));
  object(receipt, ['schema', 'sourceCommit', 'sourceDirty', 'fixtureSha256', 'writerSha256']);
  const fixtureHash = await fileDigest(fixture);
  check(receipt.schema === 'str005-v2-fixture-build-v1' && receipt.sourceCommit === source && receipt.sourceDirty === false &&
    receipt.fixtureSha256 === fixtureHash && receipt.writerSha256 === await fileDigest(resolve(root, 'scripts/str005-v2-serial/build-identity.mjs')),
  'startup_fixture_provenance');
  return { fixture_binary: fixture, fixture_sha256: fixtureHash,
    fixture_receipt_path: receiptPath, fixture_receipt_sha256: await fileDigest(receiptPath) };
}

/** Historical clear-bound tool admission remains available to its regression tests. */
export async function canonicalTools(root, options, source) {
  const flash = await realpath(resolve(root, 'bazel-bin/tools/flash/flash'));
  check(await realpath(options['--flash-binary']) === flash, 'startup_noncanonical_tool');
  return { ...await canonicalFixture(root, options['--fixture-binary'], source), flash_binary: flash, flash_sha256: await fileDigest(flash) };
}
