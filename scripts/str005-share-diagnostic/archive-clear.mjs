import { resolve } from 'node:path';
import { readFile, stat } from 'node:fs/promises';
import { proof } from '../str005-noise-serial/files.mjs';
import { protectedPath } from '../fixed-usb-qualification/contract.mjs';
import { check, sha256 } from '../str005-v2-serial/values.mjs';
import { sealed } from './anchors.mjs';

/** Rejudge the sealed self-test producer and its actual raw bytes before clearing only their on-device copy. */
export async function captureArchive(root, repo) {
  const seal = await sealed(root, repo);
  const context = (await proof(root, 'context.json')).value;
  const result = (await proof(root, 'result.json')).value;
  const review = result.capture_validation;
  const stages = ['self_test', 'acquisition', 'read_provenance', 'inspection', 'provenance', 'analysis'];
  check(context.diagnosticSuccessor === true && context.stage === 'capture' &&
    result.complete === true && result.core_capture_verified === true && result.self_test_reset_observed === true &&
    result.host_resources_released === true && Array.isArray(result.blockers) && result.blockers.length === 0 &&
    review?.schema === 'str005-diagnostic-capture-review-v1' && review.complete === true && review.decoder_released === true &&
    review.cause_proven === false && review.first_failure === null && stages.every(stage => review.states?.[stage] === 'verified'),
  'diagnostic_capture_archive_unqualified');
  const selfTest = (await proof(root, 'self-test-result.json')).value;
  const request = (await proof(root, 'self-test-claim.json')).value.request;
  check(selfTest.summary?.stage === 'complete' && selfTest.summary.expectedBootOrdinal === request.expectedBootOrdinal &&
    selfTest.summary.nextBootOrdinal === request.expectedBootOrdinal + 1 &&
    selfTest.summary.panicResetObserved === true, 'diagnostic_capture_archive_self_test');
  const acquired = (await proof(root, 'self-test-core/result.private.json')).value;
  check(acquired.terminal_category === 'complete' && acquired.rom_admitted === true && acquired.acquisition_complete === true &&
    acquired.application_identity_restored === true && acquired.cleanup_complete === true &&
    acquired.expected_installed_source === context.firmware_commit && acquired.expected_installed_elf === context.app_elf_sha256,
  'diagnostic_capture_archive_acquisition');
  const verified = (await proof(root, 'capture-validation/inspection.json')).value;
  const analyzed = (await proof(root, 'capture-analysis-validation/inspection.json')).value;
  check(verified.dump_sha256 === review.dump_sha256 && analyzed.dump_sha256 === review.dump_sha256 &&
    verified.elf_sha256 === context.app_elf_sha256 && analyzed.elf_sha256 === context.app_elf_sha256 &&
    verified.checksum_verified === true && verified.full_elf_identity_verified === true &&
    verified.original_frame_meaningful === true && verified.self_test_provenance_qualified === true &&
    analyzed.checksum_verified === true && analyzed.full_elf_identity_verified === true,
  'diagnostic_capture_archive_decode');
  const dumpPath = resolve(root, 'self-test-core/core-dump.private.bin');
  await protectedPath(dumpPath);
  check((await stat(dumpPath)).size === 974848, 'diagnostic_capture_archive_size');
  const dumpSha = sha256(await readFile(dumpPath));
  check(dumpSha === review.dump_sha256, 'diagnostic_capture_archive_digest');
  return { root, seal, context, dumpPath, dumpSha };
}
