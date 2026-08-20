import assert from "node:assert/strict";
import { readFile, rm, stat, writeFile } from "node:fs/promises";
import path from "node:path";
import test from "node:test";

import {
  captureScoreboardEvidence,
  ScoreboardEvidenceError,
} from "./scoreboard-evidence.js";
import {
  bootMiningDisabled,
  durableDifficulty,
  expectedPlanSha256,
  requiredBoolean,
  scoreboardRestartPersists,
  scoreboardView,
  validateScoreboardTaskAndSources,
} from "./scoreboard-evidence-contract.js";
import {
  scoreboardChild,
  scoreboardFixture,
  startScoreboardServer,
} from "./scoreboard-evidence.test-support.js";
import { createLocalProcessPort } from "./process.js";

const workspace = process.env["BUILD_WORKSPACE_DIRECTORY"] ?? process.cwd();

async function captureError(promise: Promise<unknown>): Promise<ScoreboardEvidenceError> {
  try {
    await promise;
    assert.fail("expected scoreboard evidence failure");
  } catch (error) {
    assert.ok(error instanceof ScoreboardEvidenceError);
    return error;
  }
}

test("real child campaign API and restart publish only closed scoreboard evidence", async () => {
  // Arrange
  const fixture = await scoreboardFixture("accepted");
  const server = await startScoreboardServer();
  const child = await scoreboardChild(fixture, server.origin);
  try {
    // Act
    const evidence = await captureScoreboardEvidence(
      fixture.root,
      fixture.options,
      createLocalProcessPort({ cwd: fixture.root, timeoutMs: 5_000 }),
      child,
      child,
      child,
      fixture.planSha256,
      async () => {},
    );

    // Assert
    assert.equal(evidence.scoreboard.entry_count, 2);
    assert.equal(evidence.scoreboard.post_restart_persistence, true);
    assert.equal(evidence.source.source_path_count, 32);
    const projection = path.join(fixture.root, fixture.options.projection);
    assert.equal((await stat(projection)).mode & 0o777, 0o644);
    assert.doesNotMatch(
      await readFile(projection, "utf8"),
      /job-a|1234ABCD|device_url|pool|credential|private-port/u,
    );
  } finally {
    await server.close();
    await rm(fixture.root, { recursive: true });
  }
});

test("changed scoreboard after restart withholds projection", async () => {
  // Arrange
  const fixture = await scoreboardFixture("restart-drift");
  const server = await startScoreboardServer({ changeAfterRestart: true });
  const child = await scoreboardChild(fixture, server.origin);
  try {
    // Act
    const error = await captureError(captureScoreboardEvidence(
      fixture.root,
      fixture.options,
      createLocalProcessPort({ cwd: fixture.root, timeoutMs: 5_000 }),
      child,
      child,
      child,
      fixture.planSha256,
      async () => {},
    ));

    // Assert
    assert.equal(error.category, "hardware_blocked");
    await assert.rejects(readFile(path.join(fixture.root, fixture.options.projection), "utf8"), {
      code: "ENOENT",
    });
  } finally {
    await server.close();
    await rm(fixture.root, { recursive: true });
  }
});

test("post-restart repeat drift withholds projection", async () => {
  // Arrange
  const fixture = await scoreboardFixture("post-restart-repeat-drift");
  const server = await startScoreboardServer({ changePostRestartRepeat: true });
  const child = await scoreboardChild(fixture, server.origin);
  try {
    // Act
    const error = await captureError(captureScoreboardEvidence(
      fixture.root,
      fixture.options,
      createLocalProcessPort({ cwd: fixture.root, timeoutMs: 5_000 }),
      child,
      child,
      child,
      fixture.planSha256,
      async () => {},
    ));

    // Assert
    assert.equal(error.category, "hardware_blocked");
    await assert.rejects(readFile(path.join(fixture.root, fixture.options.projection), "utf8"), {
      code: "ENOENT",
    });
  } finally {
    await server.close();
    await rm(fixture.root, { recursive: true });
  }
});

test("paused post-restart state with disabled boot mining publishes", async () => {
  // Arrange
  const fixture = await scoreboardFixture("paused-restart");
  const server = await startScoreboardServer({ postRestartMiningActivity: "paused" });
  const child = await scoreboardChild(fixture, server.origin);
  try {
    // Act
    const evidence = await captureScoreboardEvidence(
      fixture.root,
      fixture.options,
      createLocalProcessPort({ cwd: fixture.root, timeoutMs: 5_000 }),
      child,
      child,
      child,
      fixture.planSha256,
      async () => {},
    );

    // Assert
    assert.equal(evidence.scoreboard.boot_mining_disabled, true);
    assert.equal(evidence.scoreboard.post_restart_persistence, true);
    assert.equal((await stat(path.join(fixture.root, fixture.options.projection))).mode & 0o777, 0o644);
  } finally {
    await server.close();
    await rm(fixture.root, { recursive: true });
  }
});

test("natural analyzer closure publishes without a worker close request", async () => {
  // Arrange
  const fixture = await scoreboardFixture("natural-serial-close");
  const server = await startScoreboardServer();
  const child = await scoreboardChild(fixture, server.origin, { terminalCloseRequested: false });
  try {
    // Act
    const evidence = await captureScoreboardEvidence(
      fixture.root,
      fixture.options,
      createLocalProcessPort({ cwd: fixture.root, timeoutMs: 5_000 }),
      child,
      child,
      child,
      fixture.planSha256,
      async () => {},
    );

    // Assert
    assert.equal(evidence.campaign_status, "accepted");
    assert.equal((await stat(path.join(fixture.root, fixture.options.projection))).mode & 0o777, 0o644);
  } finally {
    await server.close();
    await rm(fixture.root, { recursive: true });
  }
});

test("invalid closure-request diagnostic shapes withhold scoreboard evidence", async () => {
  for (const [name, options] of [
    ["missing", { omitTerminalCloseRequested: true }],
    ["non-boolean", { terminalCloseRequested: "false" }],
  ] as const) {
    // Arrange
    const fixture = await scoreboardFixture(`terminal-close-${name}`);
    const server = await startScoreboardServer();
    const child = await scoreboardChild(fixture, server.origin, options);

    try {
      // Act
      const error = await captureError(captureScoreboardEvidence(
        fixture.root,
        fixture.options,
        createLocalProcessPort({ cwd: fixture.root, timeoutMs: 5_000 }),
        child,
        child,
        child,
        fixture.planSha256,
        async () => {},
      ));

      // Assert
      assert.equal(error.category, "evidence_invalid");
      await assert.rejects(readFile(path.join(fixture.root, fixture.options.projection), "utf8"), {
        code: "ENOENT",
      });
    } finally {
      await server.close();
      await rm(fixture.root, { recursive: true });
    }
  }
});

test("accepted transport evidence without final consumed handoff withholds projection", async () => {
  // Arrange
  const fixture = await scoreboardFixture("missing-final-consumed");
  const server = await startScoreboardServer();
  const child = await scoreboardChild(fixture, server.origin, { finalTerminalConsumed: false });
  try {
    // Act
    const error = await captureError(captureScoreboardEvidence(
      fixture.root,
      fixture.options,
      createLocalProcessPort({ cwd: fixture.root, timeoutMs: 5_000 }),
      child,
      child,
      child,
      fixture.planSha256,
      async () => {},
    ));

    // Assert
    assert.equal(error.category, "evidence_invalid");
    await assert.rejects(readFile(path.join(fixture.root, fixture.options.projection), "utf8"), {
      code: "ENOENT",
    });
  } finally {
    await server.close();
    await rm(fixture.root, { recursive: true });
  }
});

test("consumed attempt-004 root is rejected before hardware orchestration", async () => {
  // Arrange
  const fixture = await scoreboardFixture("consumed-attempt");
  const child = await scoreboardChild(fixture, "http://127.0.0.1:1");
  const options = {
    ...fixture.options,
    privateRoot: "scratch/stat003-scoreboard/attempt-004",
  };

  try {
    // Act
    const error = await captureError(captureScoreboardEvidence(
      fixture.root,
      options,
      createLocalProcessPort({ cwd: fixture.root, timeoutMs: 5_000 }),
      child,
      child,
      child,
      fixture.planSha256,
      async () => {},
    ));

    // Assert
    assert.equal(error.category, "evidence_invalid");
    await assert.rejects(readFile(path.join(fixture.root, fixture.options.projection), "utf8"), {
      code: "ENOENT",
    });
  } finally {
    await rm(fixture.root, { recursive: true });
  }
});

test("scoreboard parser rejects ascending malformed and oversized input", () => {
  // Arrange
  const entry = {
    difficulty: 1,
    job_id: "job",
    extranonce2: "01",
    ntime: 1,
    nonce: "00000001",
    version_bits: "00000000",
  };

  // Act / Assert
  assert.throws(() => scoreboardView([{ ...entry }, { ...entry, difficulty: 2 }], "ascending"));
  assert.throws(() => scoreboardView([{ ...entry, nonce: "private" }], "malformed"));
  assert.throws(() => scoreboardView(Array.from({ length: 21 }, () => entry), "oversized"));
});

test("durable difficulty matches pinned one-decimal ties-to-even semantics", () => {
  // Arrange
  const cases = [
    { runtime: 1.25, durable: 1.2 },
    { runtime: 1.35, durable: 1.4 },
    { runtime: 42.54, durable: 42.5 },
    { runtime: 42.55, durable: 42.5 },
    { runtime: 42.56, durable: 42.6 },
    { runtime: Number.MAX_VALUE, durable: Number.MAX_VALUE },
  ] as const;

  for (const candidate of cases) {
    // Act
    const result = durableDifficulty(candidate.runtime);

    // Assert
    assert.equal(result, candidate.durable);
  }
});

test("restart persistence changes only durable difficulty", () => {
  // Arrange
  const beforeEntries = [
    { difficulty: 42.54, job_id: "job-a", extranonce2: "0001", ntime: 1, nonce: "1234ABCD", version_bits: "20000000" },
    { difficulty: 10.06, job_id: "job-b", extranonce2: "0002", ntime: 2, nonce: "00000001", version_bits: "00000000" },
  ];
  const afterEntries = [
    { ...beforeEntries[0], difficulty: 42.5 },
    { ...beforeEntries[1], difficulty: 10.1 },
  ];
  const before = scoreboardView(beforeEntries, "before restart");

  // Act / Assert
  assert.equal(
    scoreboardRestartPersists(before, scoreboardView(afterEntries, "after restart")),
    true,
  );
  assert.equal(scoreboardRestartPersists(before, scoreboardView([
    { ...afterEntries[0], difficulty: 42.6 },
    afterEntries[1],
  ], "wrong difficulty")), false);
  assert.equal(scoreboardRestartPersists(before, scoreboardView([
    { ...afterEntries[0], nonce: "1234ABCE" },
    afterEntries[1],
  ], "changed nonce")), false);
  assert.throws(() => scoreboardView([...afterEntries].reverse(), "reordered"));
});

test("closed boolean diagnostics permit either value and reject invalid shapes", () => {
  // Arrange
  const context = "campaign network";

  // Act / Assert
  assert.equal(requiredBoolean({ terminal_close_requested: true }, "terminal_close_requested", context), true);
  assert.equal(requiredBoolean({ terminal_close_requested: false }, "terminal_close_requested", context), false);
  assert.throws(() => requiredBoolean({}, "terminal_close_requested", context));
  assert.throws(() => requiredBoolean(
    { terminal_close_requested: "false" },
    "terminal_close_requested",
    context,
  ));
});

test("boot mining disabled accepts only closed non-active states with false intent", () => {
  // Arrange
  const cases = [
    { intent: false, activity: "paused", expected: true },
    { intent: false, activity: "safe_blocked", expected: true },
    { intent: false, activity: "active", expected: false },
    { intent: false, activity: "unknown", expected: false },
    { intent: true, activity: "paused", expected: false },
    { intent: true, activity: "safe_blocked", expected: false },
  ] as const;

  for (const candidate of cases) {
    // Act
    const result = bootMiningDisabled(candidate.intent, candidate.activity);

    // Assert
    assert.equal(result, candidate.expected);
  }
});

test("current or archived immutable STAT-003 task and source inventory pass", async () => {
  // Arrange
  const root = process.env["RUNFILES_DIR"] === undefined
    ? workspace
    : path.join(process.env["RUNFILES_DIR"], "_main");

  // Act
  const inventory = await validateScoreboardTaskAndSources(root, expectedPlanSha256);

  // Assert
  assert.equal(inventory.pathCount, 32);
  assert.match(inventory.digest, /^[0-9a-f]{64}$/u);
});

test("source drift fails before hardware orchestration", async () => {
  // Arrange
  const fixture = await scoreboardFixture("source-drift");
  const source = path.join(fixture.root, "firmware/bitaxe/src/scoreboard_adapter.rs");
  await writeFile(source, "drifted\n");

  try {
    // Act / Assert
    await assert.rejects(validateScoreboardTaskAndSources(fixture.root, fixture.planSha256));
  } finally {
    await rm(fixture.root, { recursive: true });
  }
});
