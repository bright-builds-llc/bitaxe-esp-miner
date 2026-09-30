import { execFileSync } from 'node:child_process';
import { readFile } from 'node:fs/promises';
import { join } from 'node:path';
import { createHash } from 'node:crypto';

/** Bind compiled Rust input closures, including new uncommitted sources during observation mode. */
export async function sourceSnapshot(repo) {
  const files = execFileSync('git', ['ls-files', '-c', '-o', '--exclude-standard', '-z'], { cwd: repo, encoding: 'utf8' })
    .split('\0').filter(file => file && !file.startsWith('reference/') &&
      (/^(crates|firmware|tools\/stratum-v2-fixture)\//.test(file) && /\.(rs|toml|csv)$/.test(file) || ['Cargo.toml', 'Cargo.lock', '.cargo/config.toml', 'firmware/bitaxe/sdkconfig.defaults'].includes(file)))
    .sort();
  const hash = createHash('sha256');
  const inputs = [];
  for (const file of new Set(files)) {
    hash.update(file); hash.update('\0');
    let content;
    try { content = await readFile(join(repo, file)); }
    catch (error) { if (error.code !== 'ENOENT') throw error; content = Buffer.from('deleted'); }
    hash.update(content); inputs.push({ path: file, sha256: createHash('sha256').update(content).digest('hex') });
    hash.update('\0');
  }
  return { sha256: hash.digest('hex'), inputs };
}

export async function sourceDigest(repo) { return (await sourceSnapshot(repo)).sha256; }
