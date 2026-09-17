import { lstat, readFile, readdir } from "node:fs/promises";
import { resolve } from "node:path";
import { privateRoot, protectedPath, retain } from "../str005-noise-serial/files.mjs";
import { check, sha256 } from "./values.mjs";

const GROUPS = {
  stateJournal: /^state-[0-9]{4}\.json$/u,
  deviceJournal: /^device-[0-9]{4}\.json$/u,
  continuity: /^(?:install-[0-4](?:\..+)?|cycle-[1-4](?:\..+)?)$/u,
  accounting: /^(?:accounting-(?:before-install|before|after)|cooling|budget-review-[0-9]{4})\.json$/u,
  selection: /^(?:connection|job-receipt|selected-(?:dispatch|nonce|submission|device-ack|fixture-ack))\.json$/u,
  fault: /^fault(?:\.claim|-confirmed)?\.json$/u,
  restoration: /^(?:restoration|protocol-complete)\.json$/u,
  observer: /^(?:observer-start\.claim|observer-stop|cadence-observer-result)\.json$|^cadence-observer\.jsonl$/u,
  issuance: /^(?:server\.claim|server-owner|start\.claim|share-start-observed|issuance\.claim|issued|consumed|share-network-[0-9]{4}|signer-[0-9]{2}\.exit)\.json$/u,
};
async function maybeDirectory(path) {
  try { await privateRoot(path); return (await readdir(path)).sort(); }
  catch (error) { if (error.code === "ENOENT") return null; throw error; }
}
async function fileEntries(root, relative) {
  const path = resolve(root, relative), stat = await lstat(path);
  if (stat.isDirectory()) {
    await privateRoot(path); const result = [];
    for (const name of (await readdir(path)).sort()) result.push(...await fileEntries(root, `${relative}/${name}`));
    return result;
  }
  await protectedPath(path); const bytes = await readFile(path);
  return [{ path: relative, sha256: sha256(bytes), length: bytes.length }];
}
/** Preserve every available named input, including malformed/partial bytes on an unverified result. */
export async function collectInputs(root, { cleanupSnapshot = false } = {}) {
  await privateRoot(root); const names = (await readdir(root)).sort(), inputs = {};
  for (const [key, pattern] of Object.entries(GROUPS)) {
    const files = [];
    for (const name of names.filter((name) => pattern.test(name))) files.push(...await fileEntries(root, name));
    inputs[key] = files.length ? files : null;
  }
  const fixture = [];
  for (const name of names.filter((name) => /^fixture-(?:run|start\.claim\.json|owner\.json|ready\.json|exit\.json|reap\.json|cleanup-failure\.json)$/u.test(name)))
    fixture.push(...await fileEntries(root, name));
  inputs.fixture = fixture.length ? fixture : null;
  inputs.native = names.includes("native") ? await fileEntries(root, "native") : null;
  const cleanupRoot = cleanupSnapshot ? resolve(root, "final-inputs/cleanup") : `${root}.cleanup`;
  const maybeCleanupNames = await maybeDirectory(cleanupRoot);
  inputs.cleanup = maybeCleanupNames === null ? null : (await Promise.all(maybeCleanupNames.map((name) => fileEntries(cleanupRoot, name)))).flat();
  return inputs;
}
/** Snapshot only fixed safe parent witnesses, once, before classification. */
export async function snapshotCleanup(root, cleanupPath) {
  check(resolve(cleanupPath) === resolve(`${root}.cleanup`, "receipt.json"), "v2_cleanup_path");
  const names = await maybeDirectory(`${root}.cleanup`); if (names === null) return;
  check(names.every((name) => ["browser.json", "supervisor.json", "fixture.json", "resources.json", "receipt.json"].includes(name)), "v2_cleanup_input_set");
  for (const name of names) { const path = resolve(`${root}.cleanup`, name); await protectedPath(path);
    await retain(resolve(root, "final-inputs/cleanup", name), await readFile(path)); }
}
