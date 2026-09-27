import { lstat, readFile } from "node:fs/promises";
import { resolve } from "node:path";
import { protectedPath, writeNew, proof } from "../str005-noise-serial/files.mjs";
import { contextHash } from "./context.mjs";
import { check, object, schema, sha256, uint } from "./values.mjs";
const NAMES = ["flash-command-evidence.json", "flash-monitor.log", "bootstrap-host-timing-v1.json"];
/** Freeze producer bytes at the actual observed close; nonce lineage remains source/test-backed. */
export function observeCaptureExit(child, root, context) {
  return new Promise((done, reject) => child.once("close", (code, signal) => {
    const observedAtUnixMs = Date.now();
    (async () => {
      const artifacts = [], missing = [];
      for (const name of NAMES) {
        const path = `install-0/${name}`;
        try { await protectedPath(resolve(root, path)); const stat = await lstat(resolve(root, path)); check(stat.size <= 16 * 1024 * 1024, "bootstrap_capture_bound");
          const bytes = await readFile(resolve(root, path)); artifacts.push({ path, sha256: sha256(bytes), length: bytes.length });
        } catch (error) { if (error.code !== "ENOENT") throw error; missing.push(path); }
      }
      await writeNew(resolve(root, "install-0.capture-observation.json"), { schema: schema("capture-exit"), contextSha256: contextHash(context), childPid: child.pid,
        code, signal, observedAtUnixMs, artifacts, missing });
    })().then(done, reject);
  }));
}
export async function verifyCaptureExit(root, context, { requireComplete = true } = {}) {
  const receipt = (await proof(root, "install-0.capture-observation.json")).value;
  object(receipt, ["schema", "contextSha256", "childPid", "code", "signal", "observedAtUnixMs", "artifacts", "missing"]);
  uint(receipt.observedAtUnixMs);
  const owner = (await proof(root, "install-0.host-root.json")).value, exit = (await proof(root, "install-0.exit.json")).value;
  check(receipt.schema === schema("capture-exit") && receipt.contextSha256 === contextHash(context) && receipt.childPid === owner.pid && receipt.code === exit.code &&
    receipt.signal === null && Array.isArray(receipt.missing) && Array.isArray(receipt.artifacts) && receipt.missing.length + receipt.artifacts.length === 3 &&
    new Set([...receipt.missing, ...receipt.artifacts.map(row => row.path)]).size === 3 &&
    [...receipt.missing, ...receipt.artifacts.map(row => row.path)].every(path => NAMES.some(name => path === `install-0/${name}`)) && (!requireComplete || receipt.missing.length === 0), "bootstrap_capture_exit");
  for (const row of receipt.artifacts) { object(row, ["path", "sha256", "length"]);
    const bytes = await readFile(resolve(root, row.path)); check(bytes.length === row.length && sha256(bytes) === row.sha256, "bootstrap_capture_substituted"); }
  return receipt;
}

export function observeDetectorExit(child, root, context) {
  return new Promise((done, reject) => child.once("close", (code, signal) => {
    writeNew(resolve(root, "install-0.detect.exit.json"), { schema: schema("detector-exit"), contextSha256: contextHash(context), pid: child.pid, code, signal }).then(done, reject);
  }));
}
