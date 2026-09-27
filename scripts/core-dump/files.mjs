import { constants } from 'node:fs';
import { lstat, open, realpath, stat } from 'node:fs/promises';
import { dirname, isAbsolute, parse, resolve } from 'node:path';
import { createHash } from 'node:crypto';

export const sha256 = bytes => createHash('sha256').update(bytes).digest('hex');
export function check(condition, code) { if (!condition) throw new Error(code); }

/** Reject symlinks in every existing path component, including ancestor directories. */
export async function plainPath(path) {
  check(isAbsolute(path) && resolve(path) === path, 'absolute_path_required');
  let current = path;
  while (current !== parse(current).root) {
    check(!(await lstat(current)).isSymbolicLink(), 'symlink_rejected');
    current = dirname(current);
  }
  check(await realpath(path) === path, 'symlink_rejected');
}

/** Copy a stable, exclusively linked file through a no-follow descriptor. */
export async function snapshot(source, destination, sensitive) {
  await plainPath(source);
  const input = await open(source, constants.O_RDONLY | constants.O_NOFOLLOW);
  try {
    const before = await input.stat();
    check(before.isFile() && before.nlink === 1 && before.size > 0 && before.size <= 128 * 1024 * 1024, 'input_file');
    check(!sensitive || (before.mode & 0o077) === 0, 'input_not_private');
    const bytes = await input.readFile();
    const after = await input.stat();
    check(before.size === after.size && before.mtimeMs === after.mtimeMs && before.ctimeMs === after.ctimeMs, 'input_changed');
    const output = await open(destination, 'wx', 0o600);
    try { await output.writeFile(bytes); await output.sync(); } finally { await output.close(); }
    return sha256(bytes);
  } finally { await input.close(); }
}

/** Require an owner-only directory before persisting or reading raw diagnostics. */
export async function privateDirectory(path) {
  await plainPath(path);
  const info = await stat(path);
  check(info.isDirectory() && (info.mode & 0o777) === 0o700 && info.uid === process.getuid(), 'private_parent_required');
}
