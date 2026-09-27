import assert from "node:assert/strict";
import test from "node:test";
import { installMeasurementPage } from "./client.mjs";
function pageFixture({ delayed = false } = {}) {
  const elements = [], calls = [];
  const element = (id = "", tag = "button", text = "") => {
    const e = { id, tag, textContent: text, removed: false, parent: null, remove() { this.removed = true; }, closest() { return this.parent; } }; elements.push(e); return e;
  };
  for (const id of ["connect", "close", "stop", "prepare", "load", "start", "probe", "suppress", "arm-foreground"]) element(id);
  for (const id of ["configuration", "authorization-context"]) element(id, "input").parent = element(`${id}-label`, "label");
  element("signed-copy", "p", "Signed private windows arrive once."); element("intro", "p", "Signed leases stay in memory."); element("state", "pre");
  const document = { getElementById: id => elements.find(e => e.id === id && !e.removed), querySelector: id => elements.find(e => `#${e.id}` === id),
    querySelectorAll: tag => elements.filter(e => e.tag === tag), createElement: tag => element("", tag), body: { append() {} } };
  let observe = () => {}, state = { expectedFirmwareSourceCommit: "a".repeat(40), status: "ready", connected: true, running: false, deviceBaselineConfirmed: true,
    deviceLeaseInactive: true, renewalsConfirmed: 0, preservation: { baseline_id: "private-test", device_identity_match: true, settings_match: true, authorization_high_water_match: true, mine_on_boot: false } };
  const window = { workerAcceptance: { state: () => structuredClone(state), refresh: async () => observe(),
    reviewQualificationAttempts: async () => (calls.push("ledger"), { pending: false }), reviewBudget: async () => (calls.push("budget"), { pending: false }),
    close: async () => { calls.push("close"); state.status = "closed"; state.connected = false; state.serialOwnershipReleased = true; observe(); },
    configure: () => { calls.push("configure"); state.status = "configured"; observe(); }, connect: () => assert.fail("must remain native gesture") } };
  const fetch = async (path, options) => { calls.push(path); return { ok: true, json: async () => path === "/accounting-context" ? { campaignId: "private-original" } : { recorded: true } }; };
  const ui = installMeasurementPage(document, window, fetch, class { constructor(callback) { observe = delayed ? () => queueMicrotask(callback) : callback; } observe() {} });
  return { elements, calls, document, ui, click: action => document.getElementById(`bootstrap-${action}`).onclick(),
    reconnect() { state.status = "ready"; state.connected = true; state.serialOwnershipReleased = false; observe(); } };
}
test("page removes config/signing/old close affordances after original controls exist", () => {
  const f = pageFixture();
  for (const id of ["close", "prepare", "load", "start", "probe", "suppress", "arm-foreground", "configuration-label", "authorization-context-label", "signed-copy"])
    assert.equal(f.elements.find(e => e.id === id).removed, true, id);
  assert(f.document.getElementById("connect")); assert(f.document.getElementById("stop"));
});
test("native page actions retain baseline and flush both serial closures", async () => {
  const f = pageFixture();
  await f.click("before"); await f.click("close"); await f.click("configure"); f.reconnect(); await f.click("after"); await f.click("close"); await f.ui.flush();
  assert.equal(f.calls.filter(v => v === "close").length, 2); assert.equal(f.calls.filter(v => v === "/accounting").length, 2);
  assert(f.calls.indexOf("configure") > f.calls.indexOf("close")); assert(!f.calls.some(v => /sign|fixture|start|renew/u.test(v)));
});

test("close flush waits for delayed MutationObserver delivery", async () => {
  const f = pageFixture({ delayed: true });
  await f.click("before"); const count = f.calls.filter(value => value === "/record").length;
  await f.click("close");
  assert(f.calls.filter(value => value === "/record").length > count);
  assert.equal(f.calls.at(-1), "/record");
});
