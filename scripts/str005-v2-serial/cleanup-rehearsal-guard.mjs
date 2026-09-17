// Test helpers cannot be pointed at a real qualification tree, even if invoked
// manually or from an inherited test environment.
import { readFile, realpath } from "node:fs/promises";
import { resolve, sep } from "node:path";
import { tmpdir } from "node:os";
import { check, object, sha256 } from "./values.mjs";
export async function syntheticRoot(root, maybeContext) {
  const temporary = await realpath(tmpdir());
  check(typeof root === "string" && root === resolve(root) && root.startsWith(`${temporary}${sep}`) &&
    await realpath(root) === root, "v2_rehearsal_root_rejected");
  const stored = JSON.parse(await readFile(resolve(root, "context.json")));
  const context = stored.context;
  check(context?.schema === "str005-v2-serial-context-v3" && context.scope === "channel" &&
    context.firmware_commit === "a".repeat(40) && context.gate_commit === "b".repeat(40) &&
    context.before_source?.firmware_commit === "f".repeat(40) && context.before_source?.app_elf_sha256 === "f".repeat(64) &&
    context.firmware_root.startsWith(`${temporary}${sep}`) && context.gate_root.startsWith(`${temporary}${sep}`) &&
    stored.sha256 === sha256(JSON.stringify(context)) && (!maybeContext || sha256(JSON.stringify(maybeContext)) === stored.sha256),
  "v2_rehearsal_context_rejected");
  const marker = JSON.parse(await readFile(resolve(root, "synthetic-rehearsal.json")));
  object(marker, ["schema", "source", "contextSha256", "root", "hardwareEffects"]);
  check(marker.schema === "str005-v2-synthetic-rehearsal-v1" && marker.source === "test-only" &&
    marker.contextSha256 === stored.sha256 && marker.root === root && marker.hardwareEffects === false,
  "v2_rehearsal_marker_rejected");
  return context;
}
