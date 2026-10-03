import { readFile, realpath } from 'node:fs/promises';
import { resolve } from 'node:path';
import { fileDigest } from '../fixed-usb-qualification/contract.mjs';
import { check, object, sha256 } from '../str005-v2-serial/values.mjs';

/** Provenance of the qualified passive cadence observer, built from this exact clean source. */
export async function observerContext(repo, source) {
  const path = await realpath(resolve(repo, 'bazel-bin/tools/http-transport/cadence_observer'));
  const receiptPath = resolve(repo, 'bazel-bin/tools/http-transport/v2-observer-build-identity.json');
  const receiptBytes = await readFile(receiptPath), built = JSON.parse(receiptBytes);
  object(built, ['schema', 'sourceCommit', 'sourceDirty', 'observerSha256', 'writerSha256']);
  check(built.schema === 'str005-v2-observer-build-v1' && built.sourceCommit === source.commit && built.sourceDirty === false &&
    built.observerSha256 === await fileDigest(path) &&
    built.writerSha256 === await fileDigest(resolve(repo, 'scripts/str005-v2-serial/observer-build-identity.mjs')),
  'heartbeat_observer_provenance');
  return { cadence_observer: { path, sha256: built.observerSha256 }, observer_build_receipt_sha256: sha256(receiptBytes),
    observer_receipt_path: receiptPath };
}
