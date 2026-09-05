import test from "node:test";
import assert from "node:assert/strict";
import { once } from "node:events";
import { generateKeyPairSync } from "node:crypto";
import { chmod, mkdir, mkdtemp, readFile, readdir, realpath, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, resolve } from "node:path";
import { admitTrust, BUNDLE, digest, packageSnapshot, PAGE, WINDOW_MS, writeNew } from "./contract.mjs";
import { preflight, loadContext } from "./preflight.mjs";
import { signWindow } from "./authority.mjs";
import { createSupervisor } from "./server.mjs";
import { judgeWindow, validateCycle, validateState } from "./judge.mjs";
import { main } from "./main.mjs";

const SOURCE = "a".repeat(40), GATE = "b".repeat(40), REFERENCE = "c".repeat(40);
const BASELINE = Buffer.alloc(16, 3).toString("base64url");
const BINDING = Buffer.alloc(32, 4).toString("base64url");
function publicTrust() {
  const key = generateKeyPairSync("ed25519").publicKey.export({ format: "jwk" });
  const keys = [{ kid: "fixture-key", kty: "OKP", crv: "Ed25519", x: key.x, alg: "Ed25519", use: "sig", key_ops: ["verify"] }];
  return { profile: "bwg-worker-deployment-trust/0.2", updateAuthority: { issuer: "fixture-update", audience: "bwg-reference-firmware-capability/0.2", role: "update_authority", keys },
    workLeaseAuthority: { issuer: "fixture-lease", audience: "bwg-worker-controller/0.4", role: "work_lease_authority", keys } };
}
async function fixture(t) {
  const base = await realpath(await mkdtemp(resolve(tmpdir(), "fixed-usb-fixture-")));
  t.after(() => rm(base, { recursive: true, force: true }));
  await chmod(base, 0o700);
  const firmwareRoot = resolve(base, "firmware"), gateRoot = resolve(base, "gate"), authorityDirectory = resolve(base, "authority");
  await Promise.all([mkdir(firmwareRoot), mkdir(gateRoot), mkdir(authorityDirectory, { mode: 0o700 })]);
  const manifest = resolve(firmwareRoot, "bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json");
  await mkdir(dirname(manifest), { recursive: true });
  await mkdir(resolve(firmwareRoot, "firmware/bitaxe/bwg"), { recursive: true });
  const trust = publicTrust();
  await writeFile(resolve(firmwareRoot, "firmware/bitaxe/bwg/deployment-trust.json"), JSON.stringify(trust));
  await writeFile(resolve(firmwareRoot, "TASKS.md"), "## Active\n### task-fixed-usb-serial-qualification | fixture\n### task-fixed-usb-worker-live-acceptance | fixture\n");
  for (const file of [PAGE, BUNDLE]) await mkdir(dirname(resolve(gateRoot, file)), { recursive: true });
  await writeFile(resolve(gateRoot, PAGE), "<!doctype html><html><body>Fixture page without device code</body></html>");
  await writeFile(resolve(gateRoot, BUNDLE), `const fixtureCommit = '${GATE}';`);
  const artifacts = [];
  for (const kind of ["firmware_elf", "firmware_ota_image", "www_spiffs_image", "factory_merged_image", "partition_table", "otadata_initial", "bootloader", "partition_table_binary"]) {
    const bytes = Buffer.from(`fixture-${kind}`);
    const path = kind === "partition_table" ? "firmware/bitaxe/partitions-ultra205.csv" : `${kind}.bin`;
    await writeFile(resolve(kind === "partition_table" ? firmwareRoot : dirname(manifest), path), bytes);
    artifacts.push({ kind, path, sha256: digest(bytes) });
  }
  const update_segments = [["bootloader", 0], ["partition_table_binary", 0x8000], ["firmware_ota_image", 0x10000], ["www_spiffs_image", 0x410000], ["otadata_initial", 0xf10000]]
    .map(([artifact_kind, offset]) => ({ artifact_kind, offset, length: Buffer.byteLength(`fixture-${artifact_kind}`) }));
  const manifestValue = { schema_version: 4, source_commit: SOURCE, reference_commit: REFERENCE, build_identity: { source_dirty: false },
    app_elf_sha256: artifacts[0].sha256, artifacts, update_segments };
  await writeFile(manifest, JSON.stringify(manifestValue));
  const options = { firmwareRoot, gateRoot, authorityDirectory, manifest, firmwareCommit: SOURCE, gateCommit: GATE, privateRoot: resolve(base, "attempt-001") };
  const operations = { cleanPushed: () => undefined, ignored: () => undefined, authorityCall: async () => trust };
  return { base, options, operations, trust, manifestValue };
}
function preservation() {
  return { schema: "worker-preservation-continuity-v1", baseline_id: BASELINE, device_identity_match: true,
    settings_match: true, authorization_high_water_match: true, mine_on_boot: false };
}
function state(context, extra = {}) {
  return { schema: "worker-serial-acceptance-v1", gateCommit: context.gate_commit, expectedFirmwareSourceCommit: context.firmware_commit,
    expectedAppElfSha256: context.app_elf_sha256, status: "ready", connected: true, running: false, heartbeatSuppressed: false, renewalsConfirmed: 0,
    deviceRestorationConfirmed: true, deviceLeaseInactive: true, preservation: preservation(), ...extra };
}
function cycle(context, index, extra = {}) {
  return { schema: "fixed-usb-cycle-report-v1", cycle: index, firmware_commit: context.firmware_commit, app_elf_sha256: context.app_elf_sha256,
    baseline_id: BASELINE, device_identity_match: true, settings_match: true, authorization_high_water_match: true, mine_on_boot: false,
    browser_released: true, flash_success: true, runtime_identity_match: true, cleanup_complete: true, probe_request_bytes: 65536, probe_response_bytes: 65536, ...extra };
}
function qualification(window, done = false) {
  const limit = WINDOW_MS[window];
  return { schema: "worker-qualification-v1", generation: 1, active_ms: done ? limit - 4000 : 1000,
    active_limit_ms: limit, shutdown_budget_ms: 15550, work_gate_remaining_ms: done ? 0 : limit - 16550,
    generation_elapsed_ms: done ? limit - 1000 : 3000, budget_reserved_ms: WINDOW_MS.slice(0, window + 1).reduce((a, b) => a + b, 0),
    budget_complete: window === 2 && done, submitted: 3, accepted: 2, rejected: 0, nonce_work_correlations: 4, work_dispatched: 5,
    last_valid_heartbeat_ms: limit - 4000, gate_closed_ms: done ? limit - 1200 : null, shutdown_started_ms: done ? limit - 1100 : null,
    safe_stop_stage: done ? "fan_paused" : "not_started", safe_stop_complete: done, revocation_reason: done ? "heartbeat_timeout" : "none",
    voltage_volts: 5, power_watts: 10, chip_temp_celsius: 40, fan_rpm: 1500, voltage_fresh: true, power_fresh: true,
    temperature_fresh: true, fan_fresh: true, watchdog_alive: true, mine_on_boot: false };
}
async function serverFixture(t, completedCycles = false) {
  const f = await fixture(t);
  await preflight(f.options, f.operations);
  const context = await loadContext(f.options.privateRoot);
  if (completedCycles) for (let index = 1; index <= 20; index += 1) await writeNew(resolve(f.options.privateRoot, `cycle-${index}.json`), cycle(context, index));
  let reads = 0, signs = 0, clock = 1000000;
  const server = await createSupervisor({ ...f.options, context, poolCredentials: "unused-fixture" }, { verifyFrozen: async () => undefined,
    now: () => clock, readPool: async () => { reads += 1; return { endpoint: "stratum+tcp://fixture.invalid:3333/", username: "private-fixture-owner", password: "private-fixture-password" }; },
    sign: async (operation) => { signs += 1; return { profile: "bwg-worker-lease-authorization-artifact/0.1", operation, authorization: "fixture-signature-only" }; } });
  server.listen(0, "127.0.0.1"); await once(server, "listening");
  t.after(async () => { server.closeAllConnections(); await new Promise((resolveClose) => server.close(resolveClose)); });
  const origin = `http://127.0.0.1:${server.address().port}`;
  const request = (route, body, method = body === undefined ? "GET" : "POST", otherOrigin = origin) => fetch(`${origin}${route}`, {
    method, headers: { Origin: otherOrigin, "Content-Type": "application/json" }, body: body === undefined ? undefined : JSON.stringify(body) });
  return { ...f, context, request, counts: () => ({ reads, signs }), tick: (value) => { clock = value; } };
}

test("preflight verifies all eight V4 artifacts and pins one campaign across retries", async (t) => {
  const f = await fixture(t);
  await preflight(f.options, f.operations);
  const first = await loadContext(f.options.privateRoot);
  await preflight({ ...f.options, privateRoot: resolve(f.base, "attempt-002") }, f.operations);
  const second = await loadContext(resolve(f.base, "attempt-002"));
  assert.equal(first.artifacts.length, 8);
  assert.equal(first.campaign_id, second.campaign_id);
  await assert.rejects(preflight(f.options, f.operations));
});
test("V3 and NVS-erasing update geometry fail before an artifact root exists", async (t) => {
  const f = await fixture(t);
  await writeFile(f.options.manifest, JSON.stringify({ ...f.manifestValue, schema_version: 3 }));
  await assert.rejects(packageSnapshot(f.options.firmwareRoot, f.options.manifest, SOURCE));
  f.manifestValue.update_segments[1].offset = 0x9000;
  await writeFile(f.options.manifest, JSON.stringify(f.manifestValue));
  await assert.rejects(preflight(f.options, f.operations));
  assert(!await readdir(f.base).then((entries) => entries.includes("attempt-001")));
});
test("public trust permits merged keys but rejects undeployed authority and private fields", async (t) => {
  const f = await fixture(t);
  admitTrust(f.trust, f.trust);
  assert.throws(() => admitTrust(f.trust, publicTrust()));
  const privateKey = structuredClone(f.trust); privateKey.updateAuthority.keys[0].d = "forbidden";
  assert.throws(() => admitTrust(privateKey, f.trust));
});
test("waiting server and public context do not start credential TTL or read pool input", async (t) => {
  const f = await serverFixture(t);
  const initial = await (await f.request("/supervisor-state")).json();
  assert.equal(initial.authority_context_active, false);
  const context = await (await f.request("/context")).json();
  assert.deepEqual(Object.keys(context).sort(), ["expectedAppElfSha256", "expectedFirmwareSourceCommit", "expectedGateCommit", "trust"].sort());
  f.tick(5000000);
  const scope = await (await f.request("/activate", {})).json();
  assert.equal(scope.retentionExpiryUnixSeconds, 5000 + 86400);
  assert.deepEqual(f.counts(), { reads: 0, signs: 0 });
});
test("mining signing is blocked until all twenty cycle receipts exist", async (t) => {
  const f = await serverFixture(t);
  await f.request("/activate", {});
  const response = await f.request("/authorization-context", { controlSessionBindingSha256: BINDING });
  assert.equal(response.status, 400);
  assert.deepEqual(f.counts(), { reads: 0, signs: 0 });
});
test("valid completed cycles permit consume-once in-memory grants without secret files", async (t) => {
  const f = await serverFixture(t, true);
  await f.request("/record", { state: state(f.context) });
  await f.request("/activate", {});
  assert.equal((await f.request("/authorization-context", { controlSessionBindingSha256: BINDING })).status, 200);
  const artifacts = await (await f.request("/window-artifacts")).json();
  assert.equal(artifacts.grant.stratum.username, "private-fixture-owner");
  assert.equal(artifacts.grant.acceptanceCampaign.id, f.context.campaign_id);
  assert.equal((await f.request("/window-artifacts")).status, 400);
  assert.equal((await f.request("/authorization-context", { controlSessionBindingSha256: BINDING })).status, 400);
  for (const entry of await readdir(f.options.privateRoot)) {
    const text = await readFile(resolve(f.options.privateRoot, entry), "utf8");
    assert(!text.includes("private-fixture-owner") && !text.includes("private-fixture-password") && !text.includes("fixture.invalid"));
  }
});
test("cross-origin activation and unknown evidence fields are rejected", async (t) => {
  const f = await serverFixture(t);
  assert.equal((await f.request("/activate", {}, "POST", "https://unrelated.invalid")).status, 400);
  assert.equal((await f.request("/record", { state: { ...state(f.context), rawPayload: "forbidden" } })).status, 400);
  assert.equal((await f.request("/window-artifacts", undefined, "GET", "https://unrelated.invalid")).status, 400);
});
test("cycle comparison rejects page reload, settings drift, and raw fingerprints", () => {
  const context = { firmware_commit: SOURCE, app_elf_sha256: "d".repeat(64) };
  const first = validateCycle(cycle(context, 1), context);
  assert.throws(() => validateCycle(cycle(context, 2, { baseline_id: Buffer.alloc(16, 9).toString("base64url") }), context, first));
  assert.throws(() => validateCycle(cycle(context, 2, { settings_match: false }), context, first));
  assert.throws(() => validateCycle(cycle(context, 2, { settings_sha256: "f".repeat(64) }), context, first));
});
test("heartbeat loss requires its actual device cause and the three-second bound", () => {
  const context = { gate_commit: GATE, firmware_commit: SOURCE, app_elf_sha256: "d".repeat(64) };
  const initial = state(context, { status: "running", running: true, heartbeatSuppressed: true, qualification: qualification(2) });
  const final = state(context, { status: "closed", connected: false, qualification: qualification(2, true) });
  validateState(initial, context); validateState(final, context);
  const records = [{ sequence: 1, state: initial }, { sequence: 2, state: final }], fault = { window: 2, generation: 1, after_sequence: 1, kind: "heartbeats_suppressed" };
  assert.equal(judgeWindow(2, records, fault).browser_report_accepted, true);
  final.qualification.revocation_reason = "unsafe_observation";
  assert.throws(() => judgeWindow(2, records, fault));
  final.qualification.revocation_reason = "heartbeat_timeout";
  final.qualification.shutdown_started_ms += 1000;
  assert.throws(() => judgeWindow(2, records, fault));
});

test("terminal cooling, generation binding, and the statistical share exception remain distinct", () => {
  const context = { gate_commit: GATE, firmware_commit: SOURCE, app_elf_sha256: "d".repeat(64) };
  const initial = state(context, { running: true, renewalsConfirmed: 1, qualification: qualification(0) });
  const final = state(context, { renewalsConfirmed: 1, qualification: { ...qualification(0, true), accepted: 0 } });
  const records = [{ sequence: 1, state: initial }, { sequence: 2, state: final }];
  const result = judgeWindow(0, records);
  assert.equal(result.accepted_share_verified, false);
  assert.equal(result.unverified_reason, "no_accepted_share_within_budget");
  assert.equal(result.budget_reserved_ms, 180000);
  final.qualification.safe_stop_stage = "fan_full";
  assert.throws(() => judgeWindow(0, records));
  final.qualification.safe_stop_stage = "fan_paused";
  final.qualification.fan_fresh = false;
  assert.throws(() => judgeWindow(0, records));
  final.qualification.fan_fresh = true;
  final.qualification.chip_temp_celsius = 46;
  assert.throws(() => judgeWindow(0, records));
  initial.qualification = qualification(2); initial.heartbeatSuppressed = true;
  final.qualification = qualification(2, true);
  assert.throws(() => judgeWindow(2, records, { window: 2, generation: 2, after_sequence: 1, kind: "heartbeats_suppressed" }));
});
test("signer plans preserve campaign and never extend the fixed three windows", async () => {
  const seen = [];
  for (let index = 0; index < 3; index += 1) {
    const result = await signWindow({ campaignId: BASELINE, index, challengeId: "challenge_fixture", binding: BINDING,
      stratum: { endpoint: "stratum+tcp://fixture.invalid:3333/", username: "fixture", password: "fixture" },
      sign: async (operation, input) => { seen.push(input); return { profile: "bwg-worker-lease-authorization-artifact/0.1", operation, authorization: "fixture" }; } });
    assert.equal(result.grant.acceptanceCampaign.maximumActiveMilliseconds, WINDOW_MS[index]);
    assert(result.renewals.every((renewal) => !Object.hasOwn(renewal, "acceptanceCampaign")));
  }
  assert.equal(seen.filter((input) => input.operation === "start").length, 3);
});
test("CLI rejects commands with unrelated extra options before source or key access", async () => {
  await assert.rejects(main(["judge", "--private-root", "/missing", "--window", "0", "--pool-credentials", "/forbidden"]));
});
