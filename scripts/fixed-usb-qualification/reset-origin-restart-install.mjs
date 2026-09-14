import { readFile, readdir } from "node:fs/promises";
import { resolve } from "node:path";
import { isDeepStrictEqual as equal } from "node:util";
import {
  digest,
  exactObject,
  fileDigest,
  missing,
  protectedPath,
  readJson,
  requireCondition as check,
  within,
  writeNew,
} from "./contract.mjs";
import { proof, baseline } from "./cadence-premining-evidence.mjs";
import { readRestartStates as readNoMiningStates, readRestartAccounting } from "./reset-origin-restart-state.mjs";
import { loadRestartContext, restartInnerContext } from "./reset-origin-restart-context.mjs";
import { saveRestartFailure } from "./reset-origin-restart-failure.mjs";
import { inspectRestartStartup } from "./reset-origin-restart-startup.mjs";

const sameProcess = (a, b) =>
  Number.isSafeInteger(a.pid) &&
  a.pid > 0 &&
  Number.isSafeInteger(a.pgid) &&
  a.pgid > 0 &&
  typeof a.startedAt === "string" &&
  a.startedAt.length > 0 &&
  a.startedAt.length <= 200 &&
  a.pid === b.pid &&
  a.pgid === b.pgid &&
  a.startedAt === b.startedAt;
export async function consumeRestartInstall(root, operations = {}) {
  root = resolve(root);
  const context = await loadRestartContext(root, { operations });
  await missing(resolve(root, "install-001"));
  const records = await readNoMiningStates(resolve(root, "before-install"), restartInnerContext(root, context, "before-install"));
  baseline(records.at(-1).state, true);
  const before = await readRestartAccounting(
    resolve(root, "before-install"),
    restartInnerContext(root, context, "before-install"),
    "before",
  );
  check(
    records.at(-1).sequence > before.observed_sequence &&
      records.at(-1).state.preservation.baseline_id === before.state.preservation.baseline_id,
    "restart_install_baseline_join",
  );
  const armed = await proof(resolve(root, "install-001.observer-armed.json")),
    owner = await proof(resolve(root, "install-001.host-root.json"));
  check(sameProcess(armed.value, owner.value), "restart_install_not_armed");
  const at = (operations.now ?? Date.now)();
  check(Number.isSafeInteger(at) && at > 0, "restart_install_clock");
  await missing(resolve(root, "restart-failure.json"));
  await writeNew(resolve(root, "install-consumed.json"), {
    schema: "fixed-usb-restart-install-consumed-v1",
    context_sha256: digest(JSON.stringify(context)),
    claimed_at_unix_ms: at,
    closed_sequence: records.at(-1).sequence,
    manifest_sha256: context.manifest_sha256,
    app_elf_sha256: context.app_elf_sha256,
  });
  // Keep an in-flight claim consumed if another owner records failure during its write.
  await missing(resolve(root, "restart-failure.json"));
  return { install_consumed: true, maximum_installations: 1, mining_authorized: false };
}
async function installation(root, context, input) {
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
    input.schema === "worker-restart-install-review-input-v1" &&
      input.source === "parent-observed" &&
      input.capture_file === "install-001/flash-monitor.log" &&
      [0, 1].includes(input.command_exit_code) &&
      input.owned_children_absent === true &&
      input.serial_holders_absent === true,
    "restart_install_host_cleanup",
  );
  const capture = within(resolve(root, "install-001"), resolve(root, input.capture_file));
  await protectedPath(capture);
  check((await fileDigest(capture)) === input.capture_sha256, "restart_install_capture_changed");
  const claim = (await proof(resolve(root, "install-consumed.json"))).value;
  exactObject(claim, ["schema", "context_sha256", "claimed_at_unix_ms", "closed_sequence", "manifest_sha256", "app_elf_sha256"]);
  check(
    claim.schema === "fixed-usb-restart-install-consumed-v1" &&
      claim.context_sha256 === digest(JSON.stringify(context)) &&
      claim.manifest_sha256 === context.manifest_sha256 &&
      claim.app_elf_sha256 === context.app_elf_sha256 &&
      Number.isSafeInteger(claim.claimed_at_unix_ms) &&
      claim.claimed_at_unix_ms > 0,
    "restart_install_claim",
  );
  check(
    !(await readdir(root)).some((name) => /^(flash|install)-[0-9]/u.test(name) && !/^install-001(?:\.|$)/u.test(name)),
    "restart_extra_installation",
  );
  const flashed = await proof(resolve(root, "install-001/flash-command-evidence.json")),
    f = flashed.value,
    a = f.fixed_serial_assessment;
  check(
    f.flash_status === "completed" &&
      f.capture_mode === "noninteractive" &&
      f.command_kind === "flash-monitor" &&
      f.board === "205" &&
      (f.monitor_log_sha256 === undefined || f.monitor_log_sha256 === input.capture_sha256) &&
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
      a.retained_failure_history !== true &&
      Array.isArray(a.issues),
    "restart_install_identity_startup",
  );
  const startup = inspectRestartStartup(await readFile(capture), context);
  const legacy =
    a.stable_boot === true &&
    a.issues.length === 0 &&
    f.trusted_output === true &&
    f.commit_ready === true &&
    f.monitor_evidence_status === "trusted" &&
    ["completed", "timed_out_after_trusted_output"].includes(f.capture_status) &&
    input.command_exit_code === 0;
  const historicalCategory =
    startup.initial_reset_category === "panic" &&
    a.stable_boot === false &&
    equal([...a.issues].sort(), ["insufficient_advancing_samples", "reboot_observed"]) &&
    f.trusted_output === false &&
    f.commit_ready === false &&
    f.monitor_evidence_status === "untrusted" &&
    f.capture_status === "timed_out_without_trusted_output" &&
    input.command_exit_code === 1;
  check(legacy || historicalCategory, "restart_install_legacy_grade");
  const observed = await proof(resolve(root, "install-001.observation.json")),
    o = observed.value;
  const owner = (await proof(resolve(root, "install-001.host-root.json"))).value,
    armed = (await proof(resolve(root, "install-001.observer-armed.json"))).value;
  check(
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
      o.seen.some((p) => sameProcess(p, owner)) &&
      sameProcess(owner, armed) &&
      Number.isSafeInteger(o.started_at_unix_ms) &&
      o.started_at_unix_ms <= claim.claimed_at_unix_ms &&
      Number.isSafeInteger(o.finished_at_unix_ms) &&
      o.finished_at_unix_ms >= claim.claimed_at_unix_ms &&
      /^[0-9]+$/u.test(f.timestamp) &&
      Number(f.timestamp) * 1000 + 999 >= claim.claimed_at_unix_ms &&
      Number(f.timestamp) * 1000 <= o.finished_at_unix_ms,
    "restart_install_ownership",
  );
  const records = await readNoMiningStates(resolve(root, "before-install"), restartInnerContext(root, context, "before-install"));
  check(records.at(-1).sequence === claim.closed_sequence, "restart_install_closed_journal");
  baseline(records.at(-1).state, true);
  return {
    startup,
    legacy_qualified: legacy,
    flash_sha256: flashed.sha256,
    observer_sha256: observed.sha256,
    capture_sha256: input.capture_sha256,
    claim_sha256: await fileDigest(resolve(root, "install-consumed.json")),
  };
}
export async function reviewRestartInstallation(root, inputPath, operations = {}) {
  root = resolve(root);
  const context = await loadRestartContext(root, { operations });
  await protectedPath(inputPath);
  const input = await readJson(inputPath);
  let evidence;
  try {
    evidence = await installation(root, context, input);
  } catch (error) {
    await saveRestartFailure(root, context, error);
    throw error;
  }
  await writeNew(resolve(root, "installation-review.json"), {
    schema: "fixed-usb-restart-installation-review-v1",
    context_sha256: digest(JSON.stringify(context)),
    input: { path: resolve(inputPath), sha256: await fileDigest(inputPath) },
    evidence,
    legacy_result_modified: false,
    observation_required: true,
  });
  return {
    installation_factually_admitted: true,
    legacy_qualified: evidence.legacy_qualified,
    new_observation_required: true,
    restart_authorized: false,
  };
}
export async function requireRestartInstallation(root, context) {
  const saved = (await proof(resolve(root, "installation-review.json"))).value;
  exactObject(saved, ["schema", "context_sha256", "input", "evidence", "legacy_result_modified", "observation_required"]);
  check(
    saved.schema === "fixed-usb-restart-installation-review-v1" &&
      saved.context_sha256 === digest(JSON.stringify(context)) &&
      saved.legacy_result_modified === false &&
      saved.observation_required === true &&
      (await fileDigest(saved.input.path)) === saved.input.sha256,
    "restart_install_review_changed",
  );
  check(equal(saved.evidence, await installation(root, context, await readJson(saved.input.path))), "restart_install_evidence_changed");
  return saved.evidence;
}
