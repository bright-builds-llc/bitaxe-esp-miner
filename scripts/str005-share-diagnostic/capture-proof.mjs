import { readFile, readdir } from 'node:fs/promises';
import { resolve } from 'node:path';
import { proof, canonical, retain } from '../str005-noise-serial/files.mjs';
import { check, sha256 } from '../str005-v2-serial/values.mjs';
import { fileDigest } from '../fixed-usb-qualification/contract.mjs';
import { validateSelfTest, validateCompleteEvidence } from '../str005-panic-probe/self-test-evidence.mjs';
import { sourceFingerprint, recoveryStoreObservation } from '../str005-panic-probe/store-diagnostics.mjs';
import { privateBytes, validatePartition, validateReadResult, validateStoreObservation, validateInspection } from '../str005-startup-probe/capture.mjs';
import { main as decode } from '../core-dump/main.mjs';

export function validateProvenanceSummary(value, context, dumpSha) {
  check(value.schema === 'bitaxe-private-core-inspection/1' && value.chip === 'esp32s3' && value.decoder_version === '1.17.2' &&
    value.elf_sha256 === context.app_elf_sha256 && value.dump_sha256 === dumpSha && value.cause_proven === false &&
    ['checksum_verified', 'full_elf_identity_verified', 'native_cutoff_verified', 'captured_memory_verified', 'asic_outputs_disabled',
      'generation_revoked', 'self_test_marked', 'fault_provenance_verified', 'original_frame_meaningful', 'self_test_provenance_qualified'].every(key => value[key] === true), 'diagnostic_capture_provenance');
}
export function validatePanicJoin(value, context, request) {
  const words = value.panic_words;
  check(value.schema === 'bitaxe-private-fault-provenance/1' && value.elf_sha256 === context.app_elf_sha256 && value.cause_proven === false &&
    Array.isArray(words) && words.length === 48 && words.every(word => Number.isInteger(word) && word >= 0 && word <= 0xffffffff), 'diagnostic_capture_frame_shape');
  const source = (BigInt(words[5]) << 32n | BigInt(words[4])).toString(16).padStart(16, '0');
  const boot = BigInt(words[7]) << 32n | BigInt(words[6]);
  check(source === sourceFingerprint(context.firmware_commit) && boot === BigInt(request.expectedBootOrdinal), 'diagnostic_capture_frame_identity');
}
/** Join the actual global one-use read claim to this root and its post-panic recovery. */
export async function consumedRead(root, context, request) {
  const rounds = (await readdir(root)).filter(name => /^candidate-recovery-[0-9]{3}$/u.test(name)).sort();
  check(rounds.length > 0 && rounds.length <= 8, 'diagnostic_capture_recovery');
  const matches = [];
  for (const round of rounds) {
    const recovered = await proof(root, `${round}/current-recovery.json`), status = (await proof(root, `${round}/status.json`)).value;
    if (status.observation?.bootOrdinal !== request.expectedBootOrdinal + 1) continue;
    const key = sha256(canonical(recovered.value));
    let claim;
    try { claim = await proof(resolve(context.firmware_root, 'scratch/core-dump-proof-claims'), `${key}.json`); }
    catch (error) { if (error.code === 'ENOENT') continue; throw error; }
    if (claim.value.output_root !== resolve(root, 'self-test-core')) continue;
    check(claim.value.schema === 'bitaxe-core-dump-proof-claim-v1' && claim.value.operation === 'read' && claim.value.proof_semantic_sha256 === key &&
      claim.value.source_commit === context.commit && claim.value.physical_identity_sha256 === context.detector.physical &&
      recovered.value.source_commit === context.commit && recovered.value.firmware_commit === context.firmware_commit &&
      recovered.value.app_elf_sha256 === context.app_elf_sha256 && recovered.value.physical_identity_sha256 === context.detector.physical, 'diagnostic_capture_read_claim');
    const store = (await proof(root, `${round}/store-observation.json`)).value;
    const diagnostics = (await proof(root, `${round}/diagnostics.json`)).value;
    check(canonical(store) === canonical(recoveryStoreObservation(diagnostics, status, context, request)), 'diagnostic_capture_store_changed');
    matches.push({ store, status, claim });
  }
  check(matches.length === 1, 'diagnostic_capture_read_claim');
  await retain(resolve(root, 'capture-read-claim.json'), matches[0].claim.bytes);
  return matches[0];
}
function category(error) {
  if (error.code === 'ENOENT') return 'unavailable';
  if (error.message === 'decoder_timeout') return 'timeout';
  if (error.message === 'decoder_release_unproven') return 'release_unproven';
  if (error.message === 'decoder_interrupted') return 'interrupted';
  return 'rejected';
}
/** Revalidate every evidence boundary independently; decoder errors retain partial files and never qualify capture. */
export async function assessCapture(root, context, operations = {}) {
  const states = {}, failures = []; let claim, dump, dumpSha, joined; let decoderReleased = true;
  const assess = async (phase, operation) => {
    try { const value = await operation(); states[phase] = 'verified'; return value; }
    catch (error) { const reason = category(error); states[phase] = reason; failures.push({ phase, category: reason }); return undefined; }
  };
  claim = await assess('self_test', async () => {
    const requested = (await proof(root, 'self-test-claim.json')).value.request;
    const saved = (await proof(root, 'self-test-result.json')).value;
    const checked = await (operations.validateSelfTest ?? validateSelfTest)(saved, context.gate_root, requested, context);
    check(checked.summary.stage === 'complete', 'diagnostic_capture_self_test'); validateCompleteEvidence(checked, requested, context); return requested;
  });
  await assess('acquisition', async () => {
    validateReadResult((await proof(root, 'self-test-core/result.private.json')).value, context);
    validatePartition(await privateBytes(root, 'self-test-core/partition-table.private.bin', 4096));
    dump = await privateBytes(root, 'self-test-core/core-dump.private.bin', 974848);
    check(!dump.every(byte => byte === 255) && dump.readUInt32LE(0) > 0 && dump.readUInt32LE(0) <= dump.length, 'diagnostic_capture_empty_dump');
    dumpSha = sha256(dump);
  });
  await assess('read_provenance', async () => {
    check(claim && dump, 'diagnostic_capture_prerequisite'); joined = await (operations.consumedRead ?? consumedRead)(root, context, claim);
    validateStoreObservation(joined.store, joined.status, context, claim.expectedBootOrdinal, dump.readUInt32LE(0));
  });
  await assess('inspection', async () => {
    check(dumpSha, 'diagnostic_capture_prerequisite');
    const initial = (await proof(root, 'inspection/inspection.json')).value, inputs = (await proof(root, 'cutoff-review/inputs.json')).value;
    check(initial.schema === 'bitaxe-private-core-inspection/1' && initial.checksum_verified === true && initial.full_elf_identity_verified === true &&
      initial.elf_sha256 === context.app_elf_sha256 && initial.dump_sha256 === dumpSha, 'diagnostic_capture_inspection');
    validateInspection((await proof(root, 'cutoff-review/inspection.json')).value, inputs, context, dumpSha, {
      decoder: await fileDigest(resolve(context.firmware_root, 'scripts/core-dump/decode_core.py')),
      cutoff: await fileDigest(resolve(context.firmware_root, 'scripts/core-dump/cutoff.py')),
    });
    check(await fileDigest(resolve(root, 'cutoff-review/dump.raw')) === dumpSha &&
      await fileDigest(resolve(root, 'cutoff-review/firmware.elf')) === context.app_elf_sha256, 'diagnostic_capture_inspection_inputs');
  });
  await assess('provenance', async () => {
    check(dumpSha && claim, 'diagnostic_capture_prerequisite');
    try {
      const summary = await (operations.decode ?? decode)(['verify-provenance', '--dump', resolve(root, 'self-test-core/core-dump.private.bin'),
        '--elf', context.candidateElf, '--elf-sha256', context.app_elf_sha256, '--private-root', resolve(root, 'capture-validation')]);
      validateProvenanceSummary(summary, context, dumpSha);
      validatePanicJoin((await proof(root, 'capture-validation/provenance.private.json')).value, context, claim);
    } catch (error) { if (error.message === 'decoder_release_unproven') decoderReleased = false; throw error; }
  });
  await assess('analysis', async () => {
    check(dumpSha && claim, 'diagnostic_capture_prerequisite');
    try {
      const summary = await (operations.decode ?? decode)(['analyze', '--dump', resolve(root, 'self-test-core/core-dump.private.bin'),
        '--elf', context.candidateElf, '--elf-sha256', context.app_elf_sha256, '--private-root', resolve(root, 'capture-analysis-validation')]);
      check(summary.full_elf_identity_verified === true && summary.checksum_verified === true &&
        summary.dump_sha256 === dumpSha && summary.elf_sha256 === context.app_elf_sha256, 'diagnostic_capture_analysis');
      const inputs = (await proof(root, 'capture-analysis-validation/analysis-inputs.json')).value;
      check(inputs.schema === 'bitaxe-private-core-analysis/1' && inputs.dump_sha256 === dumpSha &&
        inputs.elf_sha256 === context.app_elf_sha256 && inputs.explicit_core === true && inputs.bounded_ms === 60000,
      'diagnostic_capture_analysis');
    } catch (error) { if (error.message === 'decoder_release_unproven') decoderReleased = false; throw error; }
  });
  return { schema: 'str005-diagnostic-capture-review-v1', complete: failures.length === 0, states, first_failure: failures[0] ?? null,
    failures, decoder_released: decoderReleased, dump_sha256: dumpSha ?? null, cause_proven: false };
}
