import assert from "node:assert/strict";
import test from "node:test";
import { LISTENER_ARGS, parseListenerInventory, requirePoolListenerAbsent } from "./host-resources.mjs";

test("listener parser supports repeated PID fields, wildcard and IPv6 names", () => {
  const parsed = parseListenerInventory("p123\nn127.0.0.1:80\nn[::1]:81\np123\nn*:82\np456\nn[fe80::1%en0]:83\n");
  assert.deepEqual(parsed.map((entry) => entry.port), [80, 81, 82, 83]);
  for (const value of ["", "n*:80\n", "p1\ncunknown\nn*:80\n", "p1\nn999.1.1.1:80\n", "p1\nn*:65536\n", "p1\nn127.0.0.1:80->127.0.0.2:90\n"])
    assert.throws(() => parseListenerInventory(value));
});

test("private pool port never enters lsof argv or errors", () => {
  const privatePort = 54321, calls = [];
  const operations = { spawnSync(command, args, options) { calls.push({ command, args, options }); return { status: 0, signal: null, stdout: "p1\nn*:80\n", stderr: "" }; } };
  requirePoolListenerAbsent(privatePort, operations);
  assert.deepEqual(calls[0].args, [...LISTENER_ARGS]);
  assert(!JSON.stringify(calls).includes(String(privatePort)));
  assert.throws(() => requirePoolListenerAbsent(privatePort, { spawnSync: () => ({ status: 0, stdout: `p1\nn*:${privatePort}\n`, stderr: "", signal: null }) }),
    (error) => error.code === "v2_pool_listener_present" && !error.message.includes(String(privatePort)));
});

test("lsof unavailable, partial warning and malformed output never prove absence", () => {
  requirePoolListenerAbsent(54321, { spawnSync: () => ({ status: 1, signal: null, stdout: "", stderr: "" }) });
  for (const result of [{ status: 0, stdout: "", stderr: "" }, { status: 1, stdout: "", stderr: "private warning" },
    { status: 0, stdout: "p1\nn*:80\n", stderr: "partial inventory" }, { status: null, signal: "SIGKILL", stdout: "", stderr: "" },
    { error: new Error("private endpoint"), stdout: "", stderr: "" }]) {
    assert.throws(() => requirePoolListenerAbsent(54321, { spawnSync: () => result }));
  }
  assert.throws(() => requirePoolListenerAbsent(54321, { spawnSync: () => { throw new Error("synthetic-private:54321"); } }),
    (error) => error.message === "v2_pool_listener_unproved");
});
