import assert from "node:assert/strict";
import test from "node:test";

import {
  nativeUsbRecoveryFailure,
  parseNativeUsbRecoveryArgs,
  validTransitionCandidate,
} from "./native-usb-transition-recovery.js";

const common = [
  "--board", "205",
  "--port", "/dev/cu.usbmodem-test",
  "--package-manifest", "bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json",
  "--wifi-credentials", "wifi-credentials.json",
  "--restore-bundle", "scratch/str005-installed-package-recovery/recovery-006/restore-bundle.private.json",
  "--plan", "docs/parity/work-plans/20260830T142327Z-NATIVE-USB-RECOVERY-TRANSITION/PLAN.md",
  "--projection", "docs/parity/evidence/native-usb-transition/transition-projection-001.json",
  "--redact-evidence",
] as const;

test("primary recovery parser accepts only the immutable ordinal-two contract", () => {
  // Arrange / Act
  const args = parseNativeUsbRecoveryArgs("preflight", [
    ...common,
    "--private-root", "scratch/native-usb-transition/recovery-002",
    "--recovery-ordinal", "2",
  ]);

  // Assert
  assert.equal(args.recoveryOrdinal, 2);
  assert.equal(args.privateRoot, "scratch/native-usb-transition/recovery-002");
  assert.equal(args.redactEvidence, true);
});

test("preflight rejects the contingency ordinal", () => {
  // Arrange / Act / Assert
  assert.throws(() => parseNativeUsbRecoveryArgs("preflight", [
    ...common,
    "--private-root", "scratch/native-usb-transition/recovery-003",
    "--recovery-ordinal", "3",
  ]));
});

test("failure projection exposes only a closed category and checkpoint", () => {
  // Arrange / Act
  const failure = nativeUsbRecoveryFailure(new Error("secret endpoint and port"));

  // Assert
  assert.deepEqual(failure, {
    schema_version: "bitaxe-native-usb-recovery-failure-v1",
    status: "failed",
    category: "evidence_invalid",
    checkpoint: "unexpected_failure",
  });
});

test("transition candidate validator rejects impossible stage ordering", () => {
  // Arrange
  const candidate = {
    schema_version: "bitaxe-native-usb-transition-projection-v1",
    source_commit: "1".repeat(40),
    reference_commit: "2".repeat(40),
    plan_sha256: "cbc11639a51e67d24a04b33c05dd3dd2e570914be79f3a3d80b7326894e74eca",
    evaluator_sha256: "3".repeat(64),
    manifest_sha256: "4".repeat(64),
    app_elf_sha256: "5".repeat(64),
    ready_received: false,
    committed_received: true,
    bus_reset_observed: false,
    absent_count: 1,
    same_worker_count: 1,
    same_serial_jtag_count: 0,
    same_unknown_count: 0,
    physical_mismatch_count: 0,
    rom_admitted: false,
    application_reappeared: false,
    device_write_observed: false,
    restoration_complete: false,
    cleanup_complete: true,
    redaction_status: "passed",
    terminal_category: "same_worker_after_commit",
  };

  // Act / Assert
  assert.equal(validTransitionCandidate(candidate), false);
});
