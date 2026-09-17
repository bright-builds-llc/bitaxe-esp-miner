// Explicitly synthetic device/flash input adapter. It never invokes a detector,
// flasher, signer, Serial API or hardware process; all host lifecycle is outside it.
import { mkdir } from "node:fs/promises";
import { resolve } from "node:path";
import { installed, qualification } from "./completed-fixture.mjs";
import { state } from "../str005-noise-serial/test-fixture.mjs";
import { proof, writeNew } from "../str005-noise-serial/files.mjs";

export function deviceState(context, phase = "candidate", closed = false) {
  return { ...state(context, phase, closed), ...(phase === "candidate" ? { qualification: qualification() } : {}) };
}
export function configuredState(context, acquiring = false) {
  const value = deviceState(context, "before"); delete value.preservation;
  return { ...value, status: "configured", connected: false, deviceBaselineConfirmed: false,
    deviceRestorationConfirmed: false, deviceLeaseInactive: false, serialOwnershipReleased: !acquiring };
}
export async function simulatedInstall(f, index) {
  const { root, context, put } = f;
  const { person } = await installed(f, index, true);
  const unix = f.operations.unixNow();
  await mkdir(resolve(root, `install-${index}`), { mode: 0o700 });
  await writeNew(resolve(root, `install-${index}/flash-command-evidence.json`), {
    command_kind: "flash-monitor", board: "205", flash_status: "completed", capture_mode: "noninteractive", capture_status: "timed_out_after_trusted_output",
    monitor_evidence_status: "trusted", trusted_output: true, commit_ready: true, firmware_commit: context.firmware_commit,
    observed_firmware_commit: context.firmware_commit, reference_commit: context.reference_commit, trust_basis: "fixed_serial",
    nvs_seed_status: "not_provided", redaction_mode: "commit-redacted", capture_timeout_seconds: 30, manifest_path: "[redacted-path]",
    timestamp: String(Math.floor(unix / 1000)), fixed_serial_assessment: { execution_present: true, safe_baseline_confirmed: true,
      startup_complete: true, startup_failed: false, stable_boot: true, issues: [] } });
  await put(resolve(root, `install-${index}/flash-monitor.log`), ["worker_owner_prepare", "usb_installed", "wifi_driver_prepared"].map(stage =>
    `usb_memory_checkpoint stage=${stage} free_bytes=200000 largest_block_bytes=100000 reserve_bytes=98304 redacted=true`).join("\n"));
  await writeNew(resolve(root, `install-${index}.observation.json`), { schema: "hello-passive-command-observation-v1", rootObserved: true,
    observations: 10, started_at_unix_ms: unix - 1000, finished_at_unix_ms: unix + 1000,
    seen: [person], remaining: [], failures: [], observer_effects: "process-metadata-only" });
  await writeNew(resolve(root, `install-${index}.exit.json`), { schema: "noise-serial-command-exit-v2", contextSha256: (await proof(root, `install-${index}.claim.json`)).value.contextSha256,
    index, code: 0, ownerSha256: (await proof(root, `install-${index}.host-root.json`)).sha256,
    observationSha256: (await proof(root, `install-${index}.observation.json`)).sha256 });
}
