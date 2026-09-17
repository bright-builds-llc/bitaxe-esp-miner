import assert from "node:assert/strict";
import { mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";
import { proof, writeNew } from "../str005-noise-serial/files.mjs";
import { inspectShareStart, validateStartTiming } from "./start-observation.mjs";
import { sha256 } from "./values.mjs";

test("page timing preserves independent clocks and exact invocation/reply boundaries", () => {
  // Arrange.
  const timing = { fixtureRequestAtPageMs: 0.25, startInvokedAtPageMs: 10000.25, startRepliedAtPageMs: 40000.25 };
  // Act / Assert.
  assert.doesNotThrow(() => validateStartTiming(timing));
  assert.throws(() => validateStartTiming({ ...timing, startInvokedAtPageMs: 10000.26 }), { code: "v2_start_fixture_deadline" });
  assert.throws(() => validateStartTiming({ ...timing, startRepliedAtPageMs: 40000.26 }), { code: "v2_start_reply_deadline" });
  assert.throws(() => validateStartTiming({ ...timing, startRepliedAtPageMs: 10000 }), { code: "v2_start_reply_deadline" });
  assert.throws(() => validateStartTiming({ ...timing, startRepliedAtPageMs: Infinity }), { code: "v2_start_page_clock" });
});

test("Start inspector joins exact source files and fresh idle identity without pool inputs", async t => {
  // Arrange: isolated software receipt component, not a complete Share acceptance fixture.
  const root = await mkdtemp(join(tmpdir(), "v2-start-observed-")); t.after(() => rm(root, { recursive: true }));
  const context = { scope: "share", client_sha256: "a".repeat(64) };
  await writeNew(join(root, "fixture-ready.json"), { readyAtMs: 500000 });
  await writeNew(join(root, "consumed.json"), { atHostMs: 500010 });
  const states = [{ sequence: 1, phase: "candidate", atHostMs: 520000,
    state: { running: true, qualification: { generation: 4 }, heartbeatSuppressed: false } }];
  const devices = [{ sequence: 1, atHostMs: 520002, record: { bootOrdinal: 3, workerGeneration: 4, serialTransportEpoch: 5, admittedAtDeviceUs: 7100000 } }];
  const value = { schema: "str005-v2-share-start-observed-v1", contextSha256: sha256(JSON.stringify(context)), clientSha256: context.client_sha256,
    fixtureReadySha256: (await proof(root, "fixture-ready.json")).sha256, consumedSha256: (await proof(root, "consumed.json")).sha256,
    atHostMs: 520001, observedStateSequence: 1, bootOrdinal: 3, workerGeneration: 4, serialTransportEpoch: 5, networkObservedAtDeviceUs: 7000000,
    timing: { fixtureRequestAtPageMs: 10, startInvokedAtPageMs: 20, startRepliedAtPageMs: 20020 } };
  const save = item => writeFile(join(root, "share-start-observed.json"), `${JSON.stringify(item)}\n`, { mode: 0o600 });
  await save(value);
  // Act / Assert: 20-second reply and unrelated page/host/device origins are valid.
  assert.deepEqual(await inspectShareStart(root, context, states, devices), value);
  for (const patch of [{ clientSha256: "b".repeat(64) }, { consumedSha256: "c".repeat(64) }, { workerGeneration: 7 },
    { networkObservedAtDeviceUs: 7200000 }, { networkObservedAtDeviceUs: 1000000 }, { atHostMs: 520003 }]) {
    await save({ ...value, ...patch });
    await assert.rejects(inspectShareStart(root, context, states, devices), { code: "v2_share_start_source_join" });
  }
});
