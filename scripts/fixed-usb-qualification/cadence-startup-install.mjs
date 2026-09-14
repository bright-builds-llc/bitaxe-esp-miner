import { readdir } from "node:fs/promises";
import { resolve } from "node:path";
import { digest, exactObject, missing, requireCondition, writeNew } from "./contract.mjs";
import { proof } from "./cadence-premining-evidence.mjs";
import { loadStartupRecoveryContext } from "./cadence-startup-context.mjs";

/** Consume the single installation attempt before creating any effect child or issuing a write. */
export async function consumeStartupRecoveryInstall(root, operations = {}) {
  root = resolve(root);
  const context = await loadStartupRecoveryContext(root, { operations });
  await missing(resolve(root, context.installation_directory));
  const claimedAt = (operations.now ?? Date.now)();
  requireCondition(Number.isSafeInteger(claimedAt) && claimedAt > 0, "startup_install_clock");
  await writeNew(resolve(root, "install-consumed.json"), {
    schema: "worker-cadence-recovery-install-consumed-v1",
    context_sha256: digest(JSON.stringify(context)),
    recovery_id: context.recovery_id,
    installation_directory: context.installation_directory,
    manifest_sha256: context.manifest_sha256,
    app_elf_sha256: context.app_elf_sha256,
    claimed_at_unix_ms: claimedAt,
  });
  return { install_consumed: true, maximum_installations: 1, device_effects: false, mining_authorized: false };
}
const sameProcess = (a, b) => a.pid === b.pid && a.pgid === b.pgid && a.startedAt === b.startedAt;
export async function requireStartupInstallation(root, context) {
  const claim = await proof(resolve(root, "install-consumed.json")),
    c = claim.value;
  exactObject(c, [
    "schema",
    "context_sha256",
    "recovery_id",
    "installation_directory",
    "manifest_sha256",
    "app_elf_sha256",
    "claimed_at_unix_ms",
  ]);
  requireCondition(
    c.schema === "worker-cadence-recovery-install-consumed-v1" &&
      c.context_sha256 === digest(JSON.stringify(context)) &&
      c.recovery_id === context.recovery_id &&
      c.installation_directory === "install-001" &&
      c.manifest_sha256 === context.manifest_sha256 &&
      c.app_elf_sha256 === context.app_elf_sha256 &&
      Number.isSafeInteger(c.claimed_at_unix_ms) &&
      c.claimed_at_unix_ms > 0,
    "startup_install_claim",
  );
  requireCondition(
    !(await readdir(root)).some((name) => /^(?:flash|install)-[0-9]/u.test(name) && !/^install-001(?:\.|$)/u.test(name)),
    "startup_extra_installation",
  );
  const flashed = await proof(resolve(root, "install-001/flash-command-evidence.json")),
    f = flashed.value,
    a = f.fixed_serial_assessment;
  requireCondition(
    f.flash_status === "completed" &&
      f.monitor_evidence_status === "trusted" &&
      f.trusted_output === true &&
      f.commit_ready === true &&
      f.firmware_commit === context.firmware_commit &&
      f.observed_firmware_commit === context.firmware_commit &&
      f.reference_commit === context.reference_commit &&
      f.trust_basis === "fixed_serial" &&
      f.nvs_seed_status === "not_provided" &&
      f.redaction_mode === "commit-redacted" &&
      f.manifest_path === "[redacted-path]" &&
      f.capture_timeout_seconds === 30 &&
      a?.execution_present === true &&
      a.safe_baseline_confirmed === true &&
      a.startup_complete === true &&
      a.startup_failed === false &&
      a.stable_boot === true &&
      Array.isArray(a.issues) &&
      a.issues.length === 0,
    "startup_install_not_qualified",
  );
  const observed = await proof(resolve(root, "install-001.observation.json")),
    o = observed.value;
  const commandRoot = await proof(resolve(root, "install-001.host-root.json")),
    armed = await proof(resolve(root, "install-001.observer-armed.json"));
  requireCondition(
    o.schema === "hello-passive-command-observation-v1" &&
      o.rootObserved === true &&
      Number.isSafeInteger(o.observations) &&
      o.observations > 0 &&
      Array.isArray(o.failures) &&
      o.failures.length === 0 &&
      Array.isArray(o.remaining) &&
      o.remaining.length === 0 &&
      o.complete !== false &&
      Array.isArray(o.seen) &&
      o.seen.some((row) => sameProcess(row, commandRoot.value)) &&
      sameProcess(commandRoot.value, armed.value),
    "startup_install_owner",
  );
  requireCondition(
    Number.isSafeInteger(o.started_at_unix_ms) &&
      o.started_at_unix_ms <= c.claimed_at_unix_ms &&
      Number.isSafeInteger(o.finished_at_unix_ms) &&
      o.finished_at_unix_ms >= c.claimed_at_unix_ms &&
      typeof f.timestamp === "string" &&
      /^[0-9]+$/u.test(f.timestamp) &&
      Number(f.timestamp) * 1000 + 999 >= c.claimed_at_unix_ms &&
      Number(f.timestamp) * 1000 <= o.finished_at_unix_ms,
    "startup_install_chronology",
  );
  const cleanup = await proof(resolve(root, "install-owner-cleanup.json")),
    h = cleanup.value;
  exactObject(h, [
    "schema",
    "source",
    "command_exit_code",
    "root_observed",
    "owned_children_absent",
    "serial_holders_absent",
    "observation_sha256",
    "root_sha256",
  ]);
  requireCondition(
    h.schema === "worker-cadence-recovery-install-cleanup-v1" &&
      h.source === "parent-observed" &&
      h.command_exit_code === 0 &&
      h.root_observed === true &&
      h.owned_children_absent === true &&
      h.serial_holders_absent === true &&
      h.observation_sha256 === observed.sha256 &&
      h.root_sha256 === commandRoot.sha256,
    "startup_install_cleanup",
  );
  return { claim_sha256: claim.sha256, flash_sha256: flashed.sha256, observation_sha256: observed.sha256, cleanup_sha256: cleanup.sha256 };
}
