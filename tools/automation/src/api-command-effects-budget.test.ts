import assert from "node:assert/strict";
import { mkdtemp, readFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import {
  COMMAND_EFFECTS_TRANSACTION_BUDGET,
  deriveCommandEffectsTransactionBudget,
} from "./api-command-effects-budget.js";
import { internalCommandSpec } from "./contracts.generated.js";
import { createLocalProcessPort } from "./process.js";

const nodeProgram = process.env["JS_BINARY__NODE_BINARY"] ?? process.execPath;

function runfileRoot(): string {
  const maybeRunfiles = process.env["RUNFILES_DIR"];
  return maybeRunfiles === undefined
    ? (process.env["BUILD_WORKSPACE_DIRECTORY"] ?? process.cwd())
    : path.join(maybeRunfiles, "_main");
}

test("production parent budget exceeds every bounded child phase", () => {
  // Arrange
  const budget = COMMAND_EFFECTS_TRANSACTION_BUDGET;

  // Act
  const boundedObservation =
    budget.activationMillis
    + budget.operatorReadyMillis
    + budget.observationMillis
    + budget.terminalGraceMillis;

  // Assert
  assert.equal(boundedObservation, 4_980_000);
  assert.equal(budget.childMaximumMillis, 7_450_000);
  assert.equal(budget.parentTimeoutMillis, 7_455_000);
  assert(budget.parentTimeoutMillis > budget.childMaximumMillis);
  assert(budget.parentTimeoutMillis > 810_000);
  assert(budget.fixtureTimeoutMillis > budget.parentTimeoutMillis);
});

test("budget derivation rejects unsafe arithmetic", () => {
  // Arrange
  const components = {
    versionProbeMillis: 1,
    usbCommandCount: Number.MAX_SAFE_INTEGER,
    usbCommandAttemptCount: 2,
    usbCommandMillis: 2,
    usbRetryRecoveryMillis: 1,
    usbRecoveryMillis: 1,
    activationMillis: 1,
    operatorReadyMillis: 1,
    observationMillis: 1,
    terminalGraceMillis: 1,
    finalCleanupMillis: 1,
    processTerminationMillis: 1,
    fixtureStopMarginMillis: 1,
  };

  // Act
  const derive = () => deriveCommandEffectsTransactionBudget(components);

  // Assert
  assert.throws(derive, /budget overflow/u);
});

test("budget components remain bound to child source limits", async () => {
  // Arrange
  const root = runfileRoot();
  const [campaign, environment, usb, recovery, processAdapter, fixture, justfile] =
    await Promise.all([
      readFile(path.join(root, "tools/flash/src/campaign.rs"), "utf8"),
      readFile(path.join(root, "tools/flash/src/environment.rs"), "utf8"),
      readFile(path.join(root, "tools/device-session/src/usb.rs"), "utf8"),
      readFile(path.join(root, "tools/device-session/src/usb/recovery.rs"), "utf8"),
      readFile(path.join(root, "tools/automation/src/process.ts"), "utf8"),
      readFile(path.join(root, "scripts/api-command-effects-stratum-pool.mjs"), "utf8"),
      readFile(path.join(root, "Justfile"), "utf8"),
    ]);

  // Act / Assert
  assert.match(campaign, /MINING_TERMINAL_GRACE_SECONDS: u64 = 180/u);
  assert.match(campaign, /COMMAND_EFFECTS_OPERATOR_READY_SECONDS: u64 = 3_600/u);
  assert.match(campaign, /\.saturating_mul\(2\)/u);
  assert.match(campaign, /\.saturating_add\(COMMAND_EFFECTS_OPERATOR_READY_SECONDS\)/u);
  assert.match(environment, /Duration::from_secs\(10\)/u);
  assert.equal(environment.match(/Duration::from_secs\(360\)/gu)?.length, 1);
  assert.match(usb, /for attempt in 1\.\.=2/u);
  assert.match(recovery, /STANDARD_RECOVERY_TIMEOUT: Duration = Duration::from_secs\(30\)/u);
  assert.match(recovery, /EXTENDED_RECOVERY_TIMEOUT: Duration = Duration::from_secs\(60\)/u);
  assert.match(processAdapter, /setTimeout\(\(\) => child\.kill\("SIGKILL"\), 5_000\)/u);
  assert.match(fixture, /durationSeconds > 7_800/u);
  assert.match(justfile, /signal-api-command-identify \*args:/u);
  assert.match(justfile, /signal-identify \{\{ args \}\}/u);
});

test("derived real-process guard permits child cleanup before exit", async () => {
  // Arrange
  const root = await mkdtemp(path.join(os.tmpdir(), "command-effects-budget-"));
  const cleanup = path.join(root, "cleanup.json");
  const budget = deriveCommandEffectsTransactionBudget({
    versionProbeMillis: 10,
    usbCommandCount: 1,
    usbCommandAttemptCount: 1,
    usbCommandMillis: 10,
    usbRetryRecoveryMillis: 10,
    usbRecoveryMillis: 10,
    activationMillis: 10,
    operatorReadyMillis: 10,
    observationMillis: 10,
    terminalGraceMillis: 10,
    finalCleanupMillis: 10,
    processTerminationMillis: 200,
    fixtureStopMarginMillis: 720,
  });
  const processPort = createLocalProcessPort({ cwd: root, timeoutMs: 2_000 });
  const script = [
    "const fs = require('node:fs');",
    `setTimeout(() => { fs.writeFileSync(${JSON.stringify(cleanup)}, '{\"cleanup_complete\":true}\\n'); }, 60);`,
    "setTimeout(() => process.exit(0), 70);",
  ].join("");

  // Act
  const outcome = await processPort.run(
    internalCommandSpec(nodeProgram, ["-e", script], (value) => value),
    budget.parentTimeoutMillis,
  );

  // Assert
  assert.equal(outcome.timedOut, false);
  assert.equal(outcome.exitCode, 0);
  assert.deepEqual(JSON.parse(await readFile(cleanup, "utf8")), { cleanup_complete: true });
});
