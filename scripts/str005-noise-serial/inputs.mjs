import { readdir } from "node:fs/promises";
import { resolve } from "node:path";
import { fileDigest, protectedPath } from "../fixed-usb-qualification/contract.mjs";
import { canonical, digest } from "./files.mjs";
async function maybeHash(path) {
  try { await protectedPath(path); return await fileDigest(path); }
  catch (error) { if (error.code === "ENOENT") return null; throw error; }
}
async function hashes(root, names) {
  const result = [];
  for (const name of names) { const sha256 = await maybeHash(resolve(root, name)); if (sha256 !== null) result.push({ path: name, sha256 }); }
  return result.length ? digest(canonical(result)) : null;
}
/** Bind present input bytes even when they are incomplete or cannot pass a parser. */
export async function collectInputs(root, cleanupSnapshot = false) {
  const names = (await readdir(root)).sort();
  return {
    deviceJournal: await hashes(root, names.filter((name) => /^(?:state|noise)-[0-9]{4}\.json$/u.test(name))),
    fixtureReady: await maybeHash(resolve(root, "fixture-run/ready.json")),
    fixtureTerminal: await maybeHash(resolve(root, "fixture-run/terminal.json")),
    preservation: await hashes(root, [...names.filter((name) => /^state-[0-9]{4}\.json$/u.test(name)), "restoration.json"]),
    accounting: await hashes(root, ["accounting-before-install.json", "accounting-before.json", "accounting-after.json"]),
    cleanup: await hashes(cleanupSnapshot ? resolve(root, "final-inputs/cleanup") : `${root}.cleanup`, ["browser.json", "fixture.json", "supervisor.json", "resources.json", "receipt.json"]),
    recovery: null,
  };
}
export function failureOutcome(cause, code) {
  if (/authority|credentials|task_inactive|source_dirty|source_mismatch|permission/u.test(code) ||
    ["authority_lost"].includes(cause.category) || ["cancel_requested", "session_replaced"].includes(cause.detail)) return "stop_authority_boundary";
  if (["preparation", "connect", "write", "read", "authentication", "proof", "clock_invalid", "cleanup"].includes(cause.category))
    return "stop_hardware_blocker";
  return "stop_impossible_contract";
}
