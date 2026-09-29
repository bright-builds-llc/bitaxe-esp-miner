import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { ignored, protectedPath } from '../fixed-usb-qualification/contract.mjs';
import { proof, privateRoot, verifyInventory, canonical } from '../str005-noise-serial/files.mjs';
import { predecessor as sharePredecessor } from '../str005-share-recovery/contract.mjs';
import { STAGES, conclusion, recoveryProof } from '../str005-share-recovery/model.mjs';
import { readRecoveryPart } from '../str005-share-recovery/diagnostics.mjs';
import { producerResult, acquisitionBlockers, fullRegion, DUMP_BYTES } from '../str005-share-crash/model.mjs';
import { check, sha256 } from '../str005-v2-serial/values.mjs';
export const RECOVERY_SEAL = '49abc244d0efe12af30a7265cd5661dea0e735fa78baa1a7ff30b279cf3f39a6';
export const ACQUISITION_SEAL = '3ec746b255da28290d8f1944e1cc375f7deea559f8d936bd49dea8e5ed6ad39b';
export const OLD_DUMP_SHA256 = 'b665c154d35f43bf7a0ab9acfe0aab207ac9e36c6a1a10ce649f3863f880f29c';
export async function sealed(root, repo, expected) {
  ignored(repo, root); await privateRoot(root); const seal = await proof(root, 'sealed-inventory.json');
  check(!expected || seal.sha256 === expected, 'diagnostic_anchor_seal');
  await verifyInventory(root, seal.value.files, new Set(['sealed-inventory.json'])); return seal.sha256;
}
export async function oldAnchors(repo) {
  const share = await sharePredecessor(repo, resolve(repo, 'scratch/str005-share/share001/attempt'));
  const recoveryRoot = resolve(repo, 'scratch/str005-share-recovery/recovery005/attempt');
  await sealed(recoveryRoot, repo, RECOVERY_SEAL);
  const context = (await proof(recoveryRoot, 'context.json')).value, parts = {};
  for (const stage of STAGES) parts[stage] = await readRecoveryPart(recoveryRoot, stage);
  const saved = (await proof(recoveryRoot, 'result.json')).value;
  check(saved.current_safe_recovery === true && saved.host_resources_released === true && saved.blockers.length === 0 &&
    conclusion(parts, context, true).current_safe_recovery && context.firmware_commit === share.context.firmware_commit &&
    context.app_elf_sha256 === share.context.app_elf_sha256 && context.physical === share.context.physical, 'diagnostic_recovery_anchor');
  const current = (await proof(recoveryRoot, 'current-recovery.json')).value, begin = (await proof(recoveryRoot, 'collection-begin.json')).value;
  check(canonical(current) === canonical(recoveryProof(parts, context, begin.startedAtUnixMs, begin.startedAtUnixMs + 1)), 'diagnostic_recovery_proof');
  const archiveRoot = resolve(repo, 'scratch/str005-share-crash/acquisition001/attempt'); await sealed(archiveRoot, repo, ACQUISITION_SEAL);
  const producer = producerResult((await proof(archiveRoot, 'result.private.json')).value);
  const verified = (await proof(archiveRoot, 'acquisition-verification.json')).value;
  check(acquisitionBlockers(producer, verified.source_commit).length === 0 && verified.complete && verified.release_proven &&
    verified.dump_sha256 === OLD_DUMP_SHA256, 'diagnostic_acquisition_anchor');
  for (const name of ['core-dump.private.bin', 'partition-table.private.bin']) await protectedPath(resolve(archiveRoot, name));
  const dump = await readFile(resolve(archiveRoot, 'core-dump.private.bin'));
  fullRegion(await readFile(resolve(archiveRoot, 'partition-table.private.bin')));
  check(dump.length === DUMP_BYTES && sha256(dump) === OLD_DUMP_SHA256, 'diagnostic_archive_bytes');
  return { share, context, archiveRoot, archiveSha: OLD_DUMP_SHA256, ledgerReference: parts.ledger };
}
