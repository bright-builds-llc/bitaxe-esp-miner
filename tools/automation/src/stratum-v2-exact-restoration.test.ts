import assert from "node:assert/strict";
import { lstat, mkdtemp, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import {
  createProtectedPrivateRoot,
  parseExactRestorationArgs,
} from "./stratum-v2-exact-restoration.js";
import { validateExactRestorationProjection } from "./stratum-v2-exact-restoration-validator.js";

const common = [
  "--board", "205", "--port", "/dev/cu.usbmodem101",
  "--package-manifest", "bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json",
  "--restore-bundle", "scratch/str005-installed-package-recovery/recovery-006/restore-bundle.private.json",
  "--recovery-projection", "docs/parity/evidence/str005-installed-package-recovery/restore-readiness-projection-006.json",
  "--campaign-root", "scratch/str005-stratum-v2/attempt-004",
  "--wifi-credentials", "wifi-credentials.json",
  "--projection", "docs/parity/evidence/str005-exact-restoration/restoration-projection.json",
  "--plan", "docs/parity/work-plans/20260825T150417Z-STR-005-EXACT-RESTORATION/PLAN.md",
  "--redact-evidence",
] as const;

test("exact restoration parser binds preflight start and settings-only resume roots", () => {
  // Arrange
  const values = [
    ["preflight", "scratch/str005-exact-restoration/preflight-001"],
    ["start", "scratch/str005-exact-restoration/remediation-001"],
    ["resume", "scratch/str005-exact-restoration/remediation-001"],
  ] as const;

  for (const [action, root] of values) {
    // Act / Assert
    assert.equal(parseExactRestorationArgs([
      action, ...common, "--private-root", root,
    ]).action, action);
  }
  assert.throws(() => parseExactRestorationArgs([
    "start", ...common, "--private-root", "scratch/wrong",
  ]));
});

test("exact restoration projection validator accepts only closed success facts", async () => {
  // Arrange
  const root = await mkdtemp(path.join(os.tmpdir(), "exact-restoration-projection-"));
  const candidate = path.join(root, "projection.json");
  const source = "a".repeat(40);
  const value = {
    schema_version: "bitaxe-stratum-v2-exact-restoration-v1",
    status: "accepted", board: 205, remediation_ordinal: 1,
    original_runtime_restored: true, settings_restored: true, theme_restored: true,
    mineonboot_false: true, mining_safe_blocked: true, zero_hashrate: true,
    usb_cleanup_ready: true, redaction_status: "passed", source_commit: source,
  };
  await writeFile(candidate, `${JSON.stringify(value)}\n`);

  // Act / Assert
  await assert.doesNotReject(validateExactRestorationProjection(candidate, source));
  await writeFile(candidate, `${JSON.stringify({ ...value, raw_path: "/private" })}\n`);
  await assert.rejects(validateExactRestorationProjection(candidate, source));
});

test("exact restoration creates an absent protected nested root", async () => {
  // Arrange
  const workspace = await mkdtemp(path.join(os.tmpdir(), "exact-restoration-root-"));
  const privateRoot = path.join(workspace, "scratch", "remediation-001");

  // Act
  await createProtectedPrivateRoot(privateRoot);

  // Assert
  assert.equal((await lstat(path.dirname(privateRoot))).mode & 0o777, 0o700);
  assert.equal((await lstat(privateRoot)).mode & 0o777, 0o700);
});
