import test from "node:test";
import assert from "node:assert/strict";
import { once } from "node:events";
import { mkdir, readFile, stat, writeFile } from "node:fs/promises";
import { resolve } from "node:path";
import { digest } from "../fixed-usb-qualification/contract.mjs";
import { finishScenario } from "./campaign.mjs";
import { BUNDLE, SCENARIOS } from "./contract.mjs";
import { baseline, context, fakeWatcherOperations, journal, pageState, passingInput, privateDirectory } from "./fixtures.test-helper.mjs";
import { parseActivation, parseCheckpoint, parseCompletion, parseCompletionNonce, parseReplay, parseRestorationConfiguration, parseWindow } from "./gate-shapes.test-helper.mjs";
import { createRestorationSupervisor } from "./server.mjs";

const binding = "A".repeat(43);
const POOL = { endpoint: "stratum+tcp://pool.invalid:3333/", username: "fixture-pool-user", password: "fixture-pool-password" };
const passed = (scenario) => ({ scenario, result: "passed", failures: [], facts: {}, carry: { stimulusBaseline: 0 } });
const B = "b".repeat(64);
const PHYSICAL_SCRIPT = [{ elapsed_ms: 10, event: "present", enumeration_sha256: "a".repeat(64), holder_count: 1, ready: false },
  { gate: 1, elapsed_ms: 1000, event: "absent" },
  { gate: 2, elapsed_ms: 8000, event: "reappeared", enumeration_sha256: B, enumeration_changed: true, holder_count: 0, ready: true },
  { gate: 3, elapsed_ms: 11000, event: "stable", enumeration_sha256: B, holder_count: 0, ready: true }];

async function supervisor({ script = PHYSICAL_SCRIPT } = {}) {
  const root = await privateDirectory("restoration-server-");
  const firmware = resolve(root, "firmware"), gate = resolve(root, "gate"), privateRoot = resolve(root, "attempt-001");
  await mkdir(resolve(firmware, "firmware/bitaxe/bwg"), { recursive: true });
  await mkdir(resolve(gate, "dist/worker-restoration"), { recursive: true });
  await mkdir(privateRoot, { mode: 0o700 });
  await writeFile(resolve(firmware, "firmware/bitaxe/bwg/deployment-trust.json"), "{\"profile\":\"fixture\"}");
  const page = "<!doctype html><title>restoration</title>", bundle = `// ${context.gate_commit}`;
  await writeFile(resolve(gate, "restoration.html"), page);
  await writeFile(resolve(gate, BUNDLE), bundle);
  const fake = await fakeWatcherOperations(root, script);
  const clock = { now: 1_000_000 }, signed = [], announced = [];
  let sequence = 0;
  const server = await createRestorationSupervisor({ privateRoot, context: { ...context, firmware_root: firmware, gate_root: gate,
    gate_page_sha256: digest(page), gate_bundle_sha256: digest(bundle), watcher: fake.binary } }, {
    verifyFrozen: async () => undefined, readPool: async () => ({ ...POOL }), spawn: fake.spawn, now: () => clock.now, announce: (line) => announced.push(line),
    sign: async (operation, input) => { signed.push(input); sequence += 1; return { profile: "bwg-worker-lease-authorization-artifact/0.1", operation, sequence,
      authorization: `signed-${operation}-${sequence}-artifact` }; },
  });
  server.listen(0, "127.0.0.1");
  await once(server, "listening");
  const origin = `http://127.0.0.1:${server.address().port}`;
  const call = async (path, maybeBody, { headers } = {}) => {
    const response = await fetch(origin + path, maybeBody === undefined ? { headers: headers ?? { "sec-fetch-site": "same-origin" } }
      : { method: "POST", headers: headers ?? { "content-type": "application/json", origin }, body: JSON.stringify(maybeBody) });
    return { status: response.status, value: response.headers.get("content-type") === "application/json" ? await response.json() : await response.text() };
  };
  const operator = (path, body) => call(path, body, { headers: { "content-type": "application/json" } });
  return { root: privateRoot, server, call, operator, signed, clock, announced, fake };
}

async function until(probe, timeoutMs = 5000) {
  const deadline = Date.now() + timeoutMs;
  for (;;) {
    const value = await probe();
    if (value) return value;
    if (Date.now() > deadline) throw new Error("timeout");
    await new Promise((done) => setTimeout(done, 20));
  }
}

const events = async (root) => (await readFile(resolve(root, "campaign-events.jsonl"), "utf8")).trim().split("\n").map((line) => JSON.parse(line));

test("the context is exactly the restoration configuration and the page loads the import-free client", async (t) => {
  // Arrange
  const { server, call } = await supervisor();
  t.after(() => server.close());
  // Act
  const config = await call("/context");
  const page = await call("/");
  const client = await call("/supervisor-client.mjs");
  // Assert
  assert.equal(parseRestorationConfiguration(config.value).restorationQualification, true);
  assert.ok(page.value.endsWith('<script type="module" src="/supervisor-client.mjs"></script>'));
  assert.equal(client.status, 200);
  assert.equal(/^\s*import\s|import\(/mu.test(client.value), false);
});

test("every connect within a scenario receives the same scope", async (t) => {
  // Arrange
  const { server, call } = await supervisor();
  t.after(() => server.close());
  // Act
  const first = parseActivation((await call("/activate", {})).value);
  const second = parseActivation((await call("/activate", {})).value);
  // Assert
  assert.deepEqual(first, second);
  assert.match(first.challengeId, /^challenge_/u);
});

test("a signed window matches the Gate restoration parser, carries no budget and is delivered once", async (t) => {
  // Arrange
  const { server, call, signed } = await supervisor();
  t.after(() => server.close());
  const scope = (await call("/activate", {})).value;
  // Act
  const authorized = await call("/authorization-context", { controlSessionBindingSha256: binding });
  const first = await call("/scenario-artifacts");
  const second = await call("/scenario-artifacts");
  // Assert
  assert.deepEqual(authorized.value, { authorization_context_saved: true });
  parseWindow(first.value);
  assert.equal(first.value.grant.challengeId, scope.challengeId);
  assert.deepEqual([first.value.renewals.length, second.value.error], [1, "artifacts_unavailable"]);
  assert.deepEqual(signed.map((input) => [input.operation, input.activeChallengeId === scope.challengeId, input.request.durationMilliseconds]),
    [["start", true, 60000], ["renew", true, 60000]]);
  assert.equal(["acceptanceCampaign", "qualificationAttempt", "soakAllowance", "hardwareProfile"].some((key) => key in signed[0].request), false);
});

test("a record that echoes a signed secret fails the attempt and is not stored", async (t) => {
  // Arrange
  const { root, server, call } = await supervisor();
  t.after(() => server.close());
  await call("/activate", {});
  await call("/authorization-context", { controlSessionBindingSha256: binding });
  const leaked = (await call("/scenario-artifacts")).value.grant.authorization;
  // Act
  const response = await call("/record", { operation: "statusReview", outcome: "ok", result: null, state: { leaked } });
  // Assert
  assert.equal(response.value.error, "credential_in_record");
  const result = JSON.parse(await readFile(resolve(root, "result.json"), "utf8")).result;
  assert.deepEqual([result.result, result.failure.category], ["unverified", "credential_in_record"]);
  await assert.rejects(readFile(resolve(root, "records.jsonl")), (error) => error.code === "ENOENT");
});

test("a completion review answers the exact Gate receipt and advances to the next scenario", async (t) => {
  // Arrange
  const { root, server, call } = await supervisor();
  t.after(() => server.close());
  await call("/activate", {});
  const input = passingInput("completion");
  // Act
  const nonce = parseCompletionNonce((await call("/completion-context", {})).value);
  const receipt = parseCompletion((await call("/completion-review", { nonce, reviews: input.reviews, final_state: input.finalState })).value);
  const replayed = await call("/completion-review", { nonce, reviews: input.reviews, final_state: input.finalState });
  // Assert
  assert.deepEqual(receipt, { result: "passed", scenario: "completion", cleanup_confirmed: true });
  assert.equal(replayed.value.error, "completion_challenge");
  assert.equal(server.campaign.scenario.name, "pause");
  assert.equal((await stat(resolve(root, "scenario-01-completion.json"))).mode & 0o777, 0o600);
});

test("N1 replays the reboot Start once under the persistent scope", async (t) => {
  // Arrange
  const { server, call } = await supervisor();
  t.after(() => server.close());
  for (const scenario of SCENARIOS.slice(0, 6)) finishScenario(server.campaign, passed(scenario));
  await call("/activate", {});
  await call("/checkpoint/ready", { scenario: "reboot", checkpoint: "awaiting_operator_ready" });
  await call("/authorization-context", { controlSessionBindingSha256: binding });
  const rebootStart = (await call("/scenario-artifacts")).value.grant;
  finishScenario(server.campaign, passed("reboot"));
  const scope = (await call("/activate", {})).value;
  const status = { operation: "statusReview", outcome: "ok", result: baseline("reboot"),
    state: pageState({ entries: journal(["connected", "status_reviewed:reboot"]), connected: true, status: "ready", device: baseline("reboot") }) };
  // Act
  const early = await call("/replay-artifact");
  const recorded = await call("/record", status);
  const first = await call("/replay-artifact");
  const second = await call("/replay-artifact");
  // Assert
  assert.equal(early.value.error, "n1_status_required");
  assert.deepEqual(recorded.value, { recorded: true });
  parseReplay(first.value);
  assert.deepEqual(first.value, { operation: "start", grant: rebootStart });
  assert.equal(rebootStart.challengeId, scope.challengeId);
  assert.equal(second.value.error, "replay_unavailable");
});

test("cross-origin requests are refused, including operator routes from a browser", async (t) => {
  // Arrange
  const { server, call } = await supervisor();
  t.after(() => server.close());
  // Act
  const page = await call("/activate", {}, { headers: { "content-type": "application/json", origin: "http://evil.invalid" } });
  const operator = await call("/checkpoint/cancel", { scenario: "completion" }, { headers: { "content-type": "application/json", origin: "http://evil.invalid" } });
  const artifacts = await call("/scenario-artifacts", undefined, { headers: { "sec-fetch-site": "cross-site" } });
  // Assert
  assert.deepEqual([page.value.error, operator.value.error, artifacts.value.error], ["origin_rejected", "origin_rejected", "origin_rejected"]);
});

test("the disconnect checkpoint orders watcher, removal, absence bound, restore token and stability before reconnect", async (t) => {
  // Arrange
  const { root, server, call, operator, clock, announced, fake } = await supervisor();
  t.after(() => server.close());
  for (const scenario of SCENARIOS.slice(0, 5)) finishScenario(server.campaign, passed(scenario));
  const scope = (await call("/activate", {})).value;
  const waiting = (await call("/supervisor-state")).value;
  const unsigned = await call("/authorization-context", { controlSessionBindingSha256: binding });
  await operator("/checkpoint/ready", { scenario: "disconnect", checkpoint: "awaiting_operator_ready" });
  await call("/authorization-context", { controlSessionBindingSha256: binding });
  await call("/scenario-artifacts");
  // Act
  const begun = parseCheckpoint((await call("/physical-window", { event: "begin" })).value);
  const blockedDuringRemoval = await call("/activate", {});
  await writeFile(fake.control, "1");
  await until(async () => (await call("/physical-window")).value.checkpoint === "absence_bounding");
  const early = (await call("/physical-window", { event: "arm" })).value;
  const announcedEarly = announced.length;
  clock.now += 5000;
  const armed = (await call("/physical-window", { event: "arm" })).value;
  const restoring = (await call("/supervisor-state")).value;
  await writeFile(fake.control, "2");
  await until(async () => (await call("/physical-window")).value.checkpoint === "stabilizing");
  const blockedWhileStabilizing = await call("/activate", {});
  await writeFile(fake.control, "3");
  await until(async () => (await call("/physical-window")).value.checkpoint === "reconnect_ready");
  const reconnected = (await call("/activate", {})).value;
  // Assert
  assert.deepEqual([waiting.checkpoint, waiting.human_checkpoint_armed, waiting.waiting_for_human_has_no_deadline, waiting.automated_bounds],
    ["awaiting_operator_ready", true, true, []]);
  assert.equal(unsigned.value.error, "operator_ready_required");
  assert.equal(begun.checkpoint, "remove_usb");
  assert.equal(blockedDuringRemoval.value.error, "physical_checkpoint_not_ready");
  assert.deepEqual([early.checkpoint, announcedEarly], ["absence_bounding", 0]);
  assert.equal(armed.checkpoint, "restore_usb");
  assert.deepEqual(announced, ["action_token=bwg-restoration-restore-watcher-armed-v1 response_required=false scenario=disconnect"]);
  assert.deepEqual([restoring.instruction, restoring.restore_watcher.response_required, restoring.human_checkpoint_armed],
    ["Reconnect the USB cable to the Ultra 205.", false, true]);
  assert.equal(blockedWhileStabilizing.value.error, "physical_checkpoint_not_ready");
  assert.deepEqual(reconnected, scope);
  const rows = (await events(root)).map((row) => row.event === "checkpoint" ? `checkpoint:${row.checkpoint}` : row.event);
  assert.ok(rows.indexOf("watcher_started") < rows.indexOf("checkpoint:remove_usb"));
  assert.ok(rows.indexOf("restore_watcher_armed") > rows.indexOf("checkpoint:remove_usb"));
  const stopped = (await events(root)).find((row) => row.event === "watcher_stopped");
  assert.equal(stopped.stopped_on_request, true);
});
