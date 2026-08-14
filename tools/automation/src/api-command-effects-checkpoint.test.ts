import assert from "node:assert/strict";
import { appendFile, chmod, mkdtemp, mkdir, stat, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import {
  formatOperatorCheckpointSignal,
  superviseOperatorCheckpoints,
  type OperatorCheckpointSignal,
} from "./api-command-effects-checkpoint.js";
import { internalCommandSpec } from "./contracts.generated.js";
import { createLocalProcessPort } from "./process.js";

const ok = { exitCode: 0, stdout: "", stderr: "", timedOut: false } as const;

async function privateCheckpoint(root: string, checkpoint: "ready" | "rendered" | "cleared"): Promise<void> {
  const output = path.join(root, `identify-${checkpoint}.required.json`);
  await writeFile(output, `${JSON.stringify({
    schema: "bitaxe-identify-checkpoint-v3",
    checkpoint,
    status: "required",
  })}\n`, { mode: 0o600 });
  await chmod(output, 0o600);
}

test("a partially published document is not misclassified before completion", async () => {
  // Arrange
  const root = await mkdtemp(path.join(os.tmpdir(), "api-command-checkpoint-partial-"));
  const ready = path.join(root, "identify-ready.required.json");
  await writeFile(ready, "{", { mode: 0o600 });
  let settle = (_outcome: typeof ok): void => undefined;
  const campaign = new Promise<typeof ok>((resolve) => { settle = resolve; });
  const signals: OperatorCheckpointSignal[] = [];
  const supervised = superviseOperatorCheckpoints(campaign, root, (checkpoint) => {
    signals.push(checkpoint);
  });

  // Act
  await new Promise((resolve) => setTimeout(resolve, 100));
  await appendFile(ready, `"schema":"bitaxe-identify-checkpoint-v3","checkpoint":"ready","status":"required"}\n`);
  await privateCheckpoint(root, "rendered");
  await privateCheckpoint(root, "cleared");
  await new Promise((resolve) => setTimeout(resolve, 100));
  settle(ok);
  const result = await supervised;

  // Assert
  assert.equal(result.maybeCheckpointError, undefined);
  assert.deepEqual(signals.map((checkpoint) => checkpoint.checkpoint), ["ready", "rendered", "cleared"]);
});

test("campaign settlement stops polling for later checkpoint files", async () => {
  // Arrange
  const root = await mkdtemp(path.join(os.tmpdir(), "api-command-checkpoint-stop-"));
  await privateCheckpoint(root, "ready");
  const signals: OperatorCheckpointSignal[] = [];

  // Act
  const result = await superviseOperatorCheckpoints(Promise.resolve(ok), root, (checkpoint) => {
    signals.push(checkpoint);
  });
  await privateCheckpoint(root, "rendered");
  await new Promise((resolve) => setTimeout(resolve, 100));

  // Assert
  assert(result.maybeCheckpointError !== undefined);
  assert.deepEqual(signals.map((checkpoint) => checkpoint.checkpoint), ["ready"]);
});

test("a real child publishes ordered checkpoints before it settles", {
  skip: process.platform !== "darwin",
}, async () => {
  // Arrange
  const root = await mkdtemp(path.join(os.tmpdir(), "api-command-checkpoint-"));
  const campaign = path.join(root, "campaign");
  await mkdir(campaign, { mode: 0o700 });
  const child = path.join(root, "checkpoint-campaign.sh");
  await writeFile(child, [
    "#!/bin/sh",
    "set -eu",
    "evidence_root=$1",
    "umask 077",
    "printf '%s\\n' '{\"schema\":\"bitaxe-identify-checkpoint-v3\",\"checkpoint\":\"ready\",\"status\":\"required\"}' > \"$evidence_root/identify-ready.required.json\"",
    "chmod 600 \"$evidence_root/identify-ready.required.json\"",
    "sleep 1",
    "printf '%s\\n' '{\"schema\":\"bitaxe-identify-checkpoint-v3\",\"checkpoint\":\"rendered\",\"status\":\"required\"}' > \"$evidence_root/identify-rendered.required.json\"",
    "chmod 600 \"$evidence_root/identify-rendered.required.json\"",
    "sleep 1",
    "printf '%s\\n' '{\"schema\":\"bitaxe-identify-checkpoint-v3\",\"checkpoint\":\"cleared\",\"status\":\"required\"}' > \"$evidence_root/identify-cleared.required.json\"",
    "chmod 600 \"$evidence_root/identify-cleared.required.json\"",
    "sleep 1",
    ": > \"$evidence_root/child-settled.private\"",
    "chmod 600 \"$evidence_root/child-settled.private\"",
    "exit 1",
    "",
  ].join("\n"), { mode: 0o700 });
  await chmod(child, 0o700);
  const local = createLocalProcessPort({ cwd: root, timeoutMs: 5_000 });
  const childPromise = local.run(internalCommandSpec(child, [campaign], (value) => value));
  const settled = path.join(campaign, "child-settled.private");
  const signals: OperatorCheckpointSignal[] = [];

  // Act
  const supervised = await superviseOperatorCheckpoints(childPromise, campaign, async (checkpoint) => {
    await assert.rejects(stat(settled), { code: "ENOENT" });
    signals.push(checkpoint);
  });
  await new Promise((resolve) => setTimeout(resolve, 100));

  // Assert
  assert.equal(supervised.outcome.exitCode, 1);
  assert.equal(supervised.maybeCheckpointError, undefined);
  assert.deepEqual(signals.map((checkpoint) => checkpoint.checkpoint), ["ready", "rendered", "cleared"]);
  assert.equal(signals.length, 3);
  assert(signals.every((signal) => signal.schema_version === "bitaxe-operator-checkpoint-v3"));
  assert.deepEqual(signals.map((signal) => signal.response_timeout_seconds), [3600, 30, 3600]);
  assert(signals.every((signal) => signal.local_signal_required));
  assert.deepEqual(signals.map((signal) => signal.starts_identify_window), [true, false, false]);
  assert(signals.every((signal) => signal.decline_supported));
  const publicSignal = signals.map(formatOperatorCheckpointSignal).join("");
  assert(!publicSignal.includes(root));
  assert(publicSignal.includes("BITAXE IDENTIFY"));
  assert(publicSignal.includes("operator_ready_timeout_seconds\":3600"));
  assert(publicSignal.includes("identify_duration_seconds\":30"));
});
