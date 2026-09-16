// Software test executable only: emits synthetic detector/flash artifacts, no USB.
import { mkdir, readFile, writeFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
const args = process.argv.slice(2).map((value) => value.startsWith("'") && value.endsWith("'") ? value.slice(1, -1).replaceAll("'\\''", "'") : value);
if (args[0] === "detect-ultra205") {
  process.stdout.write(`port: /dev/cu.synthetic\nusb_profile: serial_jtag_runtime\nphysical_identity_sha256: ${"c".repeat(64)}\n`);
} else {
  if (args[0] !== "flash-monitor") throw new Error("synthetic command");
  const root = args[args.indexOf("--evidence-dir") + 1];
  const { context } = JSON.parse(await readFile(resolve(dirname(root), "context.json")));
  await mkdir(root, { mode: 0o700 });
  const evidence = { command_kind: "flash-monitor", board: "205", flash_status: "completed", capture_mode: "noninteractive", capture_status: "timed_out_after_trusted_output",
    monitor_evidence_status: "trusted", trusted_output: true, commit_ready: true, firmware_commit: context.firmware_commit,
    observed_firmware_commit: context.firmware_commit, reference_commit: context.reference_commit, trust_basis: "fixed_serial", nvs_seed_status: "not_provided",
    redaction_mode: "commit-redacted", capture_timeout_seconds: 30, manifest_path: "[redacted-path]", timestamp: String(Math.floor(Date.now() / 1000)),
    fixed_serial_assessment: { execution_present: true, safe_baseline_confirmed: true, startup_complete: true, startup_failed: false, stable_boot: true, issues: [] } };
  await writeFile(resolve(root, "flash-command-evidence.json"), JSON.stringify(evidence), { mode: 0o600, flag: "wx" });
  await writeFile(resolve(root, "flash-monitor.log"), ["worker_owner_prepare", "usb_installed", "wifi_driver_prepared"].map(stage =>
    `usb_memory_checkpoint stage=${stage} free_bytes=200000 largest_block_bytes=100000 reserve_bytes=98304 redacted=true`).join("\n"), { mode: 0o600, flag: "wx" });
}
