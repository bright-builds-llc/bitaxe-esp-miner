import assert from "node:assert/strict";
import test from "node:test";
import { example } from "./fixtures.mjs";
import { parseNoiseStatus } from "./device.mjs";
import { parseNoiseStatusV2 } from "./device-v2.mjs";
import { parseStartV2 } from "./contract-v2.mjs";

test("v2 cleanup uses a fixed admission horizon and permits measured cleanup longer than five seconds", () => {
  const value = example().device;
  value.schema = "worker-noise-diagnostic-status-v2";
  value.job.resources.deadlineAtUs = value.job.authorityDeadlineUs + 5000000;
  value.job.resources.releasedAtUs = 6009000;
  value.job.stages[7].atUs = 6009000;
  value.job.stages[7].durationUs = 6002000;
  value.job.terminal.decidedAtUs = 6010000;
  value.observation.observedAtUs = 6011000;
  assert.equal(parseNoiseStatusV2(value).job.resources.deadlineMet, true);
  assert.throws(() => parseNoiseStatus(value));
  value.schema = "worker-noise-diagnostic-status-v1";
  assert.throws(() => parseNoiseStatusV2(value));
});
test("v2 cannot accept cleanup after authority despite release before observation horizon", () => {
  const value = example().device;
  value.schema = "worker-noise-diagnostic-status-v2";
  value.job.resources.deadlineAtUs = value.job.authorityDeadlineUs + 5000000;
  value.job.terminal.decidedAtUs = value.job.authorityDeadlineUs + 1;
  value.observation.observedAtUs = value.job.terminal.decidedAtUs;
  assert.throws(() => parseNoiseStatusV2(value));
});
test("v2 Start does not silently reinterpret v1 payloads", () => {
  const value = example().start;
  assert.throws(() => parseStartV2(value));
  value.schema = "worker-noise-diagnostic-start-v2";
  assert.equal(parseStartV2(value).schema, "worker-noise-diagnostic-start-v2");
});
