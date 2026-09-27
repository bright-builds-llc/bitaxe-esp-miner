import assert from "node:assert/strict";
import { once } from "node:events";
import { connect } from "node:net";
import test from "node:test";
import { ledger, original, state } from "../str005-noise-serial/test-fixture.mjs";
import { createRecoveryServer } from "./recovery-server.mjs";

const context = { scope: "share", attemptId: Buffer.alloc(16, 1).toString("base64url"),
  gate_commit: "b".repeat(40), firmware_commit: "a".repeat(40), app_elf_sha256: "c".repeat(64) };
async function setup(t, maybePersist) {
  const stored = [];
  const server = createRecoveryServer({ root: "/unused", context, page: "", bundle: Buffer.from(""), client: Buffer.from(""), trust: {} },
    { persist: async (stage, value) => { if (maybePersist) await maybePersist(stage, value); stored.push({ stage, value }); },
      validateDiagnostics: async value => value });
  server.listen(0, "127.0.0.1");
  await once(server, "listening");
  const port = server.address().port, origin = `http://127.0.0.1:${port}`;
  t.after(async () => { if (server.listening) await server.release(); });
  const post = async (path, value, maybeOrigin = origin) => {
    const response = await fetch(`${origin}${path}`, { method: "POST", headers: { "Content-Type": "application/json", Origin: maybeOrigin },
      body: JSON.stringify(value) });
    return { status: response.status, body: await response.json() };
  };
  return { server, stored, port, post, part: (stage, value) => post("/part", { stage, value }) };
}

test("real recovery HTTP server rejects grant, Start and flash routes", async t => {
  // Arrange
  const fixture = await setup(t);
  // Act
  const responses = await Promise.all(["/grant", "/start", "/flash", "/start/claim", "/issuance"].map(path => fixture.post(path, {})));
  // Assert
  assert.ok(responses.every(response => response.status === 400));
  assert.deepEqual(fixture.stored, []);
});

test("failed persistence does not poison later accounting and restoration collection", async t => {
  // Arrange
  const fixture = await setup(t, stage => { if (stage === "diagnostics") throw Error("synthetic_write_failure"); });
  // Act
  const diagnostic = await fixture.post("/diagnostic-export", { schema: "worker-diagnostic-export-v1", observations: [] });
  const accounting = await fixture.part("ledger", ledger);
  const budget = await fixture.part("original_budget", original);
  const restoration = await fixture.part("state", state(context));
  const closed = await fixture.part("closed", state(context, "candidate", true));
  // Assert
  assert.equal(diagnostic.status, 400);
  assert.ok([accounting, budget, restoration, closed].every(response => response.status === 200));
  assert.deepEqual(fixture.stored.map(row => row.stage), ["ledger", "original_budget", "state", "closed"]);
});

test("invalid secret-bearing input is neither persisted nor reflected in HTTP errors", async t => {
  // Arrange
  const fixture = await setup(t);
  const secret = "private-value-must-not-escape";
  // Act
  const response = await fixture.part("ledger", { ...ledger, password: secret });
  const later = await fixture.part("ledger", ledger);
  // Assert
  assert.equal(response.status, 400);
  assert.equal(JSON.stringify(response).includes(secret), false);
  assert.equal(JSON.stringify(fixture.stored).includes(secret), false);
  assert.equal(later.status, 200);
});

test("server release closes the actual loopback listener", async t => {
  // Arrange
  const fixture = await setup(t);
  const closed = once(fixture.server, "close");
  // Act
  await fixture.server.release(); await closed;
  const socket = connect({ host: "127.0.0.1", port: fixture.port });
  const [error] = await once(socket, "error");
  socket.destroy();
  // Assert
  assert.equal(fixture.server.listening, false);
  assert.equal(error.code, "ECONNREFUSED");
});

test("finished recovery rejects repeated and new writes", async t => {
  // Arrange
  const fixture = await setup(t);
  await fixture.part("ledger", ledger);
  // Act
  const finished = await fixture.part("finished", { failures: ["status"] });
  const repeated = await fixture.part("ledger", ledger);
  const late = await fixture.part("original_budget", original);
  const refinish = await fixture.part("finished", { failures: [] });
  // Assert
  assert.equal(finished.status, 200);
  assert.ok([repeated, late, refinish].every(response => response.status === 400));
  assert.deepEqual(fixture.stored.map(row => row.stage), ["ledger", "finished"]);
});

test("cross-origin request cannot persist recovery evidence", async t => {
  const fixture = await setup(t);
  const result = await fixture.post("/part", { stage: "ledger", value: ledger }, "https://foreign.invalid");
  assert.equal(result.status, 400);
  assert.deepEqual(fixture.stored, []);
});
