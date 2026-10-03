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
    result.contextSha256 === record.sha256 && context.profile === 'step5-diagnostic-install' &&
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
