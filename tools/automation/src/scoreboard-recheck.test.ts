import assert from "node:assert/strict";
import { chmod, readFile, rm, stat, symlink, unlink, writeFile } from "node:fs/promises";
import path from "node:path";
import test from "node:test";

import { scoreboardChild } from "./scoreboard-evidence.test-support.js";
import { ScoreboardEvidenceError } from "./scoreboard-evidence.js";
import {
  recheckScoreboardEvidence,
  type ScoreboardRecheckOptions,
} from "./scoreboard-recheck.js";
import { scoreboardRecheckFixture } from "./scoreboard-recheck.test-support.js";
import { createLocalProcessPort, type ProcessPort } from "./process.js";

async function runFixture(
  root: string,
  base: Awaited<ReturnType<typeof scoreboardRecheckFixture>>["base"],
  options: ScoreboardRecheckOptions,
  identity: Awaited<ReturnType<typeof scoreboardRecheckFixture>>["identity"],
  maybeProcessPort?: ProcessPort,
) {
  const child = await scoreboardChild(base, "http://127.0.0.1:1");
  return recheckScoreboardEvidence(
    root,
    options,
    maybeProcessPort ?? createLocalProcessPort({ cwd: root, timeoutMs: 5_000 }),
    child,
    child,
    identity,
  );
}

test("sealed durable-only restart recheck publishes validated evidence", async () => {
  // Arrange
  const fixture = await scoreboardRecheckFixture("accepted");
  try {
    // Act
    const evidence = await runFixture(
      fixture.root,
      fixture.base,
      fixture.options,
      fixture.identity,
    );

    // Assert
    assert.equal(evidence.scoreboard.entry_count, 2);
    assert.equal(evidence.scoreboard.post_restart_persistence, true);
    assert.equal(evidence.source.source_path_count, 32);
    assert.equal(evidence.hardware_rerun_used, false);
    const projection = path.join(fixture.root, fixture.options.projection);
    assert.equal((await stat(projection)).mode & 0o777, 0o644);
    assert.doesNotMatch(await readFile(projection, "utf8"), /job-a|1234ABCD|usbmodem/u);
  } finally {
    await rm(fixture.root, { recursive: true });
  }
});

test("tampered protected inputs withhold scoreboard projection", async () => {
  for (const candidate of ["seal", "repeat", "failure", "mode", "plan"] as const) {
    // Arrange
    const fixture = await scoreboardRecheckFixture(`tampered-${candidate}`);
    const privateRoot = path.join(fixture.root, fixture.options.privateRoot);
    const wrapperRoot = path.join(fixture.root, fixture.options.wrapperRoot);
    if (candidate === "seal") {
      await writeFile(path.join(privateRoot, "campaign/campaign-result.sha256"), `${"0".repeat(64)}\n`);
    } else if (candidate === "repeat") {
      const value = JSON.parse(await readFile(
        path.join(privateRoot, "scoreboard-after-restart-b.private.json"),
        "utf8",
      )) as Array<Record<string, unknown>>;
      value[0] = { ...value[0], nonce: "1234ABCE" };
      await writeFile(
        path.join(privateRoot, "scoreboard-after-restart-b.private.json"),
        `${JSON.stringify(value)}\n`,
      );
    } else if (candidate === "failure") {
      await writeFile(path.join(wrapperRoot, "capture.stderr"), "unexpected failure\n");
    } else if (candidate === "mode") {
      await chmod(path.join(privateRoot, "scoreboard-before-restart-a.private.json"), 0o644);
    } else {
      await writeFile(path.join(fixture.root, fixture.options.evaluationPlan), "drifted\n");
    }
    try {
      // Act / Assert
      await assert.rejects(runFixture(
        fixture.root,
        fixture.base,
        fixture.options,
        fixture.identity,
      ), ScoreboardEvidenceError);
      await assert.rejects(readFile(path.join(fixture.root, fixture.options.projection), "utf8"), {
        code: "ENOENT",
      });
    } finally {
      await rm(fixture.root, { recursive: true });
    }
  }
});

test("protected symlink and secret-bearing terminal output fail closed", async () => {
  for (const candidate of ["symlink", "secret"] as const) {
    // Arrange
    const fixture = await scoreboardRecheckFixture(candidate);
    const privateRoot = path.join(fixture.root, fixture.options.privateRoot);
    const wrapperRoot = path.join(fixture.root, fixture.options.wrapperRoot);
    if (candidate === "symlink") {
      const target = path.join(privateRoot, "scoreboard-before-restart-a.private.json");
      const replacement = path.join(fixture.root, "replacement.private.json");
      await writeFile(replacement, "[]\n", { mode: 0o600 });
      await unlink(target);
      await symlink(replacement, target);
    } else {
      await writeFile(
        path.join(wrapperRoot, "capture.stderr"),
        "poolPassword=fixture-secret\nbitaxe-automation: capture-scoreboard-evidence blocked: scoreboard restart persistence is invalid\n",
      );
    }
    try {
      // Act
      let maybeError: unknown;
      try {
        await runFixture(fixture.root, fixture.base, fixture.options, fixture.identity);
      } catch (error) {
        maybeError = error;
      }

      // Assert
      assert.ok(maybeError instanceof ScoreboardEvidenceError);
      assert.doesNotMatch(JSON.stringify(maybeError.publicValue), /fixture-secret/u);
    } finally {
      await rm(fixture.root, { recursive: true });
    }
  }
});

test("validator failure removes candidate and withholds projection", async () => {
  // Arrange
  const fixture = await scoreboardRecheckFixture("validator-failure");
  const child = await scoreboardChild(fixture.base, "http://127.0.0.1:1");
  const local = createLocalProcessPort({ cwd: fixture.root, timeoutMs: 5_000 });
  const processPort: ProcessPort = {
    loadEspEnvironment: local.loadEspEnvironment,
    run(spec, maybeLifetime) {
      if (spec.program === "validator") {
        return Promise.resolve({ exitCode: 1, stdout: "", stderr: "", timedOut: false });
      }
      return local.run(spec, maybeLifetime);
    },
  };
  try {
    // Act / Assert
    await assert.rejects(recheckScoreboardEvidence(
      fixture.root,
      fixture.options,
      processPort,
      child,
      "validator",
      fixture.identity,
    ), ScoreboardEvidenceError);
    await assert.rejects(readFile(path.join(fixture.root, fixture.options.projection), "utf8"), {
      code: "ENOENT",
    });
    await assert.rejects(readFile(
      path.join(fixture.root, `${fixture.options.projection}.candidate`),
      "utf8",
    ), { code: "ENOENT" });
  } finally {
    await rm(fixture.root, { recursive: true });
  }
});
