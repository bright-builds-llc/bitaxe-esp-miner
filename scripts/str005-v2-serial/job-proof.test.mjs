import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";
import { assembleHeader, hashHeader, meetsTarget, SHARE_TARGET, workPayload } from "./job-proof.mjs";

const asymmetric = JSON.parse(await readFile(new URL("../../crates/bitaxe-stratum/fixtures/nonzero-version-known-answer/fixture.json", import.meta.url)));
function littleBytes(value) { return Buffer.from(value.toString(16).padStart(64, "0"), "hex").reverse(); }

test("finite target includes equality and excludes target plus one", () => {
  // Arrange
  const at = littleBytes(SHARE_TARGET), over = littleBytes(SHARE_TARGET + 1n);
  // Act / Assert
  assert.equal(meetsTarget(at), true);
  assert.equal(meetsTarget(over), false);
  assert.equal(Buffer.from(at).reverse().toString("hex"), "00000000003fffc0000000000000000000000000000000000000000000000000");
});

test("Bitcoin genesis is an independently published positive numeric oracle", () => {
  // Arrange: factual header fields/hash from Bitcoin Core v30.0 kernel/chainparams.cpp127–130.
  // https://github.com/bitcoin/bitcoin/blob/v30.0/src/kernel/chainparams.cpp#L127-L130
  // This has a different nbits/version from the live fixture; it is not a live campaign.
  const header = Buffer.alloc(80); header.writeUInt32LE(1, 0);
  Buffer.from("4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b", "hex").reverse().copy(header, 36);
  header.writeUInt32LE(1231006505, 68); header.writeUInt32LE(0x1d00ffff, 72); header.writeUInt32LE(2083236893, 76);
  // Act
  const result = hashHeader(header);
  // Assert
  assert.equal(Buffer.from(result.sha256d).reverse().toString("hex"), "000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f");
  assert.equal(result.qualified, true);
});

test("asymmetric nonzero-version known-answer header stays negative at1024", () => {
  // Arrange
  const header = Buffer.from(asymmetric.header_hex, "hex");
  // Act
  const result = hashHeader(header);
  // Assert
  assert.equal(result.sha256d.toString("hex"), asymmetric.sha256d_hex);
  assert.equal(result.qualified, false);
});

test("independent native payload reproduces the existing asymmetric ASIC vector", () => {
  // Arrange
  const header = Buffer.from(asymmetric.header_hex, "hex");
  const job = { nbits: asymmetric.nbits, ntime: asymmetric.ntime, version: asymmetric.base_version,
    previousHash: header.subarray(4, 36), merkleRoot: header.subarray(36, 68) };
  // Act
  const payload = workPayload(job, 40);
  // Assert
  assert.equal(payload.toString("hex"), asymmetric.work_payload_hex);
});

test("submitted ntime cannot silently replace the fixture time", () => {
  const job = { channelId: 1, jobId: 2, ntime: 3 };
  assert.throws(() => assembleHeader(job, { channelId: 1, jobId: 2, ntime: 4, version: 0x20000000, nonce: 5 }),
    { code: "v2_submission_binding" });
});

test("out-of-mask versions are not accepted as supersets of the base", () => {
  const job = { channelId: 1, jobId: 2, ntime: 3 };
  assert.throws(() => assembleHeader(job, { channelId: 1, jobId: 2, ntime: 3, version: 0x20000001, nonce: 5 }),
    { code: "v2_submission_version" });
});
