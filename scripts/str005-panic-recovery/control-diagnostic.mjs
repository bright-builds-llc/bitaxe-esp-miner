import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { BUNDLE, PAGE, fileDigest } from '../fixed-usb-qualification/contract.mjs';
import { proof, privateRoot, verifyInventory } from '../str005-noise-serial/files.mjs';
import { check, sha256 } from '../str005-v2-serial/values.mjs';
import { validateLedger } from '../fixed-usb-qualification/iterative-contract.mjs';
import { main as collect } from '../str005-share-recovery/main.mjs';
import { loadSealedStartRecord } from '../str005-startup-probe/start-record.mjs';
import { HEAD } from '../str005-lineage/head.mjs';

export const ENABLED = false;
// The installed image and its latest Start come from the verified lineage head
// (`just str005-lineage show`), never from hand-edited constants.
export const INSTALL = Object.freeze({ profile: HEAD.install.profile, path: HEAD.install.path, result: HEAD.install.result,
  seal: HEAD.install.seal, identity: HEAD.install.identity });
// The idle-panic recovery001 on the same board supplies its physical identity and last attempt.
export const RECOVERY001 = Object.freeze({ path: 'scratch/str005-idle-panic/recovery001',
  seal: '3a84383e127d39785e963b1888f8c65d5e3c8664740c602d310b6494e7cd8644' });
// The latest sealed Start on this install, when one ran: its retained record replaces recovery001's
// attempt. A new install resets it to null, because installing reboots the board.
export const LATEST_START = HEAD.latestStart;

async function sealed(root, expected) {
  await privateRoot(root);
  const seal = await proof(root, 'sealed-inventory.json'); check(seal.sha256 === expected, 'share_recovery_predecessor_seal');
  await verifyInventory(root, seal.value.files, new Set(['sealed-inventory.json']));
}
/** Binds the sealed install (identity, ledger, Gate assets) and recovery001 (board, attempt). */
export async function loadInstallPredecessor(firmwareRoot, installRoot) {
  check(resolve(firmwareRoot, INSTALL.path) === resolve(installRoot), 'share_recovery_predecessor_identity');
  await sealed(installRoot, INSTALL.seal);
  check(await fileDigest(resolve(installRoot, 'final-result.json')) === INSTALL.result, 'share_recovery_predecessor_seal');
  const install = (await proof(installRoot, 'context.json')).value.context, after = (await proof(installRoot, 'accounting-after.json')).value;
  check(install.profile === INSTALL.profile &&
    Object.entries(INSTALL.identity).every(([key, value]) => install[key] === value), 'share_recovery_predecessor_identity');
  validateLedger(after.ledger); check(after.ledger.pending === false, 'share_recovery_predecessor_identity');
  const recoveryRoot = resolve(firmwareRoot, RECOVERY001.path);
  await sealed(recoveryRoot, RECOVERY001.seal);
  const recovered = (await proof(recoveryRoot, 'context.json')).value;
  const maybeLatest = LATEST_START === null ? null : await loadSealedStartRecord(resolve(firmwareRoot, LATEST_START.path), LATEST_START);
  const gate = async name => sha256(await readFile(resolve(installRoot, 'qualified-artifacts/gate', name)));
  const context = { ...INSTALL.identity, physical: recovered.physical, original_campaign_id: install.original_campaign_id,
    assetHashes: { page: await gate(PAGE), bundle: await gate(BUNDLE), trust: recovered.assetHashes.trust } };
  return { context, before: retainedAttempt(context, { attemptId: recovered.attemptId, ledger: after.ledger }, maybeLatest),
    seal: INSTALL.seal };
}
/** The attempt a status read must name. After a sealed Start on this exact install and board, the
 * firmware still retains its record this boot and keys it by the device record attempt; otherwise the
 * install lineage's attempt is named and nothing is retained. */
export function retainedAttempt(install, fallback, maybeLatest) {
  if (maybeLatest === null) return { attempt: { id: fallback.attemptId, recordRetained: false }, ledger: fallback.ledger };
  const { context, ledger, deviceRecordAttemptId } = maybeLatest;
  check(context.firmware_commit === install.firmware_commit && context.app_elf_sha256 === install.app_elf_sha256 &&
    context.physical === install.physical && typeof deviceRecordAttemptId === 'string', 'share_recovery_predecessor_identity');
  validateLedger(ledger); check(ledger.pending === false, 'share_recovery_predecessor_identity');
  return { attempt: { id: deviceRecordAttemptId, recordRetained: true }, ledger };
}
export const CONTROL_DIAGNOSTIC_RECOVERY = Object.freeze({
  enabled: ENABLED,
  // Re-owned by the active task that needs a current recovery; the panic-diagnosis task is archived.
  task: 'task-str005-heartbeat-current-image',
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
