import assert from "node:assert/strict";
import { readFile, writeFile, rename } from "node:fs/promises";
import { resolve } from "node:path";
import test from "node:test";
import { completedFixture } from "./completed-fixture.mjs";
import { readDeviceJournal, readJournal } from "./journal.mjs";
import { inspectExecution, inspectRestoration } from "./execution-inspect.mjs";
import { faultFacts } from "./execution-restoration.mjs";

async function mutate(f, name, change, action) {
  const path = resolve(f.root, name), original = await readFile(path), value = JSON.parse(original);
  change(value); await writeFile(path, `${JSON.stringify(value, null, 2)}\n`, { mode: 0o600 });
  try { await action(); } finally { await writeFile(path, original, { mode: 0o600 }); }
}

test("read-only execution joins reject receipt drift, copied source rows and completion before actual exit", async t => {
  // Arrange: synthetic device facts through the real filesystem producers.
  const f = await completedFixture(t), states = await readJournal(f.root, f.context), devices = await readDeviceJournal(f.root, f.context);
  const inspect = () => inspectExecution(f.root, f.context, states, devices);
  // Act / Assert: each independent mutation leaves the original fixture intact.
  assert.equal((await inspect()).protocolInput.scope, "channel");
  for (const [file, change, code] of [
    ["job-receipt.json", value => value.sequence++, "v2_execution_receipt_order"],
    ["job-receipt.json", value => value.producerSha256 = "f".repeat(64), "v2_execution_receipt_binding"],
    ["job-receipt.json", value => value.facts.connectionId = "A".repeat(22), "v2_execution_receipt_facts"],
    ["connection.json", value => value.facts.readinessSha256 = "f".repeat(64), "v2_connection_source_join"],
    ["connection.json", value => value.facts.deviceObservationSequence = 999, "v2_execution_connection_source"],
    ["protocol-complete.json", value => value.fixtureTerminalSha256 = "f".repeat(64), "v2_protocol_completion_join"],
    ["protocol-complete.json", value => value.atHostMs = 0, "v2_protocol_completion_join"],
    ["start.claim.json", value => value.workerGeneration++, "v2_channel_claim_join"],
    ["start.claim.json", value => value.networkObservedAtDeviceUs = 2000, "v2_channel_claim_join"],
    ["restoration.json", value => value.recordSha256 = "f".repeat(64), "v2_restoration_join"],
    ["restoration.json", value => value.closedSequence = value.observedSequence, "v2_restoration_join"],
  ]) await mutate(f, file, change, () => assert.rejects(inspect, { code }));
  const path = resolve(f.root, "fixture-run/fixture-events.json");
  await rename(path, `${path}.held`);
  try { await assert.rejects(inspect, { code: "ENOENT" }); } finally { await rename(`${path}.held`, path); }
});

test("restoration-only review never grants protocol acceptance and requires returned resources", async t => {
  // Arrange.
  const f = await completedFixture(t), states = await readJournal(f.root, f.context), devices = await readDeviceJournal(f.root, f.context);
  // Act.
  const result = await inspectRestoration(f.root, f.context, states, devices);
  // Assert.
  assert.equal(result.schema, "str005-v2-restoration-v1");
  assert.equal(Object.hasOwn(result, "acceptedProtocol"), false);
  await mutate(f, "restoration.json", value => value.state.connected = false,
    () => assert.rejects(inspectRestoration(f.root, f.context, states, devices), { code: "v2_restoration_join" }));
});

test("fault projection takes native atoms and rejects late shutdown before any receipt", () => {
  // Arrange.
  const record = { workerGeneration: 4, poolSessionGeneration: 5, serialTransportEpoch: 6, poolTransportEpoch: 7 };
  const state = { qualification: { generation: 4, revocation_reason: "heartbeat_timeout", last_valid_heartbeat_ms: 10000,
    gate_closed_ms: 12800, shutdown_started_ms: 12801 } };
  const claim = { selectedDeviceAckSha256: "a".repeat(64), requestedAtHostMs: 20 };
  const confirmed = { headroom: { headroomObservedAtDeviceUs: 10000000, leaseRemainingMs: 5000, workGateRemainingMs: 5000 } };
  // Act.
  const facts = faultFacts(record, state, claim, confirmed, { tailMs: 5100 });
  // Assert.
  assert.equal(facts.gateClosedAtDeviceUs, 12800000);
  assert.equal(facts.shutdownStartedAtDeviceUs, 12801000);
  state.qualification.shutdown_started_ms = 13001;
  assert.throws(() => faultFacts(record, state, claim, confirmed, { tailMs: 5100 }), { code: "v2_fault_safety_join" });
  state.qualification.last_valid_heartbeat_ms = null;
  assert.throws(() => faultFacts(record, state, claim, confirmed, { tailMs: 5100 }), { code: "v2_fault_native_stop" });
});
