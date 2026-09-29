import { proof } from '../str005-noise-serial/files.mjs';
import { validateAttempt, validateLedger } from '../fixed-usb-qualification/iterative-contract.mjs';
import { check } from '../str005-v2-serial/values.mjs';
import { sealed } from './anchors.mjs';
const STATUS_SEAL = 'dbda3bed4c468674c2005921e9a3ec1c0e7e9f0587dbc654909a8a92d38a04fc';
/** Identify the exact consumed attempt; its failed result is lineage, never safety authority. */
export async function statusAttempt(root, repo) {
  const seal = await sealed(root, repo);
  check(seal === STATUS_SEAL, 'diagnostic_status_attempt_seal');
  const context = (await proof(root, 'context.json')).value, before = (await proof(root, 'before.json')).value;
  const run = (await proof(root, 'run.json')).value, result = (await proof(root, 'result.json')).value;
  const ledger = (await proof(root, 'recovery-1-ledger.json')).value;
  validateAttempt(before.attempt); validateLedger(before.ledger); validateLedger(ledger);
  check(context.schema === 'str005-status-repro-context-v1' && context.source_commit === '6055bf39072257ac0be281a86ce9ef164881fd16' &&
    context.firmware_commit === 'ce8f015811b93385c5aab0bac7bcdf6307452b31' &&
    context.app_elf_sha256 === '453d2fa3bbe2b58bcffcbf2019ab69c7968d1ae90620685c8139a061a3325c31' &&
    context.gate_commit === '9643e87664397a321c715a3a1b1bb6c1183b83ea' &&
    before.attempt.ordinal === before.ledger.next_ordinal && before.ledger.pending === false &&
    run.firstFailure === 'start' && run.observedStart === false && run.startInvokedAt !== null &&
    result.complete === false && result.host_resources_released === true &&
    ledger.next_ordinal === before.ledger.next_ordinal + 1 && ledger.last_completed_ordinal === before.ledger.next_ordinal &&
    ledger.total_charged_ms === before.ledger.total_charged_ms + 180000 && ledger.pending === false,
  'diagnostic_status_attempt_binding');
  return { root, seal, context, attempt: before.attempt, ledger };
}
