import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { fileDigest } from '../fixed-usb-qualification/contract.mjs';
import { privateRoot, proof, verifyInventory } from '../str005-noise-serial/files.mjs';
import { check, object } from '../str005-v2-serial/values.mjs';
import { loadSealedStartRecord } from '../str005-startup-probe/start-record.mjs';

// The lineage head names the installed image's sealed install and, when a Start ran on it, the latest
// sealed Start. Recovery and loop owners read their pins from here instead of hand-edited constants.
// `just str005-lineage` derives every value from the sealed roots, so a pin cannot be typed wrong.
export const SCHEMA = 'str005-lineage-head-v1';
export const HEAD_PATH = 'scripts/str005-lineage/head.json';
const HEX = /^[a-f0-9]{64}$/u, COMMIT = /^[a-f0-9]{40}$/u, RELATIVE = /^scratch\/[A-Za-z0-9._/-]+$/u;

function pin(value, keys) {
  object(value, keys);
  check(RELATIVE.test(value.path) && !value.path.includes('..') && HEX.test(value.result) && HEX.test(value.seal), 'lineage_head_shape');
}
/** Shape-only validation; `verifyHead` proves the pins against the sealed roots. */
export function validateHead(value) {
  object(value, ['schema', 'install', 'latestStart']);
  check(value.schema === SCHEMA, 'lineage_head_shape');
  pin(value.install, ['profile', 'path', 'result', 'seal', 'identity']);
  object(value.install.identity, ['firmware_commit', 'app_elf_sha256', 'gate_commit']);
  const { firmware_commit, app_elf_sha256, gate_commit } = value.install.identity;
  check(/^[a-z0-9-]+$/u.test(value.install.profile) && COMMIT.test(firmware_commit) && HEX.test(app_elf_sha256) &&
    COMMIT.test(gate_commit), 'lineage_head_shape');
  if (value.latestStart !== null) pin(value.latestStart, ['path', 'result', 'seal']);
  return value;
}
/** The committed head, read next to this module so Bazel runfiles and the source tree agree. */
export const HEAD = Object.freeze(validateHead(JSON.parse(readFileSync(new URL('./head.json', import.meta.url), 'utf8'))));

/** A sealed, passing noise-serial install: its digests and the identity it installed. */
export async function installSnapshot(firmwareRoot, path) {
  const root = resolve(firmwareRoot, path); await privateRoot(root);
  const sealed = await proof(root, 'sealed-inventory.json');
  await verifyInventory(root, sealed.value.files, new Set(['sealed-inventory.json']));
  const result = await proof(root, 'final-result.json'), context = (await proof(root, 'context.json')).value.context;
  check(result.value.schema === 'noise-serial-result-v2' && result.value.status === 'passed' && result.value.outcome === 'complete',
    'lineage_install_unverified');
  return { profile: context.profile, path, result: result.sha256, seal: sealed.sha256,
    identity: { firmware_commit: context.firmware_commit, app_elf_sha256: context.app_elf_sha256, gate_commit: context.gate_commit } };
}
/** A sealed Start on the head's install: same image, and a record key its evidence agrees on. */
export async function startSnapshot(firmwareRoot, path, install) {
  const root = resolve(firmwareRoot, path);
  const result = await fileDigest(resolve(root, 'result.json')), seal = (await proof(root, 'sealed-inventory.json')).sha256;
  const start = await loadSealedStartRecord(root, { result, seal });
  check(start.context.firmware_commit === install.identity.firmware_commit &&
    start.context.app_elf_sha256 === install.identity.app_elf_sha256, 'lineage_start_other_image');
  return { path, result, seal };
}
/** Re-derives every pin from the sealed roots and requires the committed head to match exactly. */
export async function verifyHead(firmwareRoot, head = HEAD) {
  validateHead(head);
  const install = await installSnapshot(firmwareRoot, head.install.path);
  check(JSON.stringify(install) === JSON.stringify(head.install), 'lineage_head_install_changed');
  if (head.latestStart !== null)
    check(JSON.stringify(await startSnapshot(firmwareRoot, head.latestStart.path, install)) === JSON.stringify(head.latestStart),
      'lineage_head_start_changed');
  return head;
}
/** A new install replaces the image and reboots the board, so no Start record can survive it. */
export async function advanceInstall(firmwareRoot, path) {
  return validateHead({ schema: SCHEMA, install: await installSnapshot(firmwareRoot, path), latestStart: null });
}
/** Records the latest sealed Start on the current install. */
export async function recordStart(firmwareRoot, head, path) {
  return validateHead({ ...head, latestStart: await startSnapshot(firmwareRoot, path, head.install) });
}
