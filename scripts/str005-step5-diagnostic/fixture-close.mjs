import { optionalProof } from '../str005-startup-probe/finish.mjs';

/** Closed facts about the fixture's own outcome; a diagnostic Start makes no share claim. */
export async function fixtureFacts(root) {
  const terminal = await optionalProof(root, 'fixture-run/fixture-terminal.json');
  const connection = await optionalProof(root, 'fixture-run/connection-facts.json');
  const reap = await optionalProof(root, 'fixture-reap.json');
  return {
    natural_exit: reap?.kind === 'natural_exit',
    exact_peer: connection?.expectedPeerMatch === true && connection.unexpectedPeerCount === 0 && connection.candidateOverflow === false,
    closed: terminal?.peerClosed === true && terminal.socketClosed === true && terminal.listenerClosed === true,
    received_shares: Number.isSafeInteger(terminal?.receivedShares) ? terminal.receivedShares : null,
    accepted_shares: Number.isSafeInteger(terminal?.acceptedShares) ? terminal.acceptedShares : null,
    invalid_shares: Number.isSafeInteger(terminal?.rejectedShares) && Number.isSafeInteger(terminal?.duplicateShares)
      ? terminal.rejectedShares + terminal.duplicateShares : null,
    outcome: typeof terminal?.outcome === 'string' ? terminal.outcome : null,
    first_failure_stage: typeof terminal?.firstFailure?.stage === 'string' ? terminal.firstFailure.stage : null,
  };
}

/** The status-only Start stops before mining can matter, so completion is a clean natural close:
 * the owner required no share (`required: false`), the exact peer closed, nothing invalid arrived. */
export function cleanDiagnosticClose(completion, facts) {
  return completion?.schema === 'str005-startup-fixture-completion-v1' && completion.required === false &&
    completion.complete === null && facts.natural_exit && facts.exact_peer && facts.closed && facts.invalid_shares === 0 &&
    (facts.outcome === 'accepted' || facts.first_failure_stage === 'peer_eof');
}
