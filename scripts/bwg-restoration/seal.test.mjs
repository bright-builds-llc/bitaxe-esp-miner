import test from "node:test";
import assert from "node:assert/strict";
import { randomBytes } from "node:crypto";
import { execFileSync } from "node:child_process";
import { chmod, mkdir, readFile, writeFile } from "node:fs/promises";
import { resolve } from "node:path";
import { digest, writeNew } from "../fixed-usb-qualification/contract.mjs";
import { CAMPAIGN_RESULT, RESULT_SCHEMA, SCENARIOS } from "./contract.mjs";
import { context, journal, pageState, privateDirectory } from "./fixtures.test-helper.mjs";
import { argumentsFor, seal } from "./main.mjs";
import { shapeHits } from "./seal.mjs";

const POOL = { poolURL: "pool.fixture.invalid", poolPort: 3333, poolUser: "bc1qfixtureowneraddress.bitaxe", poolPassword: "fixture-pool-secret" };
const rejectsWith = (promise, code) => assert.rejects(promise, (error) => error.code === code);
const base64url = (bytes) => randomBytes(bytes).toString("base64url");

/** An attempt root as serve leaves it, with a Git firmware root that ignores the pool file. */
async function attemptRoot({ campaign = "passed", extra = {} } = {}) {
  const base = await privateDirectory("restoration-seal-");
  const firmware = resolve(base, "firmware"), root = resolve(firmware, "scratch/p/attempt-009");
  execFileSync("git", ["init", "-q", firmware]);
  await writeFile(resolve(firmware, ".gitignore"), "scratch/\n");
  await mkdir(root, { recursive: true, mode: 0o700 });
  const pool = resolve(firmware, "scratch/p/pool-credentials.json");
  await writeFile(pool, JSON.stringify(POOL), { mode: 0o600 });
  const frozen = { ...context, attempt: "attempt-009", firmware_root: firmware };
  await writeNew(resolve(root, "context.json"), { context: frozen, sha256: digest(JSON.stringify(frozen)) });
  if (campaign !== null) {
    const result = { schema: RESULT_SCHEMA, attempt: "attempt-009", result: campaign,
      failure: campaign === "passed" ? null : { scenario: "pause", category: "terminal_reason_mismatch" },
      scenarios: SCENARIOS.map((scenario) => ({ scenario, result: "passed", failures: [] })), parity_promotion: false };
    await writeNew(resolve(root, CAMPAIGN_RESULT), { result, sha256: digest(JSON.stringify(result)) });
  }
  const record = { receivedAtUnixMs: 1, scenario: "pause", operation: "statusReview", outcome: "ok", result: null,
    state: pageState({ entries: journal(["connected", "lease_loaded:no_renewal", "status_reviewed:paused"]), deviceIdentity: { epoch: 1, observations: 2 } }) };
  await writeFile(resolve(root, "records.jsonl"), `${JSON.stringify(record)}\n`, { mode: 0o600 });
  for (const [name, text] of Object.entries(extra)) await writeFile(resolve(root, name), text, { mode: 0o600 });
  return { root, pool, frozen };
}

const sealedFile = async (root) => JSON.parse(await readFile(resolve(root, "result.json"), "utf8")).result;

test("finish refuses clearly without the pool credentials file", () => {
  // Arrange / Act / Assert
  assert.throws(() => argumentsFor(["finish", "--private-root", "/p/r"]), (error) => error.code === "restoration_finish_pool_credentials_required");
  assert.equal(argumentsFor(["finish", "--private-root", "/p/r", "--pool-credentials=/p/pool.json"]).options.poolCredentials, "/p/pool.json");
});

test("a clean attempt seals passed with only the scan counts", async () => {
  // Arrange
  const { root, pool, frozen } = await attemptRoot();
  // Act
  const output = await seal(root, frozen, pool);
  // Assert
  const sealed = await sealedFile(root);
  assert.deepEqual([sealed.result, sealed.failure, sealed.credential_scan], ["passed", null, { files: 3, hits: 0 }]);
  assert.deepEqual(output.credential_scan, { files: 3, hits: 0 });
  assert.equal(JSON.stringify(sealed).includes(POOL.poolPassword), false);
});

test("an exact pool value anywhere in the attempt makes it unverified", async () => {
  // Arrange
  const cases = [{ "serve-note.log": `user ${POOL.poolUser}\n` }, { "watcher.stderr.log": `connect stratum+tcp://${POOL.poolURL}:3333/\n` },
    { "copy.json": JSON.stringify({ password: POOL.poolPassword }) }];
  // Act
  const sealed = [];
  for (const extra of cases) {
    const { root, pool, frozen } = await attemptRoot({ extra });
    await seal(root, frozen, pool);
    sealed.push(await sealedFile(root));
  }
  // Assert
  for (const result of sealed) {
    assert.deepEqual([result.result, result.failure.category], ["unverified", "credential_in_evidence"]);
    assert.ok(result.credential_scan.hits > 0);
  }
});

test("credential shapes are hits: a compact JWS, a long base64url run, challenge and lease identifiers", () => {
  // Arrange
  const samples = [`eyJhbGciOiJFZERTQSJ9.${base64url(48)}.${base64url(64)}`, `"authorization":"${base64url(48)}"`,
    `challenge_${base64url(16)}A1`, `lease_${base64url(16)}Z9`];
  // Act
  const hits = samples.map(shapeHits);
  // Assert
  for (const count of hits) assert.ok(count > 0);
});

test("closed tokens, schema names, fact names and hex digests are not credential shapes", () => {
  // Arrange
  const text = [
    JSON.stringify({ sha256: "a".repeat(64), gate: "c".repeat(40), schema: "worker-clock-discontinuity-stimulus-review-v1" }),
    "fact_restoreTokenBeforeRestoreInstruction", "action_token=bwg-restoration-restore-watcher-armed-v1",
    "lease_start_failed lease_60000_ms lease_loaded:no_renewal removal_lease_headroom_5000_ms",
    "bwg-worker-lease-authorization-artifact/0.1 scenario-07-reboot.json 127.0.0.1:48765",
  ].join("\n");
  // Act
  const hits = shapeHits(text);
  // Assert
  assert.equal(hits, 0);
});

test("a shape hit seals the attempt unverified", async () => {
  // Arrange
  const { root, pool, frozen } = await attemptRoot({ extra: { "leak.log": `lease_${base64url(16)}Q7\n` } });
  // Act
  await seal(root, frozen, pool);
  // Assert
  const sealed = await sealedFile(root);
  assert.deepEqual([sealed.result, sealed.failure.category, sealed.credential_scan.hits], ["unverified", "credential_in_evidence", 1]);
});

test("the campaign's own earlier failure keeps precedence over a scan hit", async () => {
  // Arrange
  const { root, pool, frozen } = await attemptRoot({ campaign: "unverified", extra: { "leak.log": POOL.poolPassword } });
  // Act
  await seal(root, frozen, pool);
  // Assert
  const sealed = await sealedFile(root);
  assert.deepEqual([sealed.result, sealed.failure.category, sealed.credential_scan.hits > 0], ["unverified", "terminal_reason_mismatch", true]);
});

test("an attempt without a campaign verdict seals completion_missing with its scan", async () => {
  // Arrange
  const { root, pool, frozen } = await attemptRoot({ campaign: null });
  // Act
  await seal(root, frozen, pool);
  // Assert
  const sealed = await sealedFile(root);
  assert.deepEqual([sealed.result, sealed.failure.category, sealed.credential_scan], ["unverified", "completion_missing", { files: 2, hits: 0 }]);
});

test("a readable pool file or a tampered campaign verdict is refused before anything is written", async () => {
  // Arrange
  const readable = await attemptRoot();
  await chmod(readable.pool, 0o644);
  const tampered = await attemptRoot();
  const record = JSON.parse(await readFile(resolve(tampered.root, CAMPAIGN_RESULT), "utf8"));
  await writeFile(resolve(tampered.root, CAMPAIGN_RESULT), JSON.stringify({ ...record, result: { ...record.result, attempt: "attempt-010" } }), { mode: 0o600 });
  // Act / Assert
  await rejectsWith(seal(readable.root, readable.frozen, readable.pool), "private_path_policy");
  await rejectsWith(seal(tampered.root, tampered.frozen, tampered.pool), "restoration_campaign_result_invalid");
  await rejectsWith(readFile(resolve(tampered.root, "result.json")), "ENOENT");
});
