import { spawnSync } from "node:child_process";
import { performance } from "node:perf_hooks";
import { readFile } from "node:fs/promises";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { BUNDLE, cleanPushed, fileDigest } from "../fixed-usb-qualification/contract.mjs";
import { canonical } from "../str005-noise-serial/files.mjs";
import { check, digest, object, uint, sha256, CLEANUP_AMENDMENT_PATH, CLEANUP_AMENDMENT_SHA256 } from "./values.mjs";

export const HOST_CORRECTION_CHECKER = "scripts/str005-v2-serial/host-correction.mjs";
export const HOST_CORRECTION_COMMAND = Object.freeze(["node", "--test",
  "scripts/str005-v2-serial/host-resources.test.mjs",
  "scripts/str005-v2-serial/continuity-baseline.test.mjs",
  "scripts/str005-v2-serial/cleanup-rehearsal.test.mjs"]);
const LIMIT_MS = 180000, OUTPUT_BYTES = 65536;
async function publication(context, operations) {
  for (const [root, commit] of [[context.firmware_root, context.firmware_commit], [context.gate_root, context.gate_commit]])
    (operations.cleanPushed ?? cleanPushed)(root, commit);
  const pins = [...(await readFile(resolve(context.firmware_root, "MODULE.bazel"), "utf8"))
    .matchAll(/strip_prefix\s*=\s*"bitaxe-turnstile-system-([a-f0-9]{40})"/gu)];
  check(pins.length === 1 && pins[0][1] === context.gate_commit, "v2_gate_pin_mismatch");
  const bundle = await readFile(resolve(context.gate_root, BUNDLE));
  check(bundle.includes(context.gate_commit) && sha256(bundle) === context.gate_bundle_sha256 &&
    await fileDigest(resolve(context.firmware_root, CLEANUP_AMENDMENT_PATH)) === CLEANUP_AMENDMENT_SHA256,
  "v2_host_correction_source");
  const checker = await fileDigest(resolve(context.firmware_root, HOST_CORRECTION_CHECKER));
  check(checker === await fileDigest(fileURLToPath(import.meta.url)), "v2_host_checker_changed");
  return checker;
}
/** Fixed host-only regression command. Child output is bounded and discarded, never evidence. */
export async function checkHostCorrection(context, operations = {}) {
  check((operations.hostPlatform ?? process.platform) === "darwin", "v2_host_platform_unsupported");
  const checkerSha256 = await publication(context, operations);
  const clock = operations.hostCorrectionNow ?? (() => Math.floor(performance.now())), start = clock(); uint(start);
  let result;
  try {
    result = (operations.spawnSync ?? spawnSync)(HOST_CORRECTION_COMMAND[0], HOST_CORRECTION_COMMAND.slice(1), {
      cwd: context.firmware_root, timeout: LIMIT_MS, killSignal: "SIGKILL", maxBuffer: OUTPUT_BYTES,
      stdio: ["ignore", "pipe", "pipe"], encoding: null,
      env: { PATH: process.env.PATH ?? "/usr/bin:/bin", LANG: "C", LC_ALL: "C" },
    });
    const end = clock(); uint(end);
    const bytes = [result.stdout, result.stderr].reduce((sum, value) => sum + (value?.length ?? 0), 0);
    check(!result.error && result.status === 0 && result.signal === null && bytes <= OUTPUT_BYTES &&
      end >= start && end - start <= LIMIT_MS, "v2_host_correction_failed");
    check(await publication(context, operations) === checkerSha256, "v2_host_checker_changed");
    return { schema: "str005-v2-host-correction-v1", firmwareCommit: context.firmware_commit,
      gateCommit: context.gate_commit, gateBundleSha256: context.gate_bundle_sha256, checkerSha256,
      command: [...HOST_CORRECTION_COMMAND], exitCode: 0, signal: null, elapsedMs: end - start };
  } catch (error) {
    if (typeof error?.code === "string" && /^v2_[a-z_]+$/u.test(error.code)) throw error;
    check(false, "v2_host_correction_failed");
  } finally {
    for (const output of [result?.stdout, result?.stderr]) if (Buffer.isBuffer(output)) output.fill(0);
  }
}
/** Historical review checks the sealed source-bound receipt without executing today's tests. */
export function validateHostCorrection(value, context) {
  object(value, ["schema", "firmwareCommit", "gateCommit", "gateBundleSha256", "checkerSha256", "command", "exitCode", "signal", "elapsedMs"]);
  digest(value.checkerSha256); digest(value.gateBundleSha256); uint(value.elapsedMs, LIMIT_MS);
  check(typeof value.firmwareCommit === "string" && /^[a-f0-9]{40}$/u.test(value.firmwareCommit) &&
    typeof value.gateCommit === "string" && /^[a-f0-9]{40}$/u.test(value.gateCommit) &&
    value.schema === "str005-v2-host-correction-v1" && value.firmwareCommit === context.firmware_commit &&
    value.gateCommit === context.gate_commit && value.gateBundleSha256 === context.gate_bundle_sha256 &&
    value.checkerSha256 === context.evaluator.find(item => item.path === HOST_CORRECTION_CHECKER)?.sha256 &&
    canonical(value.command) === canonical(HOST_CORRECTION_COMMAND) && value.exitCode === 0 && value.signal === null,
  "v2_host_correction_binding");
  return value;
}
