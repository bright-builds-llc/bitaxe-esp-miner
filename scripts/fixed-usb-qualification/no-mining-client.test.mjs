import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { createContext, runInContext } from "node:vm";

const client = await readFile(new URL("./no-mining-client.mjs", import.meta.url), "utf8");
function readyState() {
  return { status: "ready", connected: true, running: false, deviceLeaseInactive: true, deviceBaselineConfirmed: true,
    serialOwnershipReleased: false, renewalsConfirmed: 0, preservation: { device_identity_match: true,
      settings_match: true, authorization_high_water_match: true, mine_on_boot: false } };
}
function observerFixture(interrupt, before = readyState()) {
  const published = { textContent: JSON.stringify(before) }, posts = [];
  let internalReads = 0;
  const workerAcceptance = {
    state: () => { internalReads += 1; return { status: "internal_state_must_not_be_used" }; },
    interruptPendingStatusForQualification: () => interrupt(published),
  };
  const window = { workerAcceptance };
  const context = createContext({ window, document: { getElementById: () => undefined,
    createElement: () => ({}), body: { append: () => undefined }, querySelector: (query) => query === "#state" ? published : undefined },
    MutationObserver: class { observe() {} },
    fetch: async (path, options) => {
      posts.push({ path, input: JSON.parse(options.body) });
      return { ok: true, json: async () => ({ read_only_interruption_saved: true }) };
    } });
  runInContext(client, context);
  return { helper: window.noMiningSupervisor, posts, reads: () => internalReads };
}
const success = { schema: "worker-read-interruption-v1", interrupted: true, request_consumed: true, response_pending: true, ownership_released: true };

test("interruption helper captures published states and excludes mutable internal snapshots", async () => {
  // Arrange
  const before = readyState(), after = { ...before, status: "closed", connected: false, serialOwnershipReleased: true };
  const f = observerFixture(async (published) => { published.textContent = JSON.stringify(after); return success; }, before);
  // Act
  await f.helper.interruptReadOnlyStatus();
  // Assert
  assert.equal(f.reads(), 0);
  assert.deepEqual(f.posts, [{ path: "/read-only-interruption", input: { receipt: success, before, after } }]);
});
test("concurrent and later observer calls cannot issue a second interruption", async () => {
  // Arrange
  let calls = 0, release;
  const gateWait = new Promise((resolve) => { release = resolve; });
  const f = observerFixture(async (published) => {
    calls += 1;
    await gateWait;
    published.textContent = JSON.stringify({ ...readyState(), status: "closed", connected: false, serialOwnershipReleased: true });
    return success;
  });
  // Act
  const first = f.helper.interruptReadOnlyStatus();
  await assert.rejects(f.helper.interruptReadOnlyStatus(), /read_only_interruption_already_attempted/u);
  release();
  await first;
  // Assert
  await assert.rejects(f.helper.interruptReadOnlyStatus(), /read_only_interruption_already_attempted/u);
  assert.equal(calls, 1);
  assert.equal(f.posts.length, 1);
});
test("published nonbaseline state prevents the observer from invoking Gate", async () => {
  const f = observerFixture(() => assert.fail("no interruption is admitted"), { ...readyState(), deviceBaselineConfirmed: false });
  await assert.rejects(f.helper.interruptReadOnlyStatus(), /read_only_interruption_baseline/u);
  assert.equal(f.posts.length, 0);
});
