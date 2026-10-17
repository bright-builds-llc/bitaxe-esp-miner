import assert from "node:assert/strict";
import test from "node:test";
import { bapOperationPassed, gateAllowsOperation, runBapChange, runBapOperation, SOAK_BAP_CHANGES } from "./bap-changes.mjs";

const printed = (frames, summary) => [...frames, summary].map((line) => JSON.stringify(line)).join("\n") + "\n";
const frequency = SOAK_BAP_CHANGES[1].ops[0];

test("a frequency setting passes only on its own acknowledgement value", () => {
  // Arrange
  const summary = { outcome: "confirmed", error_code: null };
  // Act
  const exact = bapOperationPassed(frequency, summary, [{ command: "ACK", parameter: "frequency", value: "400.00" }]);
  const other = bapOperationPassed(frequency, summary, [{ command: "ACK", parameter: "frequency", value: "485.00" }]);
  // Assert
  assert.deepEqual([exact, other], [true, false]);
});

test("a silent fan setting passes only as sent silently", () => {
  // Arrange
  const fan = SOAK_BAP_CHANGES[2].ops[0];
  // Act
  const silent = bapOperationPassed(fan, { outcome: "sent_silently" }, []);
  const timedOut = bapOperationPassed(fan, { outcome: "timed_out" }, []);
  // Assert
  assert.deepEqual([silent, timedOut], [true, false]);
});

test("a read-back passes only with the expected token and value", () => {
  // Arrange
  const readBack = SOAK_BAP_CHANGES[0].ops[5];
  // Act
  const answered = bapOperationPassed(readBack, { outcome: "answered" }, [{ command: "RES", parameter: "block_found", value: "0" }]);
  const wrongToken = bapOperationPassed(readBack, { outcome: "answered" }, [{ command: "RES", parameter: "found_block", value: "0" }]);
  // Assert
  assert.deepEqual([answered, wrongToken], [true, false]);
});

test("an ASIC setting runs the client with its confirmation and allowance", async () => {
  // Arrange
  const calls = [];
  const run = async (binary, args) => { calls.push([binary, args]); return { stdout: printed([{ command: "ACK", parameter: "frequency", value: "400.00" }], { outcome: "confirmed" }) }; };
  // Act
  const result = await runBapOperation("/bin/bap_client", "/dev/cu.usbserial-0001", frequency, run);
  // Assert
  assert.equal(result.passed, true);
  assert.deepEqual(calls[0][1], ["--port", "/dev/cu.usbserial-0001", "set", "frequency", "--value", "400", "--confirm", "frequency", "--allow-asic-setting"]);
});

test("a refused setting keeps the client's printed outcome", async () => {
  // Arrange: the client exits 2 and still prints its frames.
  const run = async () => { throw Object.assign(new Error("exit 2"), { stdout: printed([{ command: "ERR", parameter: "frequency", value: "set_failed" }], { outcome: "rejected", error_code: "set_failed" }) }); };
  // Act
  const result = await runBapOperation("/bin/bap_client", "/dev/cu.usbserial-0001", frequency, run);
  // Assert
  assert.deepEqual([result.passed, result.outcome, result.error_code], [false, "rejected", "set_failed"]);
});

test("a change stops at its first failed operation", async () => {
  // Arrange
  const sent = [];
  const runOperation = async (binary, port, op) => { sent.push(op.parameter); return { passed: op.parameter !== "frequency" }; };
  // Act
  const change = await runBapChange("/bin/bap_client", "/dev/cu.usbserial-0001", SOAK_BAP_CHANGES[3].ops, { runOperation });
  // Assert
  assert.deepEqual([sent, change.maybeFailure], [["asic_voltage", "frequency"], "bap_operation_failed"]);
});

test("the plan never saves 485 MHz at 1100 mV", () => {
  // Arrange
  let saved = { frequency: 485, coreVoltage: 1200 };
  const visited = [];
  // Act
  for (const change of SOAK_BAP_CHANGES) {
    for (const op of change.ops.filter((candidate) => candidate.op === "set")) {
      if (op.parameter === "frequency") saved = { ...saved, frequency: Number(op.value) };
      if (op.parameter === "asic_voltage") saved = { ...saved, coreVoltage: Number(op.value) };
      visited.push(`${saved.frequency}/${saved.coreVoltage}`);
    }
  }
  // Assert
  assert.ok(!visited.includes("485/1100"));
  assert.equal(visited.at(-1), "485/1200");
});

const record = (overrides = {}, receivedAtUnixMs = 1000) => ({ receivedAtUnixMs,
  state: { running: true, qualification: { soak: { ordinal: 5 }, revocation_reason: "none", work_gate_remaining_ms: 180000, ...overrides } } });

test("an operation may go out while the soak mines with the gate open and a fresh record", () => {
  // Arrange / Act / Assert
  assert.equal(gateAllowsOperation(record(), 2000), true);
});

test("no operation goes out after revocation, near the gate's close, or on a stale record", () => {
  // Arrange / Act
  const revoked = gateAllowsOperation(record({ revocation_reason: "lease_or_budget_expired" }), 2000);
  const closing = gateAllowsOperation(record({ work_gate_remaining_ms: 59000 }), 2000);
  const stale = gateAllowsOperation(record(), 7000);
  // Assert
  assert.deepEqual([revoked, closing, stale], [false, false, false]);
});

test("a change stops before the operation the gate refuses", async () => {
  // Arrange: revocation lands after the first operation.
  const sent = [];
  let open = true;
  const runOperation = async (binary, port, op) => { sent.push(op.parameter); open = false; return { passed: true }; };
  // Act
  const change = await runBapChange("/bin/bap_client", "/dev/cu.usbserial-0001", SOAK_BAP_CHANGES[3].ops, { runOperation, allowed: () => open });
  // Assert
  assert.deepEqual([sent, change.maybeFailure], [["asic_voltage"], "bap_gate_closed"]);
});
