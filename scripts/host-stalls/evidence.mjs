import { createHash } from 'node:crypto';
import { closeSync, constants, lstatSync, mkdirSync, openSync, realpathSync, writeFileSync, writeSync } from 'node:fs';
import path from 'node:path';

export const OVERRIDE_KEYS = new Set(['PATH', 'CARGO_TARGET_DIR', 'RUSTUP_TOOLCHAIN', 'CARGO_BUILD_JOBS', 'CARGO_PROFILE_DEV_DEBUG', 'CARGO_PROFILE_TEST_DEBUG']);

/** Create a one-use private evidence directory without following any symlink. */
export function createEvidenceRoot(root) {
  if (!path.isAbsolute(root)) throw new Error('evidence_root_must_be_absolute');
  const parent = path.dirname(root);
  for (let entry = parent; ; entry = path.dirname(entry)) {
    if (lstatSync(entry).isSymbolicLink()) throw new Error('evidence_parent_symlink');
    if (entry === path.dirname(entry)) break;
  }
  const info = lstatSync(parent);
  if (!info.isDirectory() || (info.mode & 0o777) !== 0o700) throw new Error('evidence_parent_requires_0700');
  if (info.uid !== process.getuid()) throw new Error('evidence_parent_wrong_owner');
  mkdirSync(root, { mode: 0o700 });
  return realpathSync(root);
}

export function writeEvidence(root, name, value) {
  if (path.basename(name) !== name) throw new Error('invalid_evidence_name');
  writeFileSync(path.join(root, name), typeof value === 'string' ? value : `${JSON.stringify(value, null, 2)}\n`, { flag: 'wx', mode: 0o600 });
}

export function environmentMetadata(env) {
  const recorded = {};
  for (const key of OVERRIDE_KEYS) {
    const maybeValue = env[key];
    if (maybeValue === undefined) continue;
    if (key === 'PATH') recorded.PATH_sha256 = createHash('sha256').update(maybeValue).digest('hex');
    else recorded[key] = maybeValue;
  }
  return recorded;
}

export function outputSink(root, name, limit) {
  const fd = openSync(path.join(root, name), constants.O_CREAT | constants.O_EXCL | constants.O_WRONLY | constants.O_NOFOLLOW, 0o600);
  const result = { totalBytes: 0, storedBytes: 0, truncated: false };
  return {
    result,
    append(chunk) {
      result.totalBytes += chunk.length;
      const keep = chunk.subarray(0, Math.max(0, limit - result.storedBytes));
      let written = 0;
      while (written < keep.length) written += writeSync(fd, keep, written, keep.length - written);
      result.storedBytes += written;
      result.truncated = result.totalBytes > result.storedBytes;
    },
    close() { closeSync(fd); },
  };
}
