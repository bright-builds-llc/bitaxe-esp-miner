import test from "node:test";
import assert from "node:assert/strict";
import { execFile } from "node:child_process";
import { promisify } from "node:util";
import { chmod, mkdtemp, readFile, realpath, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

const exec = promisify(execFile);
const cli = fileURLToPath(new URL("./main.mjs", import.meta.url));

test("CLI keeps child output and arbitrary environment values out of its public summary", async (t) => {
  // Arrange
  const root = await realpath(await mkdtemp(join(tmpdir(), "host-stall-cli-")));
  await chmod(root, 0o700);
  t.after(() => rm(root, { recursive: true, force: true }));
  const secret = "fixture-private-output";
  // Act
  const { stdout } = await exec(process.execPath, [cli, "run", "--root", join(root, "run"), "--label", "fixture",
    "--timeout-ms", "5000", "--", process.execPath, "-e", "console.log(process.env.PRIVATE_TEST_SECRET)"],
  { env: { ...process.env, PRIVATE_TEST_SECRET: secret }, timeout: 15000 });
  // Assert
  const summary = JSON.parse(stdout);
  assert.equal(summary.outcome, "success");
  assert.equal(summary.cleanupComplete, true);
  assert.equal(stdout.includes(secret), false);
  assert.equal((await readFile(join(root, "run", "invocation.json"), "utf8")).includes(secret), false);
  assert.equal((await readFile(join(root, "run", "stdout.log"), "utf8")).trim(), secret);
});

test("CLI rejects duplicate options before launching a child", async () => {
  await assert.rejects(exec(process.execPath, [cli, "run", "--root", "/unused", "--root", "/other", "--label", "fixture",
    "--", process.execPath, "-e", "process.exit(0)"], { timeout: 5000 }),
  (error) => error.code === 1 && error.stderr.includes("invalid_diagnostic_option"));
});

test("spawn failure keeps the private executable name out of the public event stream", async (t) => {
  // Arrange
  const root = await realpath(await mkdtemp(join(tmpdir(), "host-stall-cli-missing-")));
  await chmod(root, 0o700);
  t.after(() => rm(root, { recursive: true, force: true }));
  const executable = join(root, "fixture-private-executable-name");
  // Act / Assert
  await assert.rejects(exec(process.execPath, [cli, "run", "--root", join(root, "run"), "--label", "fixture", "--", executable],
    { timeout: 15000 }), (error) => {
    assert.equal(error.stdout.includes("fixture-private-executable-name"), false);
    assert.equal(error.stderr.includes("fixture-private-executable-name"), false);
    assert.equal(JSON.parse(error.stdout).outcome, "spawn_error");
    return error.code === 1;
  });
  assert.equal((await readFile(join(root, "run", "summary.json"), "utf8")).includes("fixture-private-executable-name"), true);
});
