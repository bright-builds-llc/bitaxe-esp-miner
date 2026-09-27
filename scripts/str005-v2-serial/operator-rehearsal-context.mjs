import { readFile, realpath, lstat } from "node:fs/promises";
import { resolve } from "node:path";
import { check } from "./values.mjs";
/** Test-only substituted admission; never imported by a production entrypoint. */
export async function loadOperatorContext(root) {
  const temporary = await realpath("/tmp");
  check(root.startsWith(`${temporary}/v2op-rehearsal-`) && await realpath(root) === root, "v2_test_root");
  const stat = await lstat(root); check((stat.mode & 0o777) === 0o700, "v2_test_root");
  const marker = JSON.parse(await readFile(resolve(root, "synthetic-only.json"), "utf8"));
  check(marker.schema === "operator-loss-software-only-v1", "v2_test_root");
  return JSON.parse(await readFile(resolve(root, "context.json"), "utf8")).context;
}
export const loadEffectContext = loadOperatorContext;

export async function verifyCleanupInputs(context) {
  return loadOperatorContext(resolve(context.firmware_root, "scratch/str005-v2-serial", `${context.scope}-${String(context.hostOrdinal).padStart(3, "0")}`));
}
