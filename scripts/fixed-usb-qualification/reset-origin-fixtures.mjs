import { chmod, copyFile, readdir, rm, writeFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { installedRecovery, recoveryState, recordRecoveryState } from "./cadence-startup-fixtures.mjs";
import { inventory } from "./cadence-premining-evidence.mjs";
import { digest, fileDigest, readJson, writeNew } from "./contract.mjs";
import { resetOriginPreflight } from "./reset-origin-context.mjs";
import { saveNoMiningAccounting } from "./no-mining-accounting.mjs";

const HERE = dirname(fileURLToPath(import.meta.url));
export async function resetOriginFixture(t, { preflight = true } = {}) {
  const f = await installedRecovery(t), sourceRoot = f.root;
  const flashPath = resolve(sourceRoot, "install-001/flash-command-evidence.json");
  const flash = await readJson(flashPath);
  Object.assign(flash, { monitor_evidence_status: "untrusted", trusted_output: false });
  Object.assign(flash.fixed_serial_assessment, { stable_boot: false, issues: ["reboot_observed", "insufficient_advancing_samples"] });
  await writeFile(flashPath, JSON.stringify(flash));
  await rm(resolve(sourceRoot, "install-owner-cleanup.json"));
  await writeNew(resolve(sourceRoot, "failed-inventory.json"), {
    schema: "cpu0-cadence-startup-recovery-failed-inventory-v1", outcome: "stop_impossible_contract",
    qualification_pass: false, device_recovery_claimed: false, new_hardware_authorized: false,
    context_sha256: digest(JSON.stringify(f.context)), artifact_snapshot_sha256: await fileDigest(resolve(sourceRoot, "artifact-snapshot.json")),
    inventory: await inventory(sourceRoot),
  });
  for (const name of (await readdir(HERE)).filter(name => name.endsWith(".mjs") && !name.endsWith(".test.mjs"))) {
    const copied = resolve(f.options.firmwareRoot, "scripts/fixed-usb-qualification", name);
    await copyFile(resolve(HERE, name), copied);
    await chmod(copied, 0o600);
  }
  const operations = { ...f.operations, expectedResetOriginSourceSeal: await fileDigest(resolve(sourceRoot, "failed-inventory.json")),
    git: (_root, args) => {
      if (args[0] === "merge-base") return f.context.firmware_commit;
      if (args[0] === "diff") return "";
      if (args[0] === "ls-tree") return "100644 blob fixture";
      throw Error("unexpected_fixture_git");
    },
  };
  const root = resolve(f.base, "attempts/observation");
  const options = { ...f.options, privateRoot: root, predecessorRoot: sourceRoot,
    manifest: resolve(sourceRoot, "qualified-artifacts/firmware/bitaxe-ultra205-package.json"), qualificationSourceCommit: "e".repeat(40) };
  if (preflight) await resetOriginPreflight(options, operations);
  const context = preflight ? (await readJson(resolve(root, "context.json"))).context : undefined;
  return { ...f, sourceRoot, root, options, operations, context,
    ledger: { schema: "worker-qualification-ledger-v1", next_ordinal: 17, last_completed_ordinal: 16, total_charged_ms: 1380000, pending: false } };
}
export function resetDiagnostics(context, uptime = 1000) {
  return [
    { category: "boot", authoritative: false, boot_ordinal: 2, reset_reason: "panic", uptime_ms: uptime },
    { category: "startup", authoritative: false, stage: "runtime_ready", state: "complete", first_failure: "none", uptime_ms: uptime },
    { category: "runtime_identity", authoritative: false, firmware_commit: context.firmware_commit, app_elf_sha256: context.app_elf_sha256 },
    { category: "storage_http_status", authoritative: false, spiffs_available: "true", http_ready: "true" },
  ];
}
export async function recordResetAccounting(f, stage) {
  const state = recoveryState(f.context.no_mining_context);
  await recordRecoveryState(f.root, f.context.no_mining_context, state);
  await saveNoMiningAccounting(f.root, f.context.no_mining_context, { stage, ledger: f.ledger, original_budget: f.original, state });
  return state;
}
export async function completeResetOriginFixture(t) {
  const f = await resetOriginFixture(t), hash = digest(JSON.stringify(f.context));
  await writeNew(resolve(f.root, "reset-origin-server-claim.json"), { schema: "fixed-usb-reset-origin-server-claim-v1", context_sha256: hash });
  await recordResetAccounting(f, "before");
  const prime = resetDiagnostics(f.context, 1000);
  await writeNew(resolve(f.root, "diagnostic-export-0000.json"), { schema: "fixed-usb-reset-origin-batch-v1", context_sha256: hash, sequence: 0, hostMonotonicMs: 100, observations: prime });
  await writeNew(resolve(f.root, "reset-origin-start.json"), { schema: "fixed-usb-reset-origin-start-v1", context_sha256: hash, hostMonotonicMs: 200, observed_sequence: 1, primeObservations: prime });
  for (let n = 1; n <= 260; n++) await writeNew(resolve(f.root, `diagnostic-export-${String(n).padStart(4, "0")}.json`), {
    schema: "fixed-usb-reset-origin-batch-v1", context_sha256: hash, sequence: n, hostMonotonicMs: 200 + n * 500, observations: resetDiagnostics(f.context, 1000 + n * 500),
  });
  await recordRecoveryState(f.root, f.context.no_mining_context, recoveryState(f.context.no_mining_context));
  await writeNew(resolve(f.root, "reset-origin-end.json"), { schema: "fixed-usb-reset-origin-end-v1", context_sha256: hash, hostMonotonicMs: 130200, observed_sequence: 2 });
  await recordResetAccounting(f, "after");
  await recordRecoveryState(f.root, f.context.no_mining_context, recoveryState(f.context.no_mining_context, true));
  const cleanupPath = resolve(f.root, "cleanup.json");
  await writeNew(cleanupPath, { schema: "worker-reset-origin-cleanup-v1", source: "parent-observed", browser_closed: true, supervisor_exited: true,
    supervisor_exit_code: 0, listener_absent: true, owned_children_absent: true, serial_holders_absent: true });
  return { ...f, cleanupPath };
}
