import assert from "node:assert/strict";
import test from "node:test";

import { parseRomExitArgs, projectRomExit } from "./native-usb-rom-exit.js";

const args = [
  "--board", "205",
  "--port", "/dev/cu.usbmodem1101",
  "--package-manifest", "bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json",
  "--restore-bundle", "scratch/str005-installed-package-recovery/recovery-006/restore-bundle.private.json",
  "--private-root", "scratch/native-usb-rom-exit/attempt-001",
  "--projection", "docs/parity/evidence/native-usb-rom-exit/rom-exit-projection-001.json",
  "--plan", "docs/parity/work-plans/20260831T190744Z-NATIVE-USB-ROM-EXIT-DISCRIMINATOR/PLAN.md",
  "--redact-evidence",
] as const;

test("ROM exit parser admits only the immutable command", () => {
  assert.equal(parseRomExitArgs("start", args).action, "start");
  assert.throws(() => parseRomExitArgs("start", args.filter(value => value !== "--redact-evidence")));
});

test("ROM exit projection allowlists only closed fields", () => {
  const machine = {
    schema_version: "bitaxe-native-usb-rom-exit-private-v1",
    source_commit: "1".repeat(40), reference_commit: "2".repeat(40),
    plan_sha256: "3".repeat(64), manifest_sha256: "4".repeat(64),
    restore_bundle_sha256: "5".repeat(64), force_download_bit_set: true,
    reset_adapter: "managed_esptool_hard_reset", transport_profile: "serial_jtag_runtime",
    execution_owner: "application", application_marker_status: "runtime_attestation_exact",
    enumeration_changed: true, nvs_read_repeated: false, device_write_observed: false,
    host_network_effect: false, cleanup_complete: true, terminal_category: "complete",
    redaction_status: "passed", port: "/dev/private", register_value: "secret",
  };
  const projection = projectRomExit(machine, "6".repeat(64));
  assert.equal(projection["schema_version"], "bitaxe-native-usb-rom-exit-projection-v1");
  assert.equal(projection["port"], undefined);
  assert.equal(projection["register_value"], undefined);
  assert.equal(projection["evaluator_sha256"], "6".repeat(64));
});
