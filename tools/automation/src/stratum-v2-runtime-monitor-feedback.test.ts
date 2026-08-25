import assert from "node:assert/strict";
import { chmod, mkdtemp, readFile, stat, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import { runCampaignProcess } from "./stratum-v2-campaign.js";
import {
  monitorRuntimeOrigin,
  runtimeMonitorCaptureSeconds,
  runtimeMonitorProcessTimeoutMillis,
} from "./stratum-v2-runtime-admission.js";
import { parseRuntimeMonitorDiagnosticArgs } from "./stratum-v2-runtime-monitor-diagnostic.js";
import {
  runRuntimeMonitorChild,
  validateRuntimeMonitorReceipt,
} from "./stratum-v2-runtime-monitor-child.js";

const sourceCommit = "a".repeat(40);
const planSha256 = "b".repeat(64);

test("runtime monitor outer lifetime covers probe admission capture and cleanup", () => {
  // Arrange / Act / Assert
  assert.equal(runtimeMonitorCaptureSeconds, 60);
  assert.equal(runtimeMonitorProcessTimeoutMillis, 210_000);
});

test("runtime monitor diagnostic parser admits only the rolling contract command", () => {
  // Arrange
  const exact = [
    "--board", "205",
    "--port", "/dev/cu.usbmodem101",
    "--private-root", "scratch/str005-runtime-monitor-diagnostic/diagnostic-002",
    "--redact-evidence",
  ];

  // Act / Assert
  assert.equal(parseRuntimeMonitorDiagnosticArgs(exact).board, "205");
  assert.throws(() => parseRuntimeMonitorDiagnosticArgs([...exact, "--unknown", "value"]));
  assert.throws(() => parseRuntimeMonitorDiagnosticArgs(
    exact.filter(value => value !== "--redact-evidence"),
  ));
});

test("runtime monitor real child receives the qualified sixty-second admission bound", async () => {
  // Arrange
  const workspace = await mkdtemp(path.join(os.tmpdir(), "runtime-monitor-feedback-"));
  const fixture = path.join(workspace, "monitor-fixture.sh");
  await writeFile(fixture, `#!/bin/sh
set -eu
capture=0
while [ "$#" -gt 0 ]; do
  if [ "$1" = "--capture-timeout-seconds" ]; then
    shift
    capture="$1"
  fi
  shift
done
if [ "$capture" -lt 60 ]; then
  exit 78
fi
printf 'monitor_command: protected\\n'
printf 'runtime_origin=http://device.invalid\\n'
printf 'usb_session: ready\\n'
`, { mode: 0o700 });
  await chmod(fixture, 0o700);
  const receiptPath = path.join(workspace, "monitor-receipt.json");
  const fail = (category: string, message: string, checkpoint: string): never => {
    throw new Error(`${category}:${checkpoint}:${message}`);
  };

  // Act
  const origin = await monitorRuntimeOrigin(
    workspace,
    fixture,
    "/dev/fixture",
    runCampaignProcess,
    fail,
    { receiptPath, sourceCommit, planSha256 },
  );

  // Assert
  assert.equal(origin.origin, "http://device.invalid");
  assert.equal((await stat(receiptPath)).mode & 0o777, 0o600);
  const receipt = await validateRuntimeMonitorReceipt(receiptPath, sourceCommit, planSha256);
  assert.equal(receipt.terminal_category, "ready");
});

test("runtime monitor failure retains a closed receipt without output canary", async () => {
  // Arrange
  const workspace = await mkdtemp(path.join(os.tmpdir(), "runtime-monitor-failure-"));
  const fixture = path.join(workspace, "monitor-fixture.sh");
  const canary = "runtime-monitor-secret-canary";
  await writeFile(fixture, `#!/bin/sh
set -eu
printf '%s' '${canary}' >&2
exit 78
`, { mode: 0o700 });
  await chmod(fixture, 0o700);
  const receiptPath = path.join(workspace, "monitor-receipt.json");
  const fail = (category: string, message: string, checkpoint: string): never => {
    throw new Error(`${category}:${checkpoint}:${message}`);
  };

  // Act / Assert
  await assert.rejects(monitorRuntimeOrigin(
    workspace,
    fixture,
    "/dev/fixture",
    runCampaignProcess,
    fail,
    { receiptPath, sourceCommit, planSha256 },
  ), /hardware_blocked:runtime_monitor_process/u);
  const receipt = await validateRuntimeMonitorReceipt(receiptPath, sourceCommit, planSha256);
  assert.equal(receipt.terminal_category, "monitor_failed");
  assert(!(await readFile(receiptPath, "utf8")).includes(canary));
});

test("runtime monitor child classifies launch failure timeout and output limit", async () => {
  // Arrange
  const workspace = await mkdtemp(path.join(os.tmpdir(), "runtime-monitor-cases-"));
  const cases = [
    {
      name: "launch",
      program: path.join(workspace, "missing-monitor"),
      args: [] as string[],
      timeoutMillis: 1_000,
      category: "launch_failed",
    },
    {
      name: "timeout",
      program: "/bin/sh",
      args: ["-c", "sleep 1"],
      timeoutMillis: 20,
      category: "timeout",
    },
    {
      name: "limit",
      program: "/usr/bin/yes",
      args: [] as string[],
      timeoutMillis: 1_000,
      category: "output_limit",
    },
  ] as const;

  for (const value of cases) {
    // Act
    const receiptPath = path.join(workspace, `${value.name}.json`);
    const outcome = await runRuntimeMonitorChild({
      workspace,
      program: value.program,
      args: value.args,
      receiptPath,
      sourceCommit,
      planSha256,
      timeoutMillis: value.timeoutMillis,
    });

    // Assert
    assert.equal(outcome.receipt.terminal_category, value.category);
    await assert.doesNotReject(validateRuntimeMonitorReceipt(
      receiptPath,
      sourceCommit,
      planSha256,
    ));
  }
});
