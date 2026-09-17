import { createHash } from "node:crypto";
import { bytes, check, digest, object, sha256, uint } from "./values.mjs";

export const SHARE_TARGET = (0xffffn << 208n) / 1024n;
export const VERSION_MASK = 0x1fffe000;
export const BASE_VERSION = 0x20000000;
const hash = (value) => createHash("sha256").update(value).digest();

/** Numeric little-endian comparison, independent of firmware floating-point difficulty. */
export function littleInteger(value) {
  check(Buffer.isBuffer(value) && value.length === 32, "v2_hash_length");
  return BigInt(`0x${Buffer.from(value).reverse().toString("hex")}`);
}
export function meetsTarget(value) { return littleInteger(value) <= SHARE_TARGET; }
export function hashHeader(header) {
  check(Buffer.isBuffer(header) && header.length === 80, "v2_header_length");
  const sha256d = hash(hash(header));
  return { sha256d, qualified: meetsTarget(sha256d) };
}

function frame(encoded, type, channelMessage) {
  check(typeof encoded === "string" && encoded.length <= 2740, "v2_frame_bound");
  const value = Buffer.from(encoded, "base64");
  check(value.length >= 6 && value.length <= 2054 && value.toString("base64") === encoded, "v2_frame_encoding");
  check(value.readUInt16LE(0) === (channelMessage ? 0x8000 : 0) && value[2] === type, "v2_frame_header");
  check(value.readUIntLE(3, 3) === value.length - 6, "v2_frame_length");
  return { encoded: value, payload: value.subarray(6) };
}
function commitment(frames) {
  const h = createHash("sha256");
  for (const item of frames) {
    const length = Buffer.alloc(4); length.writeUInt32LE(item.length); h.update(length).update(item);
  }
  return h.digest("hex");
}

/** Decode only permitted public protocol fields, without calling the firmware codecs. */
export function decodeJob(value) {
  object(value, ["connectionId", "jobCommitment", "channelSuccess", "newMiningJob", "setTarget", "setNewPrevHash"]);
  bytes(value.connectionId, 16); digest(value.jobCommitment);
  const frames = [frame(value.channelSuccess, 0x11, false), frame(value.newMiningJob, 0x15, true),
    frame(value.setTarget, 0x21, true), frame(value.setNewPrevHash, 0x20, true)];
  check(commitment(frames.map((f) => f.encoded)) === value.jobCommitment, "v2_job_commitment");
  const [channel, job, target, previous] = frames.map((f) => f.payload);
  check(channel.length >= 45 && channel.length === 45 + channel[40] && job.length === 45 && job[8] === 0 &&
    target.length === 36 && previous.length === 48, "v2_job_payload");
  const channelId = channel.readUInt32LE(4), jobId = job.readUInt32LE(4);
  check(channelId > 0 && jobId > 0 && [job, target, previous].every((p) => p.readUInt32LE(0) === channelId) &&
    previous.readUInt32LE(4) === jobId, "v2_job_binding");
  check(littleInteger(channel.subarray(8, 40)) === SHARE_TARGET && littleInteger(target.subarray(4)) === SHARE_TARGET, "v2_job_target");
  check(job.readUInt32LE(9) === BASE_VERSION && previous.readUInt32LE(44) === 0x207fffff, "v2_job_profile");
  return { connectionId: value.connectionId, jobCommitment: value.jobCommitment, channelId, jobId,
    requestId: channel.readUInt32LE(0), merkleRoot: Buffer.from(job.subarray(13)),
    previousHash: Buffer.from(previous.subarray(8, 40)), ntime: previous.readUInt32LE(40),
    nbits: previous.readUInt32LE(44), version: BASE_VERSION };
}

/** Assemble the consensus 80-byte header directly from independently decoded fixture fields. */
export function assembleHeader(job, submission) {
  for (const key of ["channelId", "jobId", "nonce", "ntime", "version"]) uint(submission[key], 0xffffffff);
  check(submission.channelId === job.channelId && submission.jobId === job.jobId && submission.ntime === job.ntime,
    "v2_submission_binding");
  check(((submission.version & ~VERSION_MASK) >>> 0) === BASE_VERSION, "v2_submission_version");
  const header = Buffer.alloc(80);
  header.writeUInt32LE(submission.version, 0); job.previousHash.copy(header, 4); job.merkleRoot.copy(header, 36);
  header.writeUInt32LE(submission.ntime, 68); header.writeUInt32LE(job.nbits, 72); header.writeUInt32LE(submission.nonce, 76);
  return header;
}
function reverseWords(value) {
  const result = Buffer.alloc(32);
  for (let i = 0; i < 8; i++) value.copy(result, i * 4, (7 - i) * 4, (8 - i) * 4);
  return result;
}

/** Native payload commitment joins actual dispatched work to the independent job. */
export function workPayload(job, asicJobId) {
  uint(asicJobId, 127); check(asicJobId % 8 === 0, "v2_asic_job_slot");
  const payload = Buffer.alloc(82);
  payload[0] = asicJobId; payload[1] = 1;
  payload.writeUInt32LE(job.nbits, 6); payload.writeUInt32LE(job.ntime, 10);
  reverseWords(job.merkleRoot).copy(payload, 14); reverseWords(job.previousHash).copy(payload, 46);
  payload.writeUInt32LE(job.version, 78);
  return payload;
}

/** Verify a real native fact against an independent fixture job; no acceptance is inferred from ACK alone. */
export function verifyShare(jobProof, fact) {
  const job = decodeJob(jobProof);
  uint(fact.versionBits, 0xffffffff);
  check((fact.versionBits & ~VERSION_MASK) === 0 && fact.version === ((BASE_VERSION | fact.versionBits) >>> 0), "v2_nonce_version");
  check(sha256(workPayload(job, fact.asicJobId)) === fact.workFieldsSha256, "v2_dispatch_payload");
  const header = assembleHeader(job, fact), computed = hashHeader(header);
  check(computed.qualified, "v2_unqualified_nonce");
  return { headerSha256: sha256(header), sha256d: computed.sha256d.toString("hex"), qualified: true };
}
