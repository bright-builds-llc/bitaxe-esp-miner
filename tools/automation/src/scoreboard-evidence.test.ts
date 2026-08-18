import assert from "node:assert/strict";
import { readFile, rm, stat, writeFile } from "node:fs/promises";
import path from "node:path";
import test from "node:test";

import {
  captureScoreboardEvidence,
  ScoreboardEvidenceError,
} from "./scoreboard-evidence.js";
import {
  expectedPlanSha256,
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
    assert.equal(evidence.source.source_path_count, 29);
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

test("current immutable STAT-003 task and source inventory pass", async () => {
  // Arrange
  const root = process.env["RUNFILES_DIR"] === undefined
    ? workspace
    : path.join(process.env["RUNFILES_DIR"], "_main");

  // Act
  const inventory = await validateScoreboardTaskAndSources(root, expectedPlanSha256);

  // Assert
  assert.equal(inventory.pathCount, 29);
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
