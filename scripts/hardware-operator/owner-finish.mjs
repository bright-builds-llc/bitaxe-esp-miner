// Stop a detached owner by its recorded identity, prove release, write the final detector and run its `finish`.
import { lstat, readFile } from "node:fs/promises";
import { relative } from "node:path";
import { proof } from "../str005-noise-serial/files.mjs";
import { requireGone } from "../str005-noise-serial/host-resources.mjs";
import { HardwareOperatorError, refuse } from "./errors.mjs";
import { absent, breadcrumb, privateDirectory, requirePortFree, terminate, waitFor } from "./host.mjs";
import { detect, runJust } from "./just.mjs";
import { ownerLayout, windowRemainingMs } from "./owners.mjs";

const OWNER_STOP_MS = 30_000;
const COLLECTION_BEGIN_GRACE_MS = 30_000;

async function readServerOwner(layout) {
  let record;
  try { record = (await proof(layout.root, relative(layout.root, layout.serverOwner))).value; } catch (error) {
    if (error.code === "ENOENT") throw new HardwareOperatorError("owner_record_missing");
    throw new HardwareOperatorError("owner_record_invalid");
  }
  const owner = record?.owner;
  refuse(Number.isSafeInteger(owner?.pid) && owner.pid > 0 && Number.isSafeInteger(owner.pgid) && owner.pgid > 0 &&
    typeof owner.startedAt === "string" && owner.startedAt.length > 0 && Number.isSafeInteger(record.port) && record.port >= 1 && record.port <= 65535,
  "owner_record_invalid");
  return { owner: { pid: owner.pid, pgid: owner.pgid, startedAt: owner.startedAt }, port: record.port };
}

/** A missing or still-being-written record reads as not yet present; the caller's bound decides. */
async function maybeJson(path) {
  try { return JSON.parse(await readFile(path, "utf8")); } catch (error) {
    if (error.code === "ENOENT" || error instanceof SyntaxError) return null;
    throw error;
  }
}

const maybeExists = (path) => lstat(path).then(() => true, (error) => { if (error.code === "ENOENT") return null; throw error; });

/** Wait for the page's collection to finish, but never past the freshness window it opened. */
export async function awaitCollection(layout, operations = {}) {
  const maybeBegin = await waitFor(() => maybeJson(layout.collectionBegin), { timeoutMs: operations.collectionBeginGraceMs ?? COLLECTION_BEGIN_GRACE_MS });
  refuse(maybeBegin !== null, "collection_not_begun");
  const remaining = windowRemainingMs(maybeBegin.startedAtUnixMs, Date.now());
  refuse(remaining > 0, "recovery_window_expired");
  refuse(await waitFor(() => maybeExists(layout.collectionFinished), { timeoutMs: remaining }) === true, "collection_unfinished");
  return maybeBegin.startedAtUnixMs;
}

async function awaitGroupGone(owner, operations) {
  const gone = await waitFor(async () => {
    try { await requireGone([owner], operations); return true; } catch { return null; }
  }, { timeoutMs: operations.ownerStopMs ?? OWNER_STOP_MS });
  refuse(gone === true, "owner_exit_unproved");
}

/** Refuse every input problem before the first effect (the SIGTERM). */
export async function admitOwnerFinish({ name, root, maybeStage }) {
  root = await privateDirectory(root, "owner_root_missing");
  const layout = ownerLayout(name, root, maybeStage);
  await privateDirectory(layout.parent, "owner_parent_missing");
  for (const path of [layout.finalDetector, layout.finalDetectorStderr, layout.finishStdout, layout.finishStderr]) await absent(path, "owner_output_exists");
  return { layout, record: await readServerOwner(layout) };
}

/**
 * Stop and finish one owner. With `waitCollection`, first wait for its recovery page collection to finish
 * and return the collection start so callers can keep later steps inside the same freshness window.
 */
export async function stopAndFinish(request, operations = {}) {
  process.umask(0o077);
  const { layout, record } = await admitOwnerFinish(request);
  const maybeCollectionStartedAt = request.waitCollection ? await awaitCollection(layout, operations) : null;
  const stop = await terminate(record.owner, { timeoutMs: operations.ownerStopMs ?? OWNER_STOP_MS, failureCode: "owner_stop_failed" }, operations);
  await awaitGroupGone(record.owner, operations);
  requirePortFree(record.port, operations);
  await breadcrumb(layout.parent, "owner-finish", { owner: request.name, stage: request.maybeStage ?? null, step: "owner_released", stop });
  await detect(layout.finalDetector, layout.finalDetectorStderr, operations);
  const finishExit = await runJust(layout.finishArguments, { stdoutPath: layout.finishStdout, stderrPath: layout.finishStderr }, operations);
  await breadcrumb(layout.parent, "owner-finish", { owner: request.name, stage: request.maybeStage ?? null, step: "finish_exited", exit: finishExit });
  return { layout, maybeCollectionStartedAt, summary: { event: "owner_finished", owner: request.name, stage: request.maybeStage ?? null, stop,
    finish_exit: finishExit, finish_stdout: layout.finishStdout } };
}
