import { readFile, readdir } from "node:fs/promises";
import { resolve } from "node:path";
import { isDeepStrictEqual as equal } from "node:util";
import { canonicalDirectory, digest, exactObject, fileDigest, missing, protectedPath, requireCondition as check } from "./contract.mjs";
import { inventory, proof, baseline } from "./cadence-premining-evidence.mjs";
import { verifyArtifactSnapshot } from "./snapshot.mjs";
import { loadRestartContext, restartInnerContext } from "./reset-origin-restart-context.mjs";
import { readRestartStates, readRestartAccounting } from "./reset-origin-restart-state.mjs";
import { requirePreinstallFailure } from "./reset-origin-restart-preinstall.mjs";
import { parseStatisticsStartupLine, requireActiveStatistics } from "./reset-origin-restart-statistics.mjs";
import { RESTART_INSTALL_FAILURE_SHA256 } from "./reset-origin-restart-install-failure.mjs";

export const RESTART_NETWORK_FAILURE_SHA256 = "29c8f6bb14b1962676bdd8af4a5c9b406ea81ab0f68c61882c51c2680eb0d7ad";
export const RESTART_NETWORK_FAILURE_PRODUCER = "4946a65aaa619c0dd291ca09932cadaa87ed2c7f1eccb7507cf8ef2a34e5efab";
const same = (a, b) =>
  a &&
  b &&
  Number.isSafeInteger(a.pid) &&
  a.pid > 0 &&
  Number.isSafeInteger(a.pgid) &&
  a.pgid > 0 &&
  typeof a.startedAt === "string" &&
  a.startedAt.length > 0 &&
  a.pid === b.pid &&
  a.pgid === b.pgid &&
  a.startedAt === b.startedAt;

/** Closed startup facts only; neither raw marker identity nor active statistics proves authenticated health. */
export function inspectNetworkFailureCapture(bytes, context) {
  check(Buffer.isBuffer(bytes) && bytes.length > 0 && bytes.length <= 1048576, "restart_network_capture_bound");
  const lines = bytes.toString("utf8").split("\n");
  lines.pop();
  const boots = [],
    startup = [],
    statistics = [];
  let identities = 0,
    networkFailures = 0;
  for (const raw of lines) {
    const line = raw.endsWith("\r") ? raw.slice(0, -1) : raw;
    check(
      !/^(rust_panic_receipt|allocation_failure|allocation_failure_context)/u.test(line) &&
        !/Guru Meditation Error|abort\(\) was called|stack overflow/iu.test(line),
      "restart_network_other_failure",
    );
    if (line.startsWith("statistics_startup")) statistics.push(parseStatisticsStartupLine(line));
    if (line.startsWith("usb_reboot_discriminator")) {
      const m =
        /^usb_reboot_discriminator schema=v1 boot_ordinal=([0-9]+) reset_reason=(power_on|software_cpu|panic|other) uptime_ms=([0-9]+) redacted=true$/u.exec(
          line,
        );
      check(m, "restart_network_boot");
      const b = { ordinal: Number(m[1]), reason: m[2], uptime: Number(m[3]) },
        prior = boots.at(-1);
      check(
        Number.isSafeInteger(b.ordinal) &&
          b.ordinal > 0 &&
          Number.isSafeInteger(b.uptime) &&
          (!prior || (b.ordinal === prior.ordinal && b.reason === prior.reason && b.uptime >= prior.uptime)),
        "restart_network_boot_transition",
      );
      boots.push(b);
    }
    if (line.startsWith("usb_startup")) {
      const m =
        /^usb_startup schema=v1 stage=([a-z_]+) state=(entered|complete|failed) first_failure=([a-z_]+) uptime_ms=([0-9]+) redacted=true$/u.exec(
          line,
        );
      check(m, "restart_network_startup");
      check(
        (m[1] === "network" && m[2] === "entered" && m[3] === "none") ||
          (m[1] === "runtime_ready" && m[2] === "complete" && m[3] === "network"),
        "restart_network_other_failure",
      );
      const value = { first_failure: m[3], uptime: Number(m[4]) };
      check(
        Number.isSafeInteger(value.uptime) && (!startup.length || value.uptime >= startup.at(-1).uptime),
        "restart_network_startup_clock",
      );
      startup.push(value);
    }
    if (line.startsWith("usb_runtime_identity")) {
      const m = /^usb_runtime_identity schema=v1 firmware_commit=([0-9a-f]{40}) app_elf_sha256=([0-9a-f]{64}) redacted=true$/u.exec(line);
      check(m && m[1] === context.firmware_commit && m[2] === context.app_elf_sha256, "restart_network_identity");
      identities++;
    }
    if (line.startsWith("wifi_startup_failure")) {
      check(line === "wifi_startup_failure schema=v1 phase=reconnect_spawn error=no_memory redacted=true", "restart_network_marker");
      networkFailures++;
    }
  }
  const active = requireActiveStatistics(statistics),
    failed = startup.filter((v) => v.first_failure === "network");
  check(
    boots.length > 1 &&
      failed.length > 1 &&
      identities > 0 &&
      networkFailures > 0 &&
      boots.at(-1).uptime > boots[0].uptime &&
      failed.at(-1).uptime > failed[0].uptime,
    "restart_network_progress",
  );
  return {
    known_failure: {
      first_failure: "network",
      network_phase: "reconnect_spawn",
      network_error: "no_memory",
      boot_ordinal: boots[0].ordinal,
      reset_reason: boots[0].reason,
      last_boot_uptime_ms: boots.at(-1).uptime,
      last_startup_uptime_ms: failed.at(-1).uptime,
      statistics_active: active,
    },
    observations: {
      failed_startup_samples: failed.length,
      boot_records: boots.length,
      observed_transitions: 0,
      panic_receipts: 0,
      allocation_receipts: 0,
      network_failure_records: networkFailures,
      statistics_prepared_records: statistics.filter((v) => v.state === "prepared").length,
      statistics_active_records: statistics.filter((v) => v.state === "active").length,
      boot_span_ms: boots.at(-1).uptime - boots[0].uptime,
      startup_failure_span_ms: failed.at(-1).uptime - failed[0].uptime,
      prior_reset_attribution: "unknown",
    },
  };
}
async function installation(root, context, saved, hash) {
  const claim = (await proof(resolve(root, "install-consumed.json"))).value;
  exactObject(claim, ["schema", "context_sha256", "claimed_at_unix_ms", "closed_sequence", "manifest_sha256", "app_elf_sha256"]);
  check(
    claim.schema === "fixed-usb-restart-install-consumed-v1" &&
      claim.context_sha256 === hash &&
      claim.manifest_sha256 === context.manifest_sha256 &&
      claim.app_elf_sha256 === context.app_elf_sha256 &&
      Number.isSafeInteger(claim.claimed_at_unix_ms) &&
      claim.claimed_at_unix_ms > 0,
    "restart_network_claim",
  );
  const inner = restartInnerContext(root, context, "before-install"),
    scope = resolve(root, "before-install"),
    rows = await readRestartStates(scope, inner),
    accounting = await readRestartAccounting(scope, inner, "before");
  baseline(rows.at(-1).state, true);
  check(
    rows.at(-1).sequence === claim.closed_sequence &&
      rows.at(-1).sequence > accounting.observed_sequence &&
      rows.every((r) => !r.state.running && !r.state.failure && !r.state.restart && r.state.renewalsConfirmed === 0),
    "restart_network_journal",
  );
  await requirePreinstallFailure(root, context, claim.closed_sequence);
  check(
    equal(saved.last_authenticated_ledger, accounting.ledger) &&
      saved.before_accounting_sha256 === (await fileDigest(resolve(scope, "no-mining-accounting-before.json"))) &&
      saved.preinstall_failure_review_sha256 === (await fileDigest(resolve(root, "before-install-failure-review.json"))),
    "restart_network_accounting",
  );
  const inputProof = await proof(resolve(root, "install-review-input.json")),
    input = inputProof.value;
  exactObject(input, [
    "schema",
    "source",
    "capture_file",
    "capture_sha256",
    "command_exit_code",
    "owned_children_absent",
    "serial_holders_absent",
  ]);
  check(
    inputProof.sha256 === saved.install_input_sha256 &&
      input.schema === "worker-restart-install-review-input-v1" &&
      input.source === "parent-observed" &&
      input.capture_file === "install-001/flash-monitor.log" &&
      input.command_exit_code === 1 &&
      input.owned_children_absent === true &&
      input.serial_holders_absent === true &&
      input.capture_sha256 === saved.capture_sha256 &&
      input.capture_sha256 === (await fileDigest(resolve(root, input.capture_file))),
    "restart_network_input",
  );
  const flash = await proof(resolve(root, "install-001/flash-command-evidence.json")),
    f = flash.value,
    a = f.fixed_serial_assessment;
  check(
    flash.sha256 === saved.initial_flash_sha256 &&
      f.flash_status === "completed" &&
      f.capture_mode === "noninteractive" &&
      f.capture_status === "timed_out_without_trusted_output" &&
      f.command_kind === "flash-monitor" &&
      f.board === "205" &&
      f.nvs_seed_status === "not_provided" &&
      f.firmware_commit === context.firmware_commit &&
      f.observed_firmware_commit === context.firmware_commit &&
      f.reference_commit === context.reference_commit &&
      f.trust_basis === "none" &&
      f.redaction_mode === "commit-redacted" &&
      f.manifest_path === "[redacted-path]" &&
      f.capture_timeout_seconds === 30 &&
      f.trusted_output === false &&
      f.commit_ready === true &&
      f.monitor_evidence_status === "untrusted" &&
      a?.execution_present === true &&
      a.safe_baseline_confirmed === true &&
      a.startup_complete === true &&
      a.startup_failed === true &&
      a.stable_boot === true &&
      a.retained_failure_history === false &&
      equal(a.issues, ["error_diagnostic", "startup_failed"]),
    "restart_network_install_outcome",
  );
  const observed = await proof(resolve(root, "install-001.observation.json")),
    o = observed.value,
    owner = (await proof(resolve(root, "install-001.host-root.json"))).value,
    armed = (await proof(resolve(root, "install-001.observer-armed.json"))).value;
  check(
    observed.sha256 === saved.install_observer_sha256 &&
      o.schema === "hello-passive-command-observation-v1" &&
      o.rootObserved === true &&
      Number.isSafeInteger(o.observations) &&
      o.observations > 0 &&
      equal(o.failures, []) &&
      equal(o.remaining, []) &&
      o.complete !== false &&
      Array.isArray(o.seen) &&
      o.seen.some((v) => same(v, owner)) &&
      same(owner, armed) &&
      Number.isSafeInteger(o.started_at_unix_ms) &&
      o.started_at_unix_ms <= claim.claimed_at_unix_ms &&
      Number.isSafeInteger(o.finished_at_unix_ms) &&
      o.finished_at_unix_ms >= claim.claimed_at_unix_ms &&
      /^[0-9]+$/u.test(f.timestamp) &&
      Number(f.timestamp) * 1000 + 999 >= claim.claimed_at_unix_ms &&
      Number(f.timestamp) * 1000 <= o.finished_at_unix_ms,
    "restart_network_observer",
  );
  return inspectNetworkFailureCapture(await readFile(resolve(root, input.capture_file)), context);
}
/** Independently re-derive the one admitted sealed network failure; never creates a receipt or effect claim. */
export async function inspectRestartNetworkFailure(root, context) {
  const sealed = await proof(resolve(root, "failed-inventory.json")),
    saved = sealed.value,
    hash = digest(JSON.stringify(context));
  check(
    context.restart_attempt === 2 &&
      context.statistics_startup_required === true &&
      context.install_failure_predecessor?.failed_inventory_sha256 === RESTART_INSTALL_FAILURE_SHA256 &&
      saved.schema === "fixed-usb-restart-network-install-failed-inventory-v1" &&
      saved.outcome === "unverified_network_startup_failure" &&
      saved.auditor_sha256 === RESTART_NETWORK_FAILURE_PRODUCER &&
      saved.context_sha256 === hash &&
      saved.installation_consumed === true &&
      [
        "controlled_restart_consumed",
        "fresh_accounting_after_install",
        "authenticated_preservation_after_install",
        "qualification_pass",
        "continuation_authority",
        "mining_authorized",
      ].every((key) => saved[key] === false),
    "restart_network_failure_shape",
  );
  check(equal(saved.files, await inventory(root)), "restart_network_inventory");
  const snapshot = await verifyArtifactSnapshot(root, context);
  check(snapshot.receipt_sha256 === saved.artifact_snapshot_sha256, "restart_network_snapshot");
  for (const name of [
    "result.json",
    "installation-review.json",
    "install-phase-advanced.json",
    "pre-restart-observation.json",
    "restart-consumed.json",
    "restart-observation.json",
    "restart-failed-observation.json",
    "restart-finished.json",
    "issued.json",
    "consumed.json",
    "iterative.fault.json",
  ])
    await missing(resolve(root, name));
  check(
    (await readdir(resolve(root, "after-install"))).length === 0 &&
      !(await readdir(root)).some(
        (name) => /^(flash|cycle)-[0-9]/u.test(name) || (/^install-[0-9]/u.test(name) && !/^install-001(?:\.|$)/u.test(name)),
      ),
    "restart_network_after_activity",
  );
  const failure = await proof(resolve(root, "restart-failure.json")),
    cleanup = await proof(resolve(root, "host-cleanup.json"));
  check(
    failure.sha256 === saved.failure_sha256 &&
      equal(failure.value, { schema: "fixed-usb-restart-failure-v1", context_sha256: hash, code: "restart_install_identity_startup" }),
    "restart_network_cause",
  );
  check(
    cleanup.sha256 === saved.cleanup_sha256 &&
      equal(cleanup.value, {
        schema: "worker-restart-host-cleanup-v1",
        source: "parent-observed",
        browser_closed: true,
        supervisor_exited: true,
        supervisor_exit_code: 0,
        listener_absent: true,
        owned_children_absent: true,
        serial_holders_absent: true,
      }),
    "restart_network_cleanup",
  );
  const supplement = await proof(resolve(root, "supervisor.observation.json")),
    s = supplement.value;
  check(
    s.schema === "hello-passive-command-observation-v1" &&
      s.rootObserved === true &&
      s.complete === false &&
      equal(s.failures, []) &&
      s.remaining === undefined &&
      s.finished_at_unix_ms === undefined &&
      Number.isSafeInteger(s.observations) &&
      s.observations > 0 &&
      equal(saved.supplemental_observer, {
        sha256: supplement.sha256,
        complete: false,
        observations: s.observations,
        cleanup_proven_by_supplement: false,
      }),
    "restart_network_supplement",
  );
  const stop = (await proof(resolve(root, "supervisor-stop-request.json"))).value,
    owner = (await proof(resolve(root, "supervisor.host-root.json"))).value;
  check(
    stop.schema === "parent-observed-supervisor-stop-v1" &&
      stop.signal === "SIGTERM" &&
      same(stop.owner, owner) &&
      Array.isArray(stop.owned) &&
      stop.owned.some((v) => same(v, stop.target)),
    "restart_network_stop",
  );
  const facts = await installation(root, context, saved, hash);
  check(
    equal(facts.known_failure, saved.known_failure) && equal(facts.observations, saved.observations),
    "restart_network_capture_changed",
  );
  return { root, context, known_failure: facts.known_failure, binding: { root, failed_inventory_sha256: sealed.sha256 } };
}
export async function readRestartNetworkFailure(root, operations = {}) {
  root = await canonicalDirectory(root);
  await protectedPath(root, true);
  const sealPath = resolve(root, "failed-inventory.json");
  await protectedPath(sealPath);
  const bytes = await readFile(sealPath),
    hash = digest(bytes);
  check(hash === RESTART_NETWORK_FAILURE_SHA256, "restart_network_failure_anchor");
  const sealed = { value: JSON.parse(bytes.toString("utf8")), sha256: hash };
  const contextFile = await proof(resolve(root, "context.json")),
    entry = sealed.value.files?.find((value) => value.path === "context.json" && value.type === "file");
  check(entry && entry.sha256 === contextFile.sha256, "restart_network_context_changed");
  // Inspect the predecessor's generation before recursive context validation: this class can only be attempt2.
  check(contextFile.value.context?.restart_attempt === 2, "restart_network_ancestry");
  const context = await loadRestartContext(root, { historical: true, operations });
  const result = await inspectRestartNetworkFailure(root, context);
  check(result.binding.failed_inventory_sha256 === RESTART_NETWORK_FAILURE_SHA256, "restart_network_failure_anchor");
  return result;
}
