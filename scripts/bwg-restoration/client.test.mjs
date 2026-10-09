import test from "node:test";
import assert from "node:assert/strict";
import { context, pageState } from "./fixtures.test-helper.mjs";
import { parseRecord } from "./records.mjs";

const ADMISSION = Object.freeze({ stage: "idle", firstFailure: "none", readiness: 7 });

/** Load the browser client against a minimal page: a fake DOM, a fake `workerRestoration` and a recording fetch. */
async function loadClient() {
  const posted = [];
  globalThis.document = { createElement: () => ({}), body: { append: () => undefined } };
  globalThis.window = { workerRestoration: { admissionDiagnostic: async () => ({ admission: { ...ADMISSION } }),
    bootReview: async () => ({ schema: "worker-boot-review-v1", resetCause: "power_on" }),
    state: () => pageState({ admission: { ...ADMISSION }, connected: true, status: "ready" }) } };
  globalThis.fetch = async (route, init) => {
    posted.push({ route, body: JSON.parse(init.body) });
    return { ok: true, json: async () => ({ recorded: true }) };
  };
  await import(`./client.mjs?instance=${Date.now()}`);
  return { supervisor: globalThis.window.restorationSupervisor, posted };
}

test("the supervisor client runs admissionDiagnostic and records its closed value", async () => {
  // Arrange
  const { supervisor, posted } = await loadClient();
  // Act
  const value = await supervisor.run("admissionDiagnostic");
  await supervisor.flush();
  // Assert
  assert.deepEqual(value, { admission: ADMISSION });
  assert.deepEqual(posted.map((item) => item.route), ["/record"]);
  assert.deepEqual(parseRecord(posted[0].body, context).result, { admission: ADMISSION });
});

test("the supervisor client runs bootReview and records its closed value", async () => {
  // Arrange
  const { supervisor, posted } = await loadClient();
  // Act
  const value = await supervisor.run("bootReview");
  await supervisor.flush();
  // Assert
  assert.deepEqual(value, { schema: "worker-boot-review-v1", resetCause: "power_on" });
  assert.deepEqual(parseRecord(posted[0].body, context).result, value);
});
