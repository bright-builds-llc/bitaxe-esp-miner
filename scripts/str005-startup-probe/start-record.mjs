import { resolve } from 'node:path';
import { fileDigest } from '../fixed-usb-qualification/contract.mjs';
import { privateRoot, proof, verifyInventory } from '../str005-noise-serial/files.mjs';
import { check } from '../str005-v2-serial/values.mjs';

// A Start owner's evidence carries two different identifiers that are both called `attemptId`:
// - `context.json` `attemptId` is a host nonce for the owner run. Firmware never sees it.
// - the qualification attempt (`before.json` `attempt.id`) is what the grant carried and what the
//   firmware keys its retained V2 record by.
// Status reads of a retained record must name the second one; this module is the only place that
// derives it from sealed Start evidence.

/** The firmware's record key, required to agree across the baseline, the run and the fresh recovery. Heartbeat
 * runs keep the dispatch record; share and diagnostic runs keep a proof of the Start's Worker generation. A share
 * run that ended without a share keeps no proof, so its in-run status read must name the same record instead. */
export function deviceRecordAttemptId({ before, run, recoveryStatus, maybeEarlierStatus = null }) {
  const issued = before?.attempt?.id, dispatched = run?.dispatchStatus?.record, retained = recoveryStatus?.record;
  const earlier = maybeEarlierStatus?.record;
  const runBinds = dispatched ? dispatched.attemptId === issued
    : run?.proof ? Number.isSafeInteger(run.proof.generation) && run.proof.generation === retained?.workerGeneration
      : run?.observedStart === true && earlier?.attemptId === issued && Number.isSafeInteger(earlier.workerGeneration) &&
        earlier.workerGeneration === retained?.workerGeneration;
  check(typeof issued === 'string' && runBinds && retained?.attemptId === issued &&
    retained.scope === 'share' && recoveryStatus.state === 'terminal', 'start_record_attempt_mismatch');
  return issued;
}

/** Reads a sealed Start root pinned by its result and inventory digests. */
export async function loadSealedStartRecord(root, { result, seal }) {
  await privateRoot(root);
  const sealed = await proof(root, 'sealed-inventory.json');
  check(sealed.sha256 === seal, 'start_record_seal');
  await verifyInventory(root, sealed.value.files, new Set(['sealed-inventory.json']));
  check(await fileDigest(resolve(root, 'result.json')) === result, 'start_record_result');
  const read = async name => (await proof(root, name)).value;
  const [context, before, run, recoveryStatus, ledger] = await Promise.all(['context.json', 'before.json', 'run.json',
    'recovery-1-status.json', 'recovery-1-ledger.json'].map(read));
  const maybeEarlierStatus = run.proof || run.dispatchStatus ? null : await read('recovery-0-status.json');
  return { context, ledger, deviceRecordAttemptId: deviceRecordAttemptId({ before, run, recoveryStatus, maybeEarlierStatus }) };
}
