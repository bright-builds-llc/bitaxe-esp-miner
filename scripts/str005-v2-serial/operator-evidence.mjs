import { readFile } from "node:fs/promises";
import { resolve } from "node:path";
import { inventory, privateRoot, proof, retain, verifyInventory } from "../str005-noise-serial/files.mjs";
import { readStoppedOperator, requireOperatorAbsent } from "./operator-disposition.mjs";
import { readJournal } from "./journal.mjs";
import { check, sha256 } from "./values.mjs";

export const managedOperator = context => ["str005-v2-serial-context-v5", "str005-v2-serial-context-v6"].includes(context.schema);

/** Even an unsuccessful campaign cannot seal while its operator can still write. */
export async function requireOperatorStopped(root, context, operations = {}) {
  if (managedOperator(context)) await requireOperatorAbsent(root, context, operations);
}

/** Retain available terminal bytes without inventing missing success records. */
export async function snapshotOperator(root, context, operations = {}) {
  if (!managedOperator(context)) return;
  await requireOperatorStopped(root, context, operations);
  const directory = `${root}.operator`;
  try { await privateRoot(directory); }
  catch (error) { if (error.code === "ENOENT") return; throw error; }
  const files = await inventory(directory);
  for (const file of files) {
    const bytes = await readFile(resolve(directory, file.path));
    check(bytes.length === file.length && sha256(bytes) === file.sha256, "v2_operator_snapshot_changed");
    await retain(resolve(root, "final-inputs/operator", file.path), bytes);
  }
  await verifyInventory(directory, files);
  await requireOperatorStopped(root, context, operations);
}

/** Full success requires each planned operation once, not merely a stopped owner. */
export async function judgeOperator(root, context, operations = {}) {
  if (!managedOperator(context)) return null;
  const directory = operations.cleanupSnapshot ? resolve(root, "final-inputs/operator") : `${root}.operator`;
  const observed = await readStoppedOperator(root, context, { ...operations, directory });
  check(observed.disposition.value.cleanupRecorded === true && observed.stopped.value.maybeCode === null,
    "v2_operator_incomplete");
  const server = (await proof(root, "server-owner.json")).value;
  const last = (await readJournal(root, context)).at(-1);
  check(server.owner.ppid === observed.disposition.value.owner.pid && last &&
    observed.stopped.value.maybeSupervisorOrigin === server.origin && observed.stopped.value.maybeAction === "stop" &&
    observed.stopped.value.maybeIndex === null && observed.stopped.value.lastSequence === last.sequence &&
    observed.stopped.value.lastStateSha256 === sha256(JSON.stringify(last)), "v2_operator_owner_join");
  const expected = new Set([...context.install_indices.map(index => `install-${index}`), "prepare-cleanup", "finish-cleanup", "stop"]);
  const seen = new Set();
  for (const file of observed.files.filter(item => /^request-[a-f0-9]{32}\.json$/u.test(item.path))) {
    const request = (await proof(directory, file.path)).value;
    const result = (await proof(directory, `result-${request.requestId}.json`)).value;
    const key = request.action === "install" ? `install-${request.payload.index}` : request.action;
    check(expected.has(key) && !seen.has(key) && result.status === "succeeded" && result.code === null,
      "v2_operator_incomplete");
    if (key === "stop") check(observed.stopped.value.maybeRequestId === request.requestId, "v2_operator_owner_join");
    seen.add(key);
  }
  check(seen.size === expected.size, "v2_operator_incomplete");
  return { dispositionSha256: observed.disposition.sha256, operations: seen.size };
}
