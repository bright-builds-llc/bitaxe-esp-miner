import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { BUNDLE, PAGE, fileDigest } from '../fixed-usb-qualification/contract.mjs';
import { proof, privateRoot, verifyInventory } from '../str005-noise-serial/files.mjs';
import { check, sha256 } from '../str005-v2-serial/values.mjs';
import { validateLedger } from '../fixed-usb-qualification/iterative-contract.mjs';
import { main as collect } from '../str005-share-recovery/main.mjs';

export const ENABLED = false;
// Control-stack diagnostic install attempt-001: sealed, complete, ledger unchanged.
export const INSTALL = Object.freeze({
  result: '923b9c6a6a0d36140ebb4db83c763830acf6a65026013202e83711fff8388ad4',
  seal: '91144bd89061d83104e9def3ba44db381bcbc83d3709cb0e9229d8fd69b005e6',
  identity: { firmware_commit: 'c634cc206979fd4179eb32478d20feab1825e31c',
    app_elf_sha256: 'd986b2ead04672f42dbab9eb8c17e52f63cf1877881cf4c7ddcdb276cf8b5770', gate_commit: '86fc62d7a9d75da1affa2d51bc3b9eab41d86031' },
});
// The idle-panic recovery001 on the same board supplies its physical identity and last attempt.
export const RECOVERY001 = Object.freeze({ path: 'scratch/str005-idle-panic/recovery001',
  seal: '3a84383e127d39785e963b1888f8c65d5e3c8664740c602d310b6494e7cd8644' });

async function sealed(root, expected) {
  await privateRoot(root);
  const seal = await proof(root, 'sealed-inventory.json'); check(seal.sha256 === expected, 'share_recovery_predecessor_seal');
  await verifyInventory(root, seal.value.files, new Set(['sealed-inventory.json']));
}
/** Binds the sealed install (identity, ledger, Gate assets) and recovery001 (board, attempt). */
export async function loadInstallPredecessor(firmwareRoot, installRoot) {
  await sealed(installRoot, INSTALL.seal);
  check(await fileDigest(resolve(installRoot, 'final-result.json')) === INSTALL.result, 'share_recovery_predecessor_seal');
  const install = (await proof(installRoot, 'context.json')).value.context, after = (await proof(installRoot, 'accounting-after.json')).value;
  check(install.profile === 'control-stack-diagnostic-install' &&
    Object.entries(INSTALL.identity).every(([key, value]) => install[key] === value), 'share_recovery_predecessor_identity');
  validateLedger(after.ledger); check(after.ledger.pending === false, 'share_recovery_predecessor_identity');
  const recoveryRoot = resolve(firmwareRoot, RECOVERY001.path);
  await sealed(recoveryRoot, RECOVERY001.seal);
  const recovered = (await proof(recoveryRoot, 'context.json')).value;
  const gate = async name => sha256(await readFile(resolve(installRoot, 'qualified-artifacts/gate', name)));
  const context = { ...INSTALL.identity, physical: recovered.physical, original_campaign_id: install.original_campaign_id,
    assetHashes: { page: await gate(PAGE), bundle: await gate(BUNDLE), trust: recovered.assetHashes.trust } };
  return { context, before: { attempt: { id: recovered.attemptId }, ledger: after.ledger }, seal: INSTALL.seal };
}
export const CONTROL_DIAGNOSTIC_RECOVERY = Object.freeze({
  enabled: ENABLED,
  task: 'task-str005-start-panic-diagnosis',
  contract: 'docs/hardware/str005-control-stack-reproduction-amendment.md',
  lines: ['Control diagnostic recovery hardware: enabled.'],
  seal: INSTALL.seal,
  rootOption: '--predecessor-root',
  // The origin that holds the Ultra 205 Web Serial grant; another port would show the chooser.
  port: 48765,
  // The panic happened under the previous image. The RTC boot ordinal restarts with
  // the installed image, so the sealed install itself proves every boot is post-failure.
  failedBootOrdinal: 0,
  loadPredecessor: loadInstallPredecessor,
});
export const main = argv => collect(argv, CONTROL_DIAGNOSTIC_RECOVERY);
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) main(process.argv.slice(2)).then(value => {
  process.stdout.write(`${JSON.stringify(value)}\n`);
}).catch(error => { process.stdout.write(`${JSON.stringify({ error: /^share_recovery_[a-z_]+$/u.test(error.code ?? '') ? error.code : 'share_recovery_failed' })}\n`); process.exitCode = 1; });
