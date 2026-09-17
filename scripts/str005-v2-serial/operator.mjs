import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { loadEffectContext } from "./context.mjs";
import { baseline, readJournal } from "./journal.mjs";
import { observeCommand, observeOwnedExit, monotonicHostMs } from "../str005-noise-serial/operator.mjs";
import { proof } from "../str005-noise-serial/files.mjs";
import { check } from "./values.mjs";
export { observeOwnedExit, monotonicHostMs };

/** Reuse the real process observer; the new child independently admits this scope and context. */
export async function installCandidate(root, index, operations = {}) {
  const context = await loadEffectContext(root, operations);
  check(context.install_indices.includes(index), "v2_install_index");
  const rows = await readJournal(root, context); baseline(rows.at(-1)?.state, true);
  const producer = { ...operations, childProgram: fileURLToPath(new URL("./operator-child.mjs", import.meta.url)) };
  await observeCommand(root, context, "detect", index, producer);
  await observeCommand(root, context, "flash", index, producer);
  const server = (await proof(root, "server-owner.json")).value;
  check(server.origin === `http://127.0.0.1:${server.port}`, "v2_operator_origin");
  const response = await fetch(`${server.origin}/install/review`, { method: "POST", headers: {
    Origin: server.origin, "Content-Type": "application/json" }, body: JSON.stringify({ index }) });
  check(response.ok, "v2_install_review_failed"); return response.json();
}

export function installDirectory(root, index) { return resolve(root, `install-${index}`); }
