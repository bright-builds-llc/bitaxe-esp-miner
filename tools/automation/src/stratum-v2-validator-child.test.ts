import assert from "node:assert/strict";
import { chmod, mkdir, mkdtemp, readFile, realpath, stat } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import {
  runValidatorChild,
  validateValidatorChildReceipt,
} from "./stratum-v2-validator-child.js";

const sourceCommit = "a".repeat(40);
const planSha256 = "b".repeat(64);

async function protectedRoot(): Promise<string> {
  const root = await mkdtemp(path.join(os.tmpdir(), "validator-child-"));
  await chmod(root, 0o700);
  return realpath(root);
}

test("validator child accepts exact marker with workspace binding and protected receipt", async () => {
  // Arrange
  const root = await protectedRoot();
  const receiptPath = path.join(root, "accepted.json");
  const script = "test \"$PWD\" = \"$1\" && printf 'restore_readiness=accepted\\n'";

  // Act
  const receipt = await runValidatorChild({
    workspace: root,
    program: "/bin/sh",
    args: ["-c", script, "validator", root],
    receiptPath,
    sourceCommit,
    planSha256,
  });

  // Assert
  assert(receipt.validation_accepted);
  assert.equal((await stat(receiptPath)).mode & 0o777, 0o600);
  await assert.doesNotReject(validateValidatorChildReceipt(receiptPath, sourceCommit, planSha256));
});

test("validator child rejection stores only bounded digests and excludes output canary", async () => {
  // Arrange
  const root = await protectedRoot();
  const receiptPath = path.join(root, "rejected.json");
  const canary = "validator-secret-canary";

  // Act
  const receipt = await runValidatorChild({
    workspace: root,
    program: "/bin/sh",
    args: ["-c", "printf '%s' \"$1\" >&2; exit 7", "validator", canary],
    receiptPath,
    sourceCommit,
    planSha256,
  });

  // Assert
  assert(!receipt.validation_accepted);
  assert.equal(receipt.exit_code, 7);
  assert(!(await readFile(receiptPath, "utf8")).includes(canary));
});

test("validator child classifies launch failure timeout and output limit", async () => {
  // Arrange
  const root = await protectedRoot();
  await mkdir(path.join(root, "receipts"), { mode: 0o700 });
  const cases = [
    {
      name: "launch",
      program: path.join(root, "missing-validator"),
      args: [] as string[],
      timeoutMillis: 1_000,
      predicate: (value: Awaited<ReturnType<typeof runValidatorChild>>) => value.launch_failed,
    },
    {
      name: "timeout",
      program: "/bin/sh",
      args: ["-c", "sleep 1"],
      timeoutMillis: 20,
      predicate: (value: Awaited<ReturnType<typeof runValidatorChild>>) => value.timed_out,
    },
    {
      name: "limit",
      program: "/usr/bin/yes",
      args: [] as string[],
      timeoutMillis: 1_000,
      predicate: (value: Awaited<ReturnType<typeof runValidatorChild>>) => value.output_limit_exceeded,
    },
  ];

  for (const value of cases) {
    // Act
    const receipt = await runValidatorChild({
      workspace: root,
      program: value.program,
      args: value.args,
      receiptPath: path.join(root, "receipts", `${value.name}.json`),
      sourceCommit,
      planSha256,
      timeoutMillis: value.timeoutMillis,
    });

    // Assert
    assert(value.predicate(receipt), value.name);
    assert(!receipt.validation_accepted, value.name);
  }
});
