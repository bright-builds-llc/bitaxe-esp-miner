import assert from "node:assert/strict";
import test from "node:test";
import { createServer } from "node:http";
import { once } from "node:events";
import { chmod, mkdtemp, realpath, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { resolve } from "node:path";
import { installMeasurementPage } from "./client.mjs";
import { serverAccountingRoute } from "./server.mjs";
import { createJournal, verifyAccounting } from "./journal.mjs";
import { legacyView } from "./context.mjs";
import { state as deviceState, ledger, original } from "../str005-noise-serial/test-fixture.mjs";
import { qualification } from "../str005-v2-serial/completed-fixture.mjs";
import { proof } from "../str005-noise-serial/files.mjs";
import { body, send } from "../fixed-usb-qualification/http.mjs";
const version = n => `usb-bootstrap-measure-context-v${n}`;
function context(schema = version(4)) { return { schema, package: { firmware_commit: "a".repeat(40), app_elf_sha256: "b".repeat(64) }, beforeSource: { firmware_commit: "c".repeat(40), app_elf_sha256: "d".repeat(64) }, gate: { commit: "e".repeat(40) }, originalCampaign: { id: "synthetic-original" }, expectedAccounting: { ledger, original } }; }
function state(c, phase) { const value = deviceState(legacyView(c, phase)); value.qualification = { ...qualification(), budget_reserved_ms: 240000 }; return value; }
async function setup(t, schema) {
  const root = await mkdtemp(resolve(await realpath(tmpdir()), "restored-accounting-")); await chmod(root, 0o700); t.after(() => rm(root, { recursive: true, force: true }));
  const c = context(schema), journal = await createJournal(root, c), events = []; let phase = "before", current = state(c, phase), observer = () => {}, possession = false;
  current.deviceRestorationConfirmed = false;
  const elements = [], element = id => { const value = { id, textContent: "", disabled: false, remove() {}, closest() { return null; } }; elements.push(value); return value; };
  element("state"); element("stop");
  const document = { getElementById: id => elements.find(value => value.id === id), querySelector: () => elements.find(value => value.id === "state"), querySelectorAll: () => [], createElement: () => element(""), body: { append() {} } };
  const server = createServer(async (request, response) => {
    try {
      const value = await body(request); events.push(request.url);
      if (request.url === "/candidate-context") { phase = "candidate"; return send(response, 200, {}); } // Simulated completed installation boundary.
      if (request.url === "/client-failure") return send(response, 200, { recorded: true });
      return send(response, 200, await serverAccountingRoute(root, c, journal, phase, request.url, value));
    } catch (error) { send(response, 400, { error: error.code ?? "test_failure" }); }
  });
  server.listen(0, "127.0.0.1"); await once(server, "listening"); t.after(async () => { server.closeAllConnections(); await new Promise(done => server.close(done)); });
  const post = (path, input) => fetch(`http://127.0.0.1:${server.address().port}${path}`, { method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify(input), signal: AbortSignal.timeout(5000) });
  // Simulated Gate/device boundary: Restore clears possession, and the existing review helper obtains it afresh.
  const gate = { state: () => structuredClone(current), refresh: async () => { events.push("refresh"); observer(); },
    reviewQualificationAttempts: async () => { events.push("fresh-possession"); possession = true; events.push("qualification-review"); return structuredClone(ledger); },
    reviewBudget: async () => { assert(possession, "budget must follow fresh possession"); events.push("budget-review"); return structuredClone(original); },
    configure() { current = state(c, "candidate"); current.status = "configured"; current.connected = false; current.serialOwnershipReleased = true; current.deviceRestorationConfirmed = false; observer(); },
    async close() { current.status = "closed"; current.connected = false; current.serialOwnershipReleased = true; observer(); },
    start() { assert.fail("Start forbidden"); }, loadWindow() { assert.fail("Load forbidden"); }, renew() { assert.fail("Renew forbidden"); } };
  const ui = installMeasurementPage(document, { workerAcceptance: gate }, (path, options) => post(path, JSON.parse(options.body)), class { constructor(callback) { observer = callback; } observe() {} });
  return { root, c, events, ui, post, journal, current: () => current, afterButton: () => document.getElementById("bootstrap-after"), click: action => document.getElementById(`bootstrap-${action}`).onclick(),
    reconnect() { current.status = "ready"; current.connected = true; current.serialOwnershipReleased = false; observer(); },
    restore() { current.status = "stopping"; observer(); possession = false; current.status = "baseline_confirmed"; current.deviceRestorationConfirmed = true; observer(); },
    mutate(change) { change(current); observer(); } };
}
test("production collector and server retain baseline_confirmed through fresh restored accounting", async t => {
  // Arrange
  const f = await setup(t);
  // Act
  await f.click("before"); await f.click("close"); await f.click("configure"); f.reconnect(); f.restore(); await f.click("after"); await f.click("close"); await f.ui.flush();
  // Assert
  assert.equal(f.events.filter(event => event === "/accounting").length, 2);
  const after = (await proof(f.root, "accounting-after.json")).value; assert.equal(after.state.status, "baseline_confirmed"); assert.equal(after.state.deviceRestorationConfirmed, true);
  assert.equal((await verifyAccounting(f.root, f.c)).last.state.status, "closed");
  assert.equal(f.events.filter(event => event === "fresh-possession").length, 2); assert.equal(f.events.filter(event => event === "budget-review").length, 2);
});

test("after export remains disabled until explicit restored safe state and disables after collection", async t => {
  const f = await setup(t);
  assert.equal(f.afterButton().disabled, true);
  await f.click("before"); await f.click("close"); await f.click("configure"); f.reconnect();
  assert.equal(f.afterButton().disabled, true); await f.click("after"); assert.equal(f.events.filter(v => v === "/accounting").length, 1);
  f.restore(); assert.equal(f.afterButton().disabled, false); await f.click("after"); assert.equal(f.afterButton().disabled, true);
  await f.click("after"); assert.equal(f.events.filter(v => v === "/accounting").length, 2);
});
test("ready after state is accepted only when explicit restoration is retained", async t => {
  const f = await setup(t); await f.click("before"); await f.click("close"); await f.click("configure"); f.reconnect(); f.restore();
  f.mutate(s => { s.status = "ready"; }); await f.click("after");
  assert.equal((await proof(f.root, "accounting-after.json")).value.state.status, "ready");
});
for (const [name, mutate] of Object.entries({
  missing_restoration: s => { s.deviceRestorationConfirmed = false; },
  disconnected: s => { s.connected = false; },
  released: s => { s.serialOwnershipReleased = true; },
  pending_restore: s => { s.status = "stopping"; },
  identity: s => { s.preservation.device_identity_match = false; },
  settings: s => { s.preservation.settings_match = false; },
  high_water: s => { s.preservation.authorization_high_water_match = false; },
  mine_on_boot: s => { s.preservation.mine_on_boot = true; },
  lease_active: s => { s.deviceLeaseInactive = false; },
  running: s => { s.running = true; },
  renewal: s => { s.renewalsConfirmed = 1; },
  suppressed: s => { s.heartbeatSuppressed = true; },
})) test(`page and server reject unsafe restored accounting: ${name}`, async t => {
  // Arrange
  const f = await setup(t); await f.click("before"); await f.click("close"); await f.click("configure"); f.reconnect(); f.restore();
  // Act
  f.mutate(mutate); await f.ui.flush().catch(error => assert.equal(error.message, "bootstrap_client_failed"));
  const response = await f.post("/accounting", { stage: "after", ledger, original, state: f.current() }); await response.json();
  // Assert
  assert.equal(f.afterButton().disabled, true); assert.equal(response.status, 400);
  await assert.rejects(proof(f.root, "accounting-after.json"), { code: "ENOENT" });
});
test("initial accounting remains ready-only even with explicit restoration", async t => {
  const f = await setup(t); f.restore(); await f.ui.flush();
  const response = await f.post("/accounting", { stage: "before", ledger, original, state: f.current() }); await response.json();
  assert.equal(response.status, 400); await assert.rejects(proof(f.root, "accounting-before.json"), { code: "ENOENT" });
});
for (const v of [1, 2, 3]) test(`historical v${v} accounting still rejects baseline_confirmed without reinterpretation`, async t => {
  const f = await setup(t, version(v)); await f.click("before"); await f.click("close"); await f.click("configure"); f.reconnect(); f.restore(); await f.ui.flush();
  const response = await f.post("/accounting", { stage: "after", ledger, original, state: f.current() }); await response.json();
  assert.equal(response.status, 400); await assert.rejects(proof(f.root, "accounting-after.json"), { code: "ENOENT" });
});
for (const mutation of ["baseline", "ledger", "work", "private"]) test(`server rejects changed ${mutation} despite restored status`, async t => {
  const f = await setup(t); await f.click("before"); await f.click("close"); await f.click("configure"); f.reconnect(); f.restore();
  if (mutation === "baseline") f.mutate(s => { s.preservation.baseline_id = Buffer.alloc(16, 9).toString("base64url"); });
  if (mutation === "work") f.mutate(s => { s.qualification.work_dispatched++; });
  if (mutation === "private") f.mutate(s => { s.privateSecret = "synthetic forbidden field"; });
  await f.ui.flush().catch(error => assert.equal(error.message, "bootstrap_client_failed"));
  const response = await f.post("/accounting", { stage: "after", ledger: mutation === "ledger" ? { ...ledger, total_charged_ms: ledger.total_charged_ms + 1 } : ledger, original, state: f.current() }); await response.json();
  assert.equal(response.status, 400); await assert.rejects(proof(f.root, "accounting-after.json"), { code: "ENOENT" });
});
test("duplicate restored receipt cannot overwrite accepted original bytes", async t => {
  const f = await setup(t); await f.click("before"); await f.click("close"); await f.click("configure"); f.reconnect(); f.restore(); await f.click("after");
  const before = await proof(f.root, "accounting-after.json"), response = await f.post("/accounting", { stage: "after", ledger, original, state: f.current() }); await response.json();
  assert.equal(response.status, 400); assert.equal((await proof(f.root, "accounting-after.json")).sha256, before.sha256);
});
