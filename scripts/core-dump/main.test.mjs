import test from 'node:test';
import assert from 'node:assert/strict';
import { execFileSync, spawnSync } from 'node:child_process';
import { chmod, link, mkdir, mkdtemp, readFile, realpath, rm, stat, symlink, writeFile } from 'node:fs/promises';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { fixture } from './fixture.mjs';
import { privateProcess } from './process.mjs';
import { managedGdb } from './main.mjs';
import { snapshot } from './files.mjs';

const source = dirname(fileURLToPath(import.meta.url));
const repo = execFileSync('git', ['rev-parse', '--show-toplevel'], { encoding: 'utf8' }).trim();
async function workspace(t) {
  await mkdir(join(repo, 'scratch'), { recursive: true });
  const root = await realpath(await mkdtemp(join(repo, 'scratch/core-test-')));
  await chmod(root, 0o700); t.after(() => rm(root, { recursive: true, force: true })); return root;
}
async function inputs(root, kind = 'valid') {
  const f = fixture(kind);
  await writeFile(join(root, 'dump'), f.dump, { mode: 0o600 }); await writeFile(join(root, 'elf'), f.program, { mode: 0o600 });
  return ['inspect', '--dump', join(root, 'dump'), '--elf', join(root, 'elf'), '--elf-sha256', f.identity, '--private-root', join(root, 'result')];
}
function run(args) { return spawnSync(process.execPath, [join(source, 'main.mjs'), ...args], { cwd: repo, encoding: 'utf8' }); }

test('raw synthetic core inspects through real pinned vendor loader with private artifacts', async t => {
  const root = await workspace(t), args = await inputs(root);
  const result = run(args); assert.equal(result.status, 0, result.stderr);
  const summary = JSON.parse(result.stdout); assert.equal(summary.full_elf_identity_verified, true);
  assert.equal(summary.cause_proven, false); assert.equal((await stat(join(root, 'result'))).mode & 0o777, 0o700);
  for (const file of ['dump.raw', 'firmware.elf', 'inputs.json', 'inspection.json', 'inspect.stdout', 'inspect.stderr'])
    assert.equal((await stat(join(root, 'result', file))).mode & 0o777, 0o600);
});
for (const kind of ['wrong', 'absent', 'truncated', 'duplicate', 'checksum', 'short-abi', 'unterminated']) test(`rejects ${kind} identity/checksum through real decoder`, async t => {
  const root = await workspace(t); const result = run(await inputs(root, kind));
  assert.equal(result.status, 1); assert.equal(JSON.parse(result.stderr).category, 'decoder_failed');
  assert.equal(result.stdout, '');
});
test('wrong caller ELF identity fails before decoder launch', async t => {
  const root = await workspace(t), args = await inputs(root); args[6] = '0'.repeat(64);
  assert.equal(JSON.parse(run(args).stderr).category, 'elf_identity');
});
test('existing private root fails without changing it', async t => {
  const root = await workspace(t), args = await inputs(root); await mkdir(join(root, 'result'));
  const result = run(args); assert.equal(result.status, 1);
});
test('nonprivate raw input is rejected', async t => {
  const root = await workspace(t), args = await inputs(root); await chmod(join(root, 'dump'), 0o644);
  assert.equal(JSON.parse(run(args).stderr).category, 'input_not_private');
});
for (const kind of ['symlink', 'hardlink']) test(`rejects ${kind} input`, async t => {
  const root = await workspace(t); await inputs(root);
  if (kind === 'symlink') await symlink(join(root, 'dump'), join(root, 'alias'));
  else await link(join(root, 'dump'), join(root, 'alias'));
  await assert.rejects(snapshot(join(root, 'alias'), join(root, 'copy'), true));
});
test('subprocess failure keeps sensitive stderr private', async t => {
  const root = await workspace(t);
  await assert.rejects(privateProcess(process.execPath, ['-e', 'console.error("synthetic-sensitive");process.exit(9)'], root, 'failed', process.env), /decoder_failed/);
  assert.match(await readFile(join(root, 'failed.stderr'), 'utf8'), /synthetic-sensitive/);
  assert.equal((await stat(join(root, 'failed.stderr'))).mode & 0o777, 0o600);
});
test('subprocess timeout releases its process group and preserves partial output', async t => {
  const root = await workspace(t);
  await assert.rejects(privateProcess(process.execPath, ['-e', 'console.log("partial");setInterval(()=>{},1000)'], root, 'timeout', process.env, 250), /decoder_timeout/);
  assert.match(await readFile(join(root, 'timeout.stdout'), 'utf8'), /partial/);
});

test('unrecognized serial arguments never reach decoder', async t => {
  // Arrange
  const root = await workspace(t), args = await inputs(root);
  // Act
  const result = run([...args, '--port', '/dev/never-open']);
  // Assert
  assert.equal(JSON.parse(result.stderr).category, 'arguments_invalid');
});
test('unignored private output is rejected before acquisition', async t => {
  // Arrange
  const root = await workspace(t), args = await inputs(root);
  const unignored = await mkdtemp(join(repo, 'core-test-unignored-')); await chmod(unignored, 0o700);
  t.after(() => rm(unignored, { recursive: true, force: true })); args[8] = join(unignored, 'result');
  // Act
  const result = run(args);
  // Assert
  assert.equal(JSON.parse(result.stderr).category, 'private_root_not_ignored');
});

test('timeout kills decoder descendants as well as immediate process', async t => {
  // Arrange
  const root = await workspace(t);
  const script = `const {spawn}=require('node:child_process');const fs=require('node:fs');
    const child=spawn(process.execPath,['-e','setInterval(()=>{},1000)'],{stdio:'ignore'});
    fs.writeFileSync('child.pid',String(child.pid));setInterval(()=>{},1000);`;
  // Act
  await assert.rejects(privateProcess(process.execPath, ['-e', script], root, 'descendant', process.env, 250), /decoder_timeout/);
  const pid = Number(await readFile(join(root, 'child.pid'), 'utf8'));
  await new Promise(resolve => setTimeout(resolve, 50));
  // Assert
  assert.throws(() => process.kill(pid, 0), { code: 'ESRCH' });
});

for (const kind of ['input', 'output']) test(`rejects permissive ${kind} parent`, async t => {
  // Arrange
  const root = await workspace(t), args = await inputs(root);
  const permissive = join(root, 'permissive'); await mkdir(permissive); await chmod(permissive, 0o755);
  if (kind === 'output') args[8] = join(permissive, 'result');
  else { await writeFile(join(permissive, 'dump'), fixture().dump, { mode: 0o600 }); args[2] = join(permissive, 'dump'); }
  // Act
  const result = run(args);
  // Assert
  assert.equal(JSON.parse(result.stderr).category, 'private_parent_required');
});

async function fakeManagedGdb(root, withRecommended) {
  const manifestRoot = join(root, '.embuild/espressif/esp-idf/v5.5.4/tools'); await mkdir(manifestRoot, { recursive: true });
  await writeFile(join(manifestRoot, 'tools.json'), JSON.stringify({ tools: [{ name: 'xtensa-esp-elf-gdb', versions: [
    { name: '16.3_20250913', status: 'recommended' }, { name: '99.0_future', status: 'supported' }] }] }));
  for (const version of withRecommended ? ['16.3_20250913', '99.0_future'] : ['99.0_future']) {
    const tool = join(root, '.embuild/espressif/tools/xtensa-esp-elf-gdb', version, 'xtensa-esp-elf-gdb/bin');
    await mkdir(tool, { recursive: true }); await writeFile(join(tool, 'xtensa-esp32s3-elf-gdb'), version);
  }
}

test('GDB selection ignores a lexically newer installation', async t => {
  // Arrange
  const root = await workspace(t); await fakeManagedGdb(root, true);
  // Act
  const selected = await managedGdb(root);
  // Assert
  assert.equal(selected.version, '16.3_20250913'); assert.match(selected.path, /16\.3_20250913/);
});

test('GDB selection blocks when exact recommended version is absent', async t => {
  // Arrange
  const root = await workspace(t); await fakeManagedGdb(root, false);
  // Act / Assert
  await assert.rejects(managedGdb(root), /managed_gdb_missing/);
});

test('analyze invokes real managed GDB with explicit private core and retains synthetic failure', async t => {
  // Arrange: this identity-only synthetic core deliberately has no crashed task registers.
  const root = await workspace(t), args = await inputs(root); args[0] = 'analyze';
  // Act
  const result = run(args);
  // Assert: the tool must reach GDB; this is not a claim of real crash analysis.
  assert.equal(result.status, 1); assert.equal(JSON.parse(result.stderr).category, 'decoder_failed');
  const receipt = JSON.parse(await readFile(join(root, 'result/analysis-inputs.json'), 'utf8'));
  assert.equal(receipt.gdb_version, '16.3_20250913'); assert.match(receipt.gdb_sha256, /^[0-9a-f]{64}$/);
  assert.equal(receipt.explicit_core, true); assert.equal(receipt.core_format, 'verified-elf');
  assert.equal(receipt.frame_arguments, 'none'); assert.equal(receipt.bounded_ms, 60000);
  assert.match(receipt.core_elf_sha256, /^[0-9a-f]{64}$/);
  assert.ok((await readFile(join(root, 'result/analysis.stdout'))).length + (await readFile(join(root, 'result/analysis.stderr'))).length > 0);
  assert.equal((await stat(join(root, 'result/core.elf'))).mode & 0o777, 0o600);
  assert.equal((await stat(join(root, 'result/analysis.stderr'))).mode & 0o777, 0o600);
  assert.equal(result.stdout, '');
});

test('native IDF 5.5.4 descriptor ABI matches host C layout and fixture', async t => {
  // Arrange
  const root = await workspace(t);
  const sourceText = await readFile(join(repo, '.embuild/espressif/esp-idf/v5.5.4/components/espcoredump/src/core_dump_elf.c'), 'utf8');
  assert.match(sourceText, /#define ELF_APP_SHA256_SIZE 66/);
  assert.match(sourceText, /uint32_t\s+version;[^\n]*\n\s+uint8_t\s+app_elf_sha256\[ELF_APP_SHA256_SIZE\];/);
  assert.match(sourceText, /core_dump_elf_t self = \{ 0 \};/);
  assert.match(sourceText, /sizeof\(self->elf_version_info\)\);/);
  // Act
  execFileSync('cc', ['-std=c11', '-Wall', '-Werror', join(source, 'abi-layout.c'), '-o', join(root, 'abi-layout')]);
  const layout = execFileSync(join(root, 'abi-layout'), { encoding: 'utf8' });
  // Assert
  assert.equal(layout, '72 4 66\n');
  const raw = fixture().dump;
  assert.equal(raw.readUInt32LE(24 + 84 + 4), 72);
});

test('native descriptor spare and alignment padding are not treated as identity', async t => {
  // Arrange
  const root = await workspace(t), args = await inputs(root, 'padding');
  // Act
  const result = run(args);
  // Assert
  assert.equal(result.status, 0, result.stderr);
  assert.equal(JSON.parse(result.stdout).full_elf_identity_verified, true);
});

test('analysis uses bounded offline batch frames over the deterministic verified core ELF', async () => {
  const { analysisArguments } = await import('./analysis.mjs');
  const args = analysisArguments('/private/program.elf', '/private/core.elf');
  assert.ok(args.includes('--nx')); assert.ok(args.includes('--batch')); assert.ok(args.includes('--core=/private/core.elf'));
  assert.ok(args.includes('set print frame-arguments none')); assert.ok(args.includes('thread apply all bt 40'));
  assert.equal(args.some(arg => /remote|info_corefile|bt full|print .*memory/u.test(arg)), false);
});
