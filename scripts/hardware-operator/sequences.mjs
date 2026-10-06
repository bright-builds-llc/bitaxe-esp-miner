// Time-bound chains that must land inside one recovery's 120-second freshness window.
import { dirname, relative, resolve } from "node:path";
import { proof } from "../str005-noise-serial/files.mjs";
import { HardwareOperatorError, refuse } from "./errors.mjs";
import { processSnapshot } from "../host-stalls/capture.mjs";
import { absent, breadcrumb, isLive, liveIdentity, privateDirectory, waitFor } from "./host.mjs";
import { detect, runJust, startJustDetached } from "./just.mjs";
import { admitOwnerFinish, stopAndFinish } from "./owner-finish.mjs";
import { OWNERS, ownerLayout, windowRemainingMs } from "./owners.mjs";

const SERVE_STOP_MS = 30_000;

async function readProof(root, path, missingCode) {
  try { return (await proof(root, relative(root, path))).value; } catch (error) {
    if (error.code === "ENOENT") throw new HardwareOperatorError(missingCode);
    throw new HardwareOperatorError(`${missingCode.replace(/_missing$/u, "")}_invalid`);
  }
}

/** Stop a serve that never became ready, so no untracked serve keeps the Gate port. */
async function stopServeGroup(pgid, operations) {
  try { process.kill(-pgid, "SIGTERM"); } catch (error) { if (error.code !== "ESRCH") throw error; }
  const gone = await waitFor(async () => (await (operations.processSnapshot ?? processSnapshot)())
    .some((row) => row.pgid === pgid && !String(row.state).startsWith("Z")) ? null : true, { timeoutMs: SERVE_STOP_MS });
  refuse(gone === true, "restart_serve_stop_unproved");
}

/** Wait for the restart serve's owner record, or its exit; a serve that is not ready is stopped. */
async function awaitServe(root, serverOwner, pgid, timeoutMs, operations) {
  const maybeServeIdentity = await liveIdentity(pgid, operations);
  let maybeReady = null;
  try {
    maybeReady = await waitFor(async () => {
      // A SyntaxError is a record the serve is still writing.
      try { return { record: (await proof(root, relative(root, serverOwner))).value }; } catch (error) {
        if (error.code !== "ENOENT" && !(error instanceof SyntaxError)) throw error;
      }
      return maybeServeIdentity && !(await isLive(maybeServeIdentity, operations)) ? { exited: true } : null;
    }, { timeoutMs });
  } finally {
    if (!maybeReady?.record) await stopServeGroup(pgid, operations);
  }
  return maybeReady;
}

function requireWindow(startedAtUnixMs) {
  const remaining = windowRemainingMs(startedAtUnixMs, Date.now());
  refuse(remaining > 0, "recovery_window_expired");
  return remaining;
}

/**
 * Restart: wait for the recovery collection, stop and finish the recovery stage, write the restart
 * detector, and start the restart-stage serve, all before the recovery's window closes.
 */
export async function restartSequence({ name, root }, operations = {}) {
  process.umask(0o077);
  const owner = OWNERS[name];
  refuse(owner?.kind === "restart", "owner_not_restart");
  root = await privateDirectory(root, "owner_root_missing");
  const restart = ownerLayout(name, root, "restart"), parent = restart.parent;
  const paths = { detector: resolve(parent, "restart-detector.stdout.log"), detectorStderr: resolve(parent, "restart-detector.stderr.log"),
    serveStdout: resolve(parent, "restart-serve.stdout.log"), serveStderr: resolve(parent, "restart-serve.stderr.log") };
  await admitOwnerFinish({ name, root, maybeStage: "recovery" });
  for (const path of Object.values(paths)) await absent(path, "restart_output_exists");
  await absent(restart.stageRoot, "restart_stage_exists");
  const finished = await stopAndFinish({ name, root, maybeStage: "recovery", waitCollection: true }, operations);
  refuse(finished.summary.finish_exit === 0, "recovery_finish_failed");
  const result = await readProof(finished.layout.stageRoot, resolve(finished.layout.stageRoot, "result.json"), "recovery_result_missing");
  refuse(result.current_recovery_complete === true, "recovery_incomplete");
  requireWindow(finished.maybeCollectionStartedAt);
  await detect(paths.detector, paths.detectorStderr, operations);
  const remaining = requireWindow(finished.maybeCollectionStartedAt);
  await breadcrumb(parent, "restart-sequence", { step: "restart_detector_written" });
  const serve = await startJustDetached([name, "serve", "--private-root", root, "--stage", "restart"],
    { stdoutPath: paths.serveStdout, stderrPath: paths.serveStderr }, operations);
  const maybeReady = await awaitServe(root, restart.serverOwner, serve.pid, operations.serveReadyMs ?? remaining, operations);
  refuse(maybeReady !== null && maybeReady.exited !== true, maybeReady?.exited ? "restart_serve_exited" : "restart_serve_not_ready");
  await breadcrumb(parent, "restart-sequence", { step: "restart_serving" });
  return { event: "restart_serving", recovery_finish_exit: 0, window_remaining_ms: requireWindow(finished.maybeCollectionStartedAt),
    serve_stdout: paths.serveStdout };
}

function proofFields(value) {
  const fields = { source: value?.firmware_commit, elf: value?.app_elf_sha256, physical: value?.physical_identity_sha256, observedAt: value?.observed_at_unix_ms };
  refuse(/^[0-9a-f]{40}$/u.test(fields.source ?? "") && /^[0-9a-f]{64}$/u.test(fields.elf ?? "") &&
    /^[0-9a-f]{64}$/u.test(fields.physical ?? "") && Number.isSafeInteger(fields.observedAt), "recovery_proof_invalid");
  return fields;
}

/**
 * Recovery plus core-dump read: finish a recovery owner, then start one read-only `just core-dump-read`
 * bound to the fresh `current-recovery.json` before that proof's window closes.
 */
export async function recoveryCoreDump({ name, root, coreRoot, waitCollection }, operations = {}) {
  process.umask(0o077);
  refuse(OWNERS[name]?.writesRecoveryProof === true, "owner_not_recovery");
  root = await privateDirectory(root, "owner_root_missing");
  const parent = dirname(root), proofPath = resolve(root, "current-recovery.json");
  const paths = { detector: resolve(parent, "core-detector.stdout.log"), detectorStderr: resolve(parent, "core-detector.stderr.log"),
    readStdout: resolve(parent, "core-read.stdout.log"), readStderr: resolve(parent, "core-read.stderr.log") };
  await admitOwnerFinish({ name, root });
  for (const path of Object.values(paths)) await absent(path, "core_output_exists");
  await absent(proofPath, "recovery_proof_exists");
  await absent(coreRoot, "core_root_exists");
  await privateDirectory(dirname(coreRoot), "core_parent_missing");
  const finished = await stopAndFinish({ name, root, waitCollection }, operations);
  refuse(finished.summary.finish_exit === 0, "recovery_finish_failed");
  const fields = proofFields(await readProof(root, proofPath, "recovery_proof_missing"));
  requireWindow(fields.observedAt);
  const device = await detect(paths.detector, paths.detectorStderr, operations);
  refuse(device.physical === fields.physical, "detector_physical_mismatch");
  await breadcrumb(parent, "recovery-core-dump", { step: "core_detector_written" });
  requireWindow(fields.observedAt);
  const readExit = await runJust(["core-dump-read", "--board", "205", "--port", device.port, "--expected-physical-sha256", fields.physical,
    "--expected-installed-source", fields.source, "--expected-installed-elf", fields.elf, "--recovery-proof", proofPath, "--private-root", coreRoot],
  { stdoutPath: paths.readStdout, stderrPath: paths.readStderr }, operations);
  await breadcrumb(parent, "recovery-core-dump", { step: "core_dump_read_exited", exit: readExit });
  return { event: "core_dump_read_finished", recovery_finish_exit: 0, read_exit: readExit, read_stdout: paths.readStdout };
}
