import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, realpath, mkdir, writeFile, chmod, readFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { resolve } from 'node:path';
import { fileDigest } from '../fixed-usb-qualification/contract.mjs';
import { sha256 } from '../str005-v2-serial/values.mjs';
import { runClear, verifyClear } from './clear.mjs';

test('real child-process clear wrapper binds argv and verifies official result plus erased readback', async t => {
  // Arrange: the executable is a synthetic CLI, never espflash or a real device command.
  const parent = await realpath(await mkdtemp(resolve(tmpdir(), 'startup-clear-test-')));
  const root = resolve(parent, 'attempt'); await mkdir(root, { mode: 0o700 });
  t.after(() => rm(parent, { recursive: true, force: true })); await chmod(root, 0o700);
  const archive = Buffer.alloc(974848, 3), physical = 'd'.repeat(64);
  await writeFile(resolve(root, 'archive.bin'), archive, { mode: 0o600 });
  const binary = resolve(root, 'synthetic-cli'), script = resolve(root, 'synthetic-cli.cjs');
  const quote = value => "'" + value.replaceAll("'", "'\"'\"'") + "'";
  // Bazel's Node path can exceed the kernel shebang limit.
  await writeFile(binary, `#!/bin/sh\nexec ${quote(await realpath(process.env.JS_BINARY__NODE_BINARY ?? process.execPath))} ${quote(script)} "$@"\n`, { mode: 0o700 });
  await writeFile(script, `const fs = require('node:fs'), p = require('node:path');
const args=process.argv.slice(2), value=k=>args[args.indexOf(k)+1];
if(args[0]!=='core-dump-clear')process.exit(2);
const child=value('--private-root');fs.mkdirSync(child,{mode:0o700});
const put=(name,value)=>fs.writeFileSync(p.join(child,name),value,{mode:0o600,flag:'wx'});
put('core-dump.private.bin',fs.readFileSync(value('--preserved-dump')));
put('cleared-core-dump.private.bin',Buffer.alloc(974848,255));
put('result.private.json',JSON.stringify({schema_version:'bitaxe-development-core-dump-clear-v1',clearing_complete:true,
terminal_category:'complete',first_failure_stage:'complete',source_commit:'${'a'.repeat(40)}',rom_admitted:true,acquisition_complete:true,
application_identity_restored:true,hardware_baseline_verified:false,cleanup_complete:true,expected_installed_source:value('--expected-installed-source'),
expected_installed_elf:value('--expected-installed-elf')}));
`, { mode: 0o700 });
  const context = { source_commit: 'a'.repeat(40), firmware_commit: 'b'.repeat(40), app_elf_sha256: 'c'.repeat(64), gate_commit: 'e'.repeat(40),
    detector: { port: '/dev/test-only' }, physical, archiveSha: sha256(archive), flash_binary: binary, flash_sha256: await fileDigest(binary),
    bindings: { archiveRoot: root, archiveRelative: 'archive.bin' } };
  const recovery = { schema: 'str005-current-recovery-proof-v1', source_commit: context.source_commit, gate_commit: context.gate_commit,
    firmware_commit: context.firmware_commit, app_elf_sha256: context.app_elf_sha256, physical_identity_sha256: physical, observed_at_unix_ms: Date.now(),
    ledger: { schema: 'worker-qualification-ledger-v1', next_ordinal: 23, last_completed_ordinal: 22, total_charged_ms: 2160000, pending: false },
    original_budget: { schema: 'worker-budget-review-v1', campaign_match: true, reserved_mask: 7, completed_mask: 7, charged_ms: 240000, pending: false },
    safe_baseline: true, restoration_confirmed: true, device_lease_inactive: true, serial_ownership_released: true, preservation_matches: true,
    mine_on_boot: false, current_v2_idle: true };
  await writeFile(resolve(root, 'current-recovery.json'), JSON.stringify(recovery), { mode: 0o600 });
  await writeFile(resolve(parent, 'clear-detector.stdout.log'), `port: /dev/cu.synthetic\nphysical_identity_sha256: ${physical}\nusb_profile: serial_jtag_runtime\n`, { mode: 0o600 });
  // Act
  try { await runClear(root, context, { requireNoHolders: () => {} }); }
  catch (error) { throw new Error(await readFile(resolve(root, 'clear-command.stderr'), 'utf8'), { cause: error }); }
  const result = await verifyClear(root, context);
  // Assert
  assert.equal(result.clearedSha256, sha256(Buffer.alloc(974848, 255)));
  await assert.rejects(runClear(root, context));
  const changed = JSON.parse(await readFile(resolve(root, 'clear-launch.json'), 'utf8')); changed.arguments[0] = 'factory-reset';
  await writeFile(resolve(root, 'clear-launch.json'), JSON.stringify(changed), { mode: 0o600 });
  await assert.rejects(verifyClear(root, context));
});
