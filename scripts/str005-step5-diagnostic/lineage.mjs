import { lstat, readFile, realpath } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { fileDigest, protectedPath } from '../fixed-usb-qualification/contract.mjs';
import { digest, proof, verifyInventory } from '../str005-noise-serial/files.mjs';
import { check } from '../str005-v2-serial/values.mjs';
import { PINS } from './contract.mjs';

/** Reads the sealed phase-1 install: exact identity, current safe facts and the frozen ELF. */
export async function installation(root, pins = PINS) {
  check(pins.installationResult && pins.installationSeal, 'step5_installation_unpinned');
  check(await realpath(root) === resolve(root), 'step5_installation_alias'); await protectedPath(root, true);
  const resultPath = resolve(root, 'final-result.json'); await protectedPath(resultPath);
  check(await fileDigest(resultPath) === pins.installationResult, 'step5_installation_anchor');
  const sealed = await proof(root, 'sealed-inventory.json');
  check(sealed.sha256 === pins.installationSeal && sealed.value.schema === 'noise-serial-seal-v2', 'step5_installation_seal');
  await verifyInventory(root, sealed.value.files, new Set(['sealed-inventory.json']));
  const result = (await proof(root, 'final-result.json')).value, record = (await proof(root, 'context.json')).value;
  const context = record.context, accounting = (await proof(root, 'accounting-after.json')).value, state = accounting.state;
  const restoration = (await proof(root, 'restoration.json')).value;
  check(result.status === 'passed' && result.outcome === 'complete' && record.sha256 === digest(JSON.stringify(context)) &&
    result.contextSha256 === record.sha256 && context.profile === pins.installationProfile &&
    accounting.stage === 'after' && accounting.contextSha256 === record.sha256 && accounting.ledger?.pending === false &&
    accounting.original_budget?.pending === false && state?.expectedFirmwareSourceCommit === context.firmware_commit &&
    state.expectedAppElfSha256 === context.app_elf_sha256 && state.deviceRestorationConfirmed === true &&
    state.deviceLeaseInactive === true && state.running === false && state.preservation?.mine_on_boot === false &&
    Number.isSafeInteger(restoration.status?.observation?.bootOrdinal), 'step5_installation_state');
  const detector = await readFile(resolve(root, 'install-4.detect.stdout.log'), 'utf8');
  const physical = [...detector.matchAll(/^physical_identity_sha256: ([a-f0-9]{64})$/gmu)];
  check(physical.length === 1, 'step5_installation_physical');
  const retainedManifest = resolve(root, 'qualified-artifacts/firmware/bitaxe-ultra205-package.json');
  const manifest = JSON.parse(await readFile(retainedManifest, 'utf8'));
  const elf = manifest.artifacts?.find(item => item.kind === 'firmware_elf');
  const candidateElf = resolve(dirname(retainedManifest), elf?.path ?? '');
  check(elf && (await lstat(candidateElf)).isFile() && await fileDigest(candidateElf) === context.app_elf_sha256 &&
    manifest.source_commit === context.firmware_commit, 'step5_installation_elf');
  return { root, seal: sealed.sha256, physical: physical[0][1], gate_root: context.gate_root,
    identity: { firmware_commit: context.firmware_commit, app_elf_sha256: context.app_elf_sha256, gate_commit: context.gate_commit },
    expectedBootOrdinal: restoration.status.observation.bootOrdinal, ledger: accounting.ledger,
    original_campaign_id: context.original_campaign_id, candidateElf, retainedManifest,
    retainedManifestSha256: await fileDigest(retainedManifest) };
}

/** The Start's Worker generation: diagnostic and share runs keep a proof; heartbeat runs keep the dispatch record. A
 * share run that ended without a share keeps neither, so its in-run status read of the record supplies it. */
export function startGeneration(run, maybeEarlierStatus = null) {
  return run.proof?.generation ?? run.dispatchStatus?.record?.workerGeneration ??
    (run.observedStart === true ? maybeEarlierStatus?.record?.workerGeneration : undefined);
}

/** Current safe facts after a sealed earlier Start on the same install: its fresh recovery
 * (round 1) supplies the ledger and boot ordinal the next baseline must observe unchanged. */
export async function previousStart(root, installed, pins = PINS) {
  check(pins.previousStartResult && pins.previousStartSeal, 'step5_previous_unpinned');
  check(await realpath(root) === resolve(root), 'step5_previous_alias'); await protectedPath(root, true);
  check(await fileDigest(resolve(root, 'result.json')) === pins.previousStartResult, 'step5_previous_anchor');
  const sealed = await proof(root, 'sealed-inventory.json');
  check(sealed.sha256 === pins.previousStartSeal, 'step5_previous_seal');
  await verifyInventory(root, sealed.value.files, new Set(['sealed-inventory.json']));
  const context = (await proof(root, 'context.json')).value, run = (await proof(root, 'run.json')).value;
  const ledger = (await proof(root, 'recovery-1-ledger.json')).value, state = (await proof(root, 'recovery-1-state.json')).value;
  const status = (await proof(root, 'recovery-1-status.json')).value, closed = (await proof(root, 'recovery-1-closed.json')).value;
  const finished = (await proof(root, 'recovery-1-finished.json')).value;
  // Diagnostic and share results differ in shape; both persist the observed Start in run.json.
  check(run.observedStart === true && context.anchors?.installation?.seal === installed.seal &&
    context.firmware_commit === installed.identity.firmware_commit && context.app_elf_sha256 === installed.identity.app_elf_sha256 &&
    context.physical === installed.physical && ledger.pending === false && Array.isArray(finished.failures) && finished.failures.length === 0 &&
    state.deviceRestorationConfirmed === true && state.deviceBaselineConfirmed === true && state.deviceLeaseInactive === true &&
    state.running === false && state.preservation?.mine_on_boot === false && closed.status === 'closed' &&
    closed.serialOwnershipReleased === true && status.state === 'terminal' && Number.isSafeInteger(status.observation?.bootOrdinal),
  'step5_previous_state');
  return { root, seal: sealed.sha256, ledger, expectedBootOrdinal: status.observation.bootOrdinal };
}

/** A sealed, complete no-mining restart after the previous Start: same ledger, boot ordinal N + 1. */
export async function restartAfter(root, previous, pins = PINS) {
  check(pins.restartResult && pins.restartSeal, 'step5_restart_unpinned');
  check(await realpath(root) === resolve(root), 'step5_restart_alias'); await protectedPath(root, true);
  check(await fileDigest(resolve(root, 'result.json')) === pins.restartResult, 'step5_restart_anchor');
  const sealed = await proof(root, 'sealed-inventory.json');
  check(sealed.sha256 === pins.restartSeal, 'step5_restart_seal');
  await verifyInventory(root, sealed.value.files, new Set(['sealed-inventory.json']));
  const result = (await proof(root, 'result.json')).value;
  check(result.schema === 'str005-startup-preparation-result-v1' && result.complete === true && result.mining_started === false &&
    result.parentSeals?.start === previous.seal && Number.isSafeInteger(result.after_boot_ordinal) &&
    result.after_boot_ordinal === result.before_boot_ordinal + 1 && result.before_boot_ordinal === previous.expectedBootOrdinal &&
    result.ledger?.pending === false && JSON.stringify(result.ledger) === JSON.stringify(previous.ledger), 'step5_restart_state');
  return { root, seal: sealed.sha256, ledger: result.ledger, expectedBootOrdinal: result.after_boot_ordinal };
}

/** A sealed current recovery on the installed image and board: idle V2, a settled ledger, and the boot it
 * observed. It re-bases the expected boot after reboots that left no Start, such as reset loops. */
export async function currentRecovery(root, installed, pins = PINS) {
  check(pins.currentRecoveryResult && pins.currentRecoverySeal, 'step5_recovery_unpinned');
  check(await realpath(root) === resolve(root), 'step5_recovery_alias'); await protectedPath(root, true);
  check(await fileDigest(resolve(root, 'result.json')) === pins.currentRecoveryResult, 'step5_recovery_anchor');
  const sealed = await proof(root, 'sealed-inventory.json');
  check(sealed.sha256 === pins.currentRecoverySeal, 'step5_recovery_seal');
  await verifyInventory(root, sealed.value.files, new Set(['sealed-inventory.json']));
  const read = async name => (await proof(root, name)).value;
  const [result, context, ledger, status] = await Promise.all(['result.json', 'context.json', 'ledger.json', 'status.json'].map(read));
  return { root, seal: sealed.sha256, ...recoveryAnchor({ result, context, ledger, status }, installed) };
}
/** Pure admission of a current recovery against the install it must follow. */
export function recoveryAnchor({ result, context, ledger, status }, installed) {
  check(result.schema === 'str005-share-current-recovery-v1' && result.current_safe_recovery === true &&
    result.current_v2_idle === true && result.first_failure === null && result.host_resources_released === true &&
    context.firmware_commit === installed.identity.firmware_commit && context.app_elf_sha256 === installed.identity.app_elf_sha256 &&
    context.physical === installed.physical && ledger?.pending === false && status?.state === 'idle' &&
    Number.isSafeInteger(status.observation?.bootOrdinal), 'step5_recovery_state');
  return { ledger, expectedBootOrdinal: status.observation.bootOrdinal };
}
