import assert from "node:assert/strict";
import test from "node:test";

import { parseRestorationFinalizeArgs } from "./stratum-v2-restoration-finalize.js";

const exact = [
  "--board", "205", "--port", "/dev/cu.usbmodem101",
  "--package-manifest", "bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json",
  "--restore-bundle", "scratch/str005-installed-package-recovery/recovery-006/restore-bundle.private.json",
  "--campaign-root", "scratch/str005-stratum-v2/attempt-004",
  "--wifi-credentials", "wifi-credentials.json",
  "--remediation-root", "scratch/str005-exact-restoration/remediation-002",
  "--private-root", "scratch/str005-restoration-finalize/finalize-001",
  "--projection", "docs/parity/evidence/str005-exact-restoration/restoration-projection.json",
  "--plan", "docs/parity/work-plans/20260826T135721Z-STR-005-INACTIVE-RESTORATION/PLAN.md",
  "--redact-evidence",
] as const;

test("restoration finalizer admits only the exact read-only continuation", () => {
  // Arrange
  const changed: string[] = [...exact];
  changed[changed.indexOf("205")] = "601";

  // Act / Assert
  assert.equal(parseRestorationFinalizeArgs(exact).board, "205");
  assert.throws(() => parseRestorationFinalizeArgs(changed));
});
