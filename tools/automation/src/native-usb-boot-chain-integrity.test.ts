import assert from "node:assert/strict";
import test from "node:test";
import { parseBootChainArgs, projectBootChain } from "./native-usb-boot-chain-integrity.js";

const args = ["--board", "205", "--port", "/dev/cu.fixture", "--package-manifest", "bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json", "--restore-bundle", "scratch/str005-installed-package-recovery/recovery-006/restore-bundle.private.json", "--private-root", "scratch/native-usb-boot-chain-integrity/attempt-001", "--projection", "docs/parity/evidence/native-usb-boot-chain-integrity/boot-chain-projection-001.json", "--plan", "docs/parity/work-plans/20260902T022334Z-NATIVE-USB-BOOT-CHAIN-INTEGRITY/PLAN.md", "--redact-evidence"] as const;

test("boot-chain parser is immutable", () => { assert.equal(parseBootChainArgs("preflight", args).port, "/dev/cu.fixture"); assert.throws(() => parseBootChainArgs("repeat", args)); });
test("boot-chain projection excludes raw fields", () => {
  const machine: Record<string, unknown> = { schema_version: "bitaxe-native-usb-boot-chain-private-v1", source_commit: "1".repeat(40), reference_commit: "2".repeat(40), plan_sha256: "3".repeat(64), manifest_sha256: "4".repeat(64), restore_bundle_sha256: "5".repeat(64), display_category: "unknown", bootloader_match: true, partition_table_match: true, otadata_match: true, partition_table_valid: true, selected_partition_bundle_match: true, selected_app_digest_match: true, selected_app_header_valid: true, selected_app_identity_match: true, physical_identity_match: true, cleanup_complete: true, ota_selection_category: "ota_selected", selected_partition_category: "ota_0", rom_admission_count: 1, metadata_read_count: 3, selected_app_read_count: 1, application_exit_count: 1, device_write_observed: false, host_network_effect: false, terminal_category: "boot_chain_exact", redaction_status: "passed", raw_flash: "secret", port: "/dev/private" };
  const projection = projectBootChain(machine, "6".repeat(64)); assert.equal(projection["raw_flash"], undefined); assert.equal(projection["port"], undefined);
});
