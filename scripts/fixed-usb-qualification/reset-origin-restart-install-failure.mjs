import { readFile, readdir } from "node:fs/promises";
import { resolve } from "node:path";
import { isDeepStrictEqual as equal } from "node:util";
import { canonicalDirectory, digest, exactObject, fileDigest, missing, requireCondition as check } from "./contract.mjs";
import { baseline, inventory, proof } from "./cadence-premining-evidence.mjs";
import { verifyArtifactSnapshot } from "./snapshot.mjs";
import { readRestartStates, readRestartAccounting } from "./reset-origin-restart-state.mjs";
import { restartInnerContext, loadRestartContext } from "./reset-origin-restart-context.mjs";

export const RESTART_INSTALL_FAILURE_SHA256 = "883b0a4f40eb97c59ec03f517d283d556e4ea3412de9e807e44371fdcf7ab3f1";
const PRODUCER = "72064ed17475fcf45c96d61d7ed95b7330e4830f8c6982ec6f644c7c7abd7101";

export function inspectStatisticsFailureCapture(bytes, context) {
  check(bytes.length > 0 && bytes.length <= 1048576, "restart_failed_capture_bound");
  const lines = bytes.toString("utf8").split("\n");
  lines.pop();
  const boots = [],
    failed = [],
    memory = [];
  let identities = 0;
  for (const line of lines) {
    check(
      !/^(rust_panic_receipt|allocation_failure|allocation_failure_context)/u.test(line) &&
        !/Guru Meditation Error|abort\(\) was called|stack overflow/iu.test(line),
      "restart_failed_capture_other_failure",
    );
    if (line.startsWith("usb_reboot_discriminator")) {
      const match =
        /^usb_reboot_discriminator schema=v1 boot_ordinal=([0-9]+) reset_reason=([a-z_]+) uptime_ms=([0-9]+) redacted=true$/u.exec(line);
      check(match, "restart_failed_capture_boot");
      const boot = { ordinal: Number(match[1]), reason: match[2], uptime: Number(match[3]) },
        previous = boots.at(-1);
      check(
        Number.isSafeInteger(boot.ordinal) &&
          boot.ordinal > 0 &&
          Number.isSafeInteger(boot.uptime) &&
          (!previous || (boot.ordinal === previous.ordinal && boot.reason === previous.reason && boot.uptime >= previous.uptime)),
        "restart_failed_capture_transition",
      );
      boots.push(boot);
    } else if (line.startsWith("usb_startup")) {
      const match =
        /^usb_startup schema=v1 stage=([a-z_]+) state=(entered|complete|failed) first_failure=([a-z_]+) uptime_ms=([0-9]+) redacted=true$/u.exec(
          line,
        );
      check(match, "restart_failed_capture_startup");
      if (match[3] !== "none") {
        check(match[1] === "runtime_ready" && match[2] === "complete" && match[3] === "statistics", "restart_failed_capture_other_failure");
        const uptime = Number(match[4]);
        check(Number.isSafeInteger(uptime) && (!failed.length || uptime >= failed.at(-1)), "restart_failed_capture_clock");
        failed.push(uptime);
      }
    } else if (line.startsWith("usb_runtime_identity")) {
      const match = /^usb_runtime_identity schema=v1 firmware_commit=([a-f0-9]{40}) app_elf_sha256=([a-f0-9]{64}) redacted=true$/u.exec(
        line,
      );
      check(match && match[1] === context.firmware_commit && match[2] === context.app_elf_sha256, "restart_failed_capture_identity");
      identities++;
    }
    const m =
      /^usb_memory_checkpoint stage=(statistics_start|statistics_started) free_bytes=([0-9]+) largest_block_bytes=([0-9]+) reserve_bytes=([0-9]+) redacted=true$/u.exec(
        line,
      );
    if (m && !memory.some((value) => value.stage === m[1]))
      memory.push({
        stage: m[1],
        free_bytes: Number(m[2]),
        largest_block_bytes: Number(m[3]),
        reserve_bytes: Number(m[4]),
        capability_class: "internal_dma_8bit",
      });
  }
  check(
    boots.length > 1 && failed.length > 1 && identities > 0 && boots.at(-1).uptime > boots[0].uptime && failed.at(-1) > failed[0],
    "restart_failed_capture_progress",
  );
  return {
    observations: {
      first_failure: "statistics",
      failed_startup_samples: failed.length,
      boot_ordinal: boots[0].ordinal,
      initial_reset_category: boots[0].reason,
      observed_transitions: 0,
      panic_receipts: 0,
      allocation_receipts: 0,
      memory,
      exact_pthread_errno: "unavailable",
    },
    known_failure: {
      first_failure: "statistics",
      boot_ordinal: boots[0].ordinal,
      reset_reason: boots[0].reason,
      last_boot_uptime_ms: boots.at(-1).uptime,
      last_startup_uptime_ms: failed.at(-1),
    },
  };
}

/** Re-derive the known failed installation; it never establishes post-install authentication or health. */
export async function inspectRestartInstallFailure(root, context) {
  const seal = await proof(resolve(root, "failed-inventory.json")),
    saved = seal.value;
  check(
    context.restart_attempt === undefined &&
      context.install_failure_predecessor === undefined &&
      saved.schema === "fixed-usb-restart-install-failed-inventory-v1" &&
      saved.outcome === "unverified_statistics_startup_failure" &&
      saved.auditor_sha256 === PRODUCER &&
      saved.context_sha256 === digest(JSON.stringify(context)) &&
      saved.installation_consumed === true &&
      [
        "fresh_accounting_after_install",
        "authenticated_preservation_after_install",
        "controlled_restart_consumed",
        "qualification_pass",
        "continuation_authority",
        "mining_authorized",
      ].every((key) => saved[key] === false),
    "restart_install_failure_shape",
  );
  const snapshot = await verifyArtifactSnapshot(root, context);
  check(
    snapshot.receipt_sha256 === saved.artifact_snapshot_sha256 && equal(saved.files, await inventory(root)),
    "restart_install_failure_inventory",
  );
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
  check((await readdir(resolve(root, "after-install"))).length === 0, "restart_install_failure_after_activity");
  const failure = await proof(resolve(root, "restart-failure.json")),
    cleanup = await proof(resolve(root, "host-cleanup.json"));
  check(
    failure.sha256 === saved.failure_sha256 &&
      equal(failure.value, {
        schema: "fixed-usb-restart-failure-v1",
        context_sha256: saved.context_sha256,
        code: "restart_install_identity_startup",
      }),
    "restart_install_failure_cause",
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
    "restart_install_failure_cleanup",
  );
  const before = restartInnerContext(root, context, "before-install"),
    scope = resolve(root, "before-install"),
    records = await readRestartStates(scope, before),
    accounting = await readRestartAccounting(scope, before, "before");
  baseline(records.at(-1).state, true);
  check(
    records.at(-1).sequence > accounting.observed_sequence &&
      records.every((row) => !row.state.failure && !row.state.restart && !row.state.running && row.state.renewalsConfirmed === 0) &&
      equal(saved.last_authenticated_ledger, accounting.ledger) &&
      (await fileDigest(resolve(scope, "no-mining-accounting-before.json"))) === saved.before_accounting_sha256,
    "restart_install_failure_accounting",
  );
  const claim = (await proof(resolve(root, "install-consumed.json"))).value;
  check(
    claim.schema === "fixed-usb-restart-install-consumed-v1" &&
      claim.context_sha256 === saved.context_sha256 &&
      claim.closed_sequence === records.at(-1).sequence &&
      claim.manifest_sha256 === context.manifest_sha256 &&
      claim.app_elf_sha256 === context.app_elf_sha256,
    "restart_install_failure_claim",
  );
  const input = (await proof(resolve(root, "install-review-input.json"))).value,
    flash = await proof(resolve(root, "install-001/flash-command-evidence.json")),
    f = flash.value,
    a = f.fixed_serial_assessment;
  check(
    input.capture_file === "install-001/flash-monitor.log" &&
      input.command_exit_code === 1 &&
      input.owned_children_absent === true &&
      input.serial_holders_absent === true &&
      input.capture_sha256 === (await fileDigest(resolve(root, input.capture_file))),
    "restart_install_failure_capture_binding",
  );
  check(
    flash.sha256 === saved.initial_flash_sha256 &&
      f.flash_status === "completed" &&
      f.capture_mode === "noninteractive" &&
      f.capture_status === "timed_out_without_trusted_output" &&
      f.nvs_seed_status === "not_provided" &&
      f.firmware_commit === context.firmware_commit &&
      f.observed_firmware_commit === context.firmware_commit &&
      f.trusted_output === false &&
      f.commit_ready === true &&
      a?.startup_complete === true &&
      a.startup_failed === true &&
      a.safe_baseline_confirmed === true &&
      a.stable_boot === true &&
      equal(a.issues, ["startup_failed"]),
    "restart_install_failure_outcome",
  );
  const facts = inspectStatisticsFailureCapture(await readFile(resolve(root, input.capture_file)), context);
  check(equal(facts.observations, saved.observations), "restart_install_failure_observations");
  return { root, context, known_failure: facts.known_failure, binding: { root, failed_inventory_sha256: seal.sha256 } };
}
export async function readRestartInstallFailure(root, operations = {}) {
  root = await canonicalDirectory(root);
  check((await proof(resolve(root, "failed-inventory.json"))).sha256 === RESTART_INSTALL_FAILURE_SHA256, "restart_install_failure_anchor");
  const context = await loadRestartContext(root, { historical: true, operations });
  return inspectRestartInstallFailure(root, context);
}
