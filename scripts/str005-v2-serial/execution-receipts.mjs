import { resolve } from "node:path";
import { writeNew, proof } from "../str005-noise-serial/files.mjs";
import { check, digest, object, sha256, uint } from "./values.mjs";
import { decodeJob, verifyShare } from "./job-proof.mjs";
import { matchShareFields } from "./protocol-shares.mjs";

export const PRODUCER = "scripts/str005-v2-serial/execution-routes.mjs";
export const RECEIPT_FILES = Object.freeze({ "connection.json": "connection-comparison", "job-receipt.json": "job",
  "selected-dispatch.json": "dispatch", "selected-nonce.json": "nonce", "selected-submission.json": "submission",
  "selected-device-ack.json": "acknowledgement", "selected-fixture-ack.json": "acknowledgement", "fault.json": "heartbeat-fault" });
export const receiptBytes = value => Buffer.from(`${JSON.stringify(value, null, 2)}\n`);
export function producerDigest(context) {
  const rows = context.evaluator?.filter(row => row.path === PRODUCER);
  check(rows?.length === 1, "v2_execution_producer_missing"); return digest(rows[0].sha256);
}
export function envelope(context, sequence, kind, clock, at, facts) {
  uint(sequence); uint(at); check(sequence > 0 && Object.values(RECEIPT_FILES).includes(kind) &&
    ["device-us", "fixture-us", "supervisor-ms", "parent-hrtime-ms"].includes(clock), "v2_execution_receipt_shape");
  return { schema: "str005-v2-serial-receipt-v1", kind, contextSha256: sha256(JSON.stringify(context)),
    producerSha256: producerDigest(context), sequence, clock, at, facts };
}
export function validateEnvelope(value, context, kind) {
  object(value, ["schema", "kind", "contextSha256", "producerSha256", "sequence", "clock", "at", "facts"]);
  const expected = envelope(context, value.sequence, kind, value.clock, value.at, value.facts);
  check(value.schema === expected.schema && value.kind === kind && value.contextSha256 === expected.contextSha256 && value.producerSha256 === expected.producerSha256,
    "v2_execution_receipt_binding"); return value;
}
export function createReceiptWriter(root, context) {
  let sequence = 0;
  return async (name, clock, at, facts) => {
    check(Object.hasOwn(RECEIPT_FILES, name), "v2_execution_receipt_name");
    const value = envelope(context, ++sequence, RECEIPT_FILES[name], clock, at, facts);
    await writeNew(resolve(root, name), value);
    const stored = await proof(root, name);
    check(stored.sha256 === sha256(receiptBytes(value)), "v2_execution_receipt_changed");
    return stored;
  };
}
export function binding(record) {
  return { workerGeneration: record.workerGeneration, poolSessionGeneration: record.poolSessionGeneration,
    serialTransportEpoch: record.serialTransportEpoch, poolTransportEpoch: record.poolTransportEpoch };
}
/** All sources are decoded native/fixture facts; hashes bind earlier complete receipt bytes. */
export function selectedFacts(record, job, fact, share, maybeNonceSha, maybeSubmissionSha) {
  const decoded = decodeJob(job), submitted = matchShareFields(fact, share), computed = verifyShare(job, fact);
  check(record.jobCommitment === decoded.jobCommitment && computed.sha256d === share.headerSha256d && fact.ackAtDeviceUs !== null &&
    fact.asicIndex === 0 && fact.coreId < 112 && fact.smallCoreId < 8, "v2_selected_share_unproved");
  const ids = binding(record);
  const dispatch = { jobCommitment: job.jobCommitment, ...ids, dispatchSequence: fact.dispatchSequence,
    asicJobId: fact.asicJobId, workFieldsSha256: fact.workFieldsSha256 };
  const nonce = { jobCommitment: job.jobCommitment, ...ids, dispatchSequence: fact.dispatchSequence, asicJobId: fact.asicJobId,
    nonce: fact.nonce, versionBits: fact.versionBits, asicIndex: fact.asicIndex, coreId: fact.coreId, smallCoreId: fact.smallCoreId };
  const submission = { connectionId: job.connectionId, jobCommitment: job.jobCommitment, ...ids,
    nonceReceiptSha256: maybeNonceSha, channelId: submitted.channelId, jobId: submitted.jobId, sequenceNumber: submitted.sequenceNumber,
    nonce: submitted.nonce, ntime: submitted.ntime, version: submitted.version };
  const acknowledgement = { connectionId: job.connectionId, ...ids, submissionSha256: maybeSubmissionSha,
    channelId: fact.channelId, lastSequenceNumber: fact.submissionSequence, matchedSubmitCount: fact.matchedSubmitCount,
    acceptedCount: fact.ackAcceptedCount, sharesSum: fact.ackSharesSum };
  return { dispatch, nonce, submission, deviceAck: { ...acknowledgement, producer: "device" },
    fixtureAck: { ...acknowledgement, matchedSubmitCount: 1, acceptedCount: share.acceptedCount, sharesSum: share.sharesSum, producer: "fixture" } };
}
