import assert from "node:assert/strict";
import test from "node:test";

import { parseConfigApRecoveryArgs } from "./native-usb-config-ap-recovery.js";

const sealedArgs = [
  "--board", "205",
  "--port", "/dev/cu.usbmodem1101",
  "--package-manifest", "bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json",
  "--wifi-credentials", "wifi-credentials.json",
  "--restore-bundle", "scratch/str005-installed-package-recovery/recovery-006/restore-bundle.private.json",
  "--private-root", "scratch/native-usb-config-ap-recovery/attempt-001",
  "--projection", "docs/parity/evidence/native-usb-config-ap-recovery/recovery-projection-001.json",
  "--plan", "docs/parity/work-plans/20260831T033840Z-NATIVE-USB-CONFIG-AP-RECOVERY-NVS-FIRST/PLAN.md",
  "--redact-evidence",
] as const;

test("stage one accepts only the immutable NVS-first invocation", () => {
  // Arrange / Act
  const parsed = parseConfigApRecoveryArgs("read-nvs", sealedArgs);

  // Assert
  assert.equal(parsed.action, "read-nvs");
  assert.equal(parsed.port, "/dev/cu.usbmodem1101");
  assert.equal(parsed.wifiCredentials, "wifi-credentials.json");
});

test("stage one rejects a changed partition plan or missing redaction", () => {
  // Arrange
  const changedPlan: string[] = [...sealedArgs];
  changedPlan[changedPlan.indexOf("--plan") + 1] = "other-plan.md";
  const withoutRedaction = sealedArgs.filter(value => value !== "--redact-evidence");

  // Act / Assert
  assert.throws(() => parseConfigApRecoveryArgs("read-nvs", changedPlan), /invalid_invocation/u);
  assert.throws(
    () => parseConfigApRecoveryArgs("read-nvs", withoutRedaction),
    /invalid_invocation/u,
  );
});

test("stage two actions remain syntactically sealed for later eligibility checks", () => {
  // Arrange / Act / Assert
  for (const action of ["recover", "resume", "finalize"] as const) {
    assert.equal(parseConfigApRecoveryArgs(action, sealedArgs).action, action);
  }
});
