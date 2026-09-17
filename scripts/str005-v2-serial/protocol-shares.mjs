import { check, digest, list, object, sha256, uint } from "./values.mjs";
import { verifyShare } from "./job-proof.mjs";
import { acknowledgementFrame, submissionFrame } from "./protocol-fixture.mjs";

/** Correlate source-produced fields; this helper alone makes no proof-of-work claim. */
export function matchShareFields(fact, share) {
  object(share, ["submission", "receivedAtFixtureUs", "headerSha256d", "targetValid", "writeStartedAtFixtureUs", "writeCompletedAtFixtureUs", "acceptedCount", "sharesSum"]);
  const submitted = share.submission;
  object(submitted, ["channelId", "sequenceNumber", "jobId", "nonce", "ntime", "version"]);
  for (const value of Object.values(submitted)) uint(value, 0xffffffff);
  digest(share.headerSha256d);
  for (const key of ["receivedAtFixtureUs", "writeStartedAtFixtureUs", "writeCompletedAtFixtureUs"]) uint(share[key]);
  check(share.targetValid === true && share.acceptedCount === 1 && share.sharesSum === 1024 &&
    share.writeStartedAtFixtureUs >= share.receivedAtFixtureUs && share.writeCompletedAtFixtureUs >= share.writeStartedAtFixtureUs &&
    share.writeCompletedAtFixtureUs - share.writeStartedAtFixtureUs <= 2000000, "v2_fixture_share_ack");
  check(fact.submissionSequence === submitted.sequenceNumber, "v2_submission_sequence");
  for (const key of ["channelId", "jobId", "nonce", "ntime", "version"]) check(fact[key] === submitted[key], "v2_native_fixture_share_mismatch");
  check(fact.writeStartedAtDeviceUs !== null && fact.writeCompletedAtDeviceUs !== null &&
    fact.writeCompletedAtDeviceUs >= fact.writeStartedAtDeviceUs && fact.writeCompletedAtDeviceUs - fact.writeStartedAtDeviceUs <= 2000000, "v2_native_write_unproved");
  // A fixture ACK write is not a device receipt. Unobserved late ACKs remain distinct.
  if (fact.ackAtDeviceUs !== null) check(fact.ackLastSequence === fact.submissionSequence && fact.ackAcceptedCount === 1 &&
    fact.ackSharesSum === 1024 && fact.matchedSubmitCount === 1 && fact.ackAtDeviceUs >= fact.writeCompletedAtDeviceUs, "v2_native_ack_mismatch");
  return submitted;
}

/** Every credited fixture share joins real ASIC evidence and an independently verified header. */
export function judgeShares(record, jobProof, fixture) {
  const shares = list(fixture.shares, 1024), facts = record.shareFacts;
  check(shares.length > 0 && facts.length >= shares.length, "v2_share_inventory_join");
  const workReady = record.events.find(row => row.kind === "work_ready");
  check(workReady, "v2_share_work_ready_missing");
  const proofs = new Map();
  for (const fact of facts) {
    check(fact.dispatchedAtDeviceUs >= workReady.atDeviceUs && fact.nonceAtDeviceUs <= record.observedAtUs &&
      (fact.writeCompletedAtDeviceUs === null || fact.writeCompletedAtDeviceUs <= record.observedAtUs) &&
      (fact.ackAtDeviceUs === null || fact.ackAtDeviceUs <= record.observedAtUs), "v2_native_share_time");
    check(fact.asicJobId <= 120 && fact.asicJobId % 8 === 0 && fact.asicIndex === 0 && fact.coreId < 112 && fact.smallCoreId < 8, "v2_asic_fact_domain");
    check(!proofs.has(fact.submissionSequence), "v2_duplicate_share_credit");
    proofs.set(fact.submissionSequence, verifyShare(jobProof, fact));
  }
  const sequences = new Set(), nonces = new Set();
  const verified = [];
  for (const [index, share] of shares.entries()) {
    const fact = facts.find(row => row.submissionSequence === share.submission?.sequenceNumber);
    check(fact, "v2_native_share_missing");
    const submitted = matchShareFields(fact, share), nonceKey = `${submitted.nonce}:${submitted.version}`;
    check(!sequences.has(submitted.sequenceNumber) && !nonces.has(nonceKey) &&
      (index === 0 || submitted.sequenceNumber > shares[index - 1].submission.sequenceNumber), "v2_duplicate_share_credit");
    sequences.add(submitted.sequenceNumber); nonces.add(nonceKey);
    const computed = proofs.get(fact.submissionSequence);
    check(computed.sha256d === share.headerSha256d, "v2_fixture_header_disagreement");
    const received = fixture.events[6 + index * 2], acknowledged = fixture.events[7 + index * 2];
    check(received.kind === "share_received" && received.submissionSequence === submitted.sequenceNumber &&
      received.payloadSha256 === sha256(submissionFrame(submitted)) && received.atFixtureUs <= share.receivedAtFixtureUs &&
      acknowledged.kind === "share_success_sent" && acknowledged.submissionSequence === submitted.sequenceNumber &&
      acknowledged.payloadSha256 === sha256(acknowledgementFrame(submitted)) && acknowledged.atFixtureUs >= share.writeCompletedAtFixtureUs,
    "v2_fixture_share_event_join");
    verified.push({ fact, headerSha256: computed.headerSha256, sha256d: computed.sha256d });
  }
  for (const fact of facts) if (!sequences.has(fact.submissionSequence)) {
    check(fact.writeCompletedAtDeviceUs === null && fact.ackAtDeviceUs === null && fact.ackLastSequence === null && fact.ackAcceptedCount === null && fact.ackSharesSum === null && fact.matchedSubmitCount === null, "v2_completed_write_without_peer_receipt");
  }
  const received = verified.filter(row => row.fact.ackAtDeviceUs !== null).sort((a, b) => a.fact.ackAtDeviceUs - b.fact.ackAtDeviceUs || a.fact.submissionSequence - b.fact.submissionSequence);
  check(received.length > 0, "v2_device_ack_missing");
  return { selected: received[0], submitted: shares.length, acknowledged: received.length, unsubmitted: facts.length - shares.length };
}

/** Device-time fault proof stays independent of host request and fixture arrival clocks. */
export function judgeFault(fault, record, selected) {
  object(fault, ["workerGeneration", "poolSessionGeneration", "serialTransportEpoch", "poolTransportEpoch", "selectedDeviceAckSha256", "suppressionRequestedAtHostMs", "headroomObservedAtDeviceUs", "leaseRemainingMs", "workGateRemainingMs", "revocationReason", "lastValidHeartbeatAtDeviceUs", "gateClosedAtDeviceUs", "shutdownStartedAtDeviceUs", "observerTailMs"]);
  for (const key of ["workerGeneration", "poolSessionGeneration", "serialTransportEpoch", "poolTransportEpoch"]) check(fault[key] === record[key], "v2_fault_binding");
  digest(fault.selectedDeviceAckSha256);
  for (const key of ["suppressionRequestedAtHostMs", "headroomObservedAtDeviceUs", "leaseRemainingMs", "workGateRemainingMs", "lastValidHeartbeatAtDeviceUs", "gateClosedAtDeviceUs", "shutdownStartedAtDeviceUs", "observerTailMs"]) uint(fault[key]);
  check(fault.revocationReason === "heartbeat_timeout" && fault.leaseRemainingMs >= 5000 && fault.leaseRemainingMs <= 60000 && fault.workGateRemainingMs >= 5000 && fault.workGateRemainingMs <= 164450 && fault.observerTailMs >= 5000, "v2_fault_policy");
  check(fault.headroomObservedAtDeviceUs >= Math.floor(selected.fact.ackAtDeviceUs / 1000) * 1000 && fault.headroomObservedAtDeviceUs <= fault.gateClosedAtDeviceUs &&
    fault.gateClosedAtDeviceUs >= fault.lastValidHeartbeatAtDeviceUs + 2800000 && fault.gateClosedAtDeviceUs <= fault.lastValidHeartbeatAtDeviceUs + 3000000 &&
    fault.shutdownStartedAtDeviceUs >= fault.gateClosedAtDeviceUs && fault.shutdownStartedAtDeviceUs <= fault.lastValidHeartbeatAtDeviceUs + 3000000,
  "v2_fault_device_deadline");
  check(record.authorityDeadlineDeviceUs !== null && fault.headroomObservedAtDeviceUs + fault.workGateRemainingMs * 1000 <= record.authorityDeadlineDeviceUs - 15550000,
    "v2_fault_work_gate");
  for (const fact of record.shareFacts) check(fact.dispatchedAtDeviceUs <= fault.gateClosedAtDeviceUs && (fact.writeStartedAtDeviceUs === null || fact.writeStartedAtDeviceUs <= fault.gateClosedAtDeviceUs), "v2_activity_after_revocation");
  for (const kind of ["revoked", "shutdown"]) {
    const rows = record.events.filter(row => row.kind === kind);
    const at = kind === "revoked" ? fault.gateClosedAtDeviceUs : fault.shutdownStartedAtDeviceUs;
    check(rows.length === 1 && rows[0].atDeviceUs >= at && rows[0].atDeviceUs <= record.observedAtUs, "v2_fault_event_join");
  }
  return { heartbeatToRevocationUs: fault.gateClosedAtDeviceUs - fault.lastValidHeartbeatAtDeviceUs,
    heartbeatToShutdownUs: fault.shutdownStartedAtDeviceUs - fault.lastValidHeartbeatAtDeviceUs, observerTailMs: fault.observerTailMs };
}
