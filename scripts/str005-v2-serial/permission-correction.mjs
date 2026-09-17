import { spawnSync } from "node:child_process";
import { performance } from "node:perf_hooks";
import { readFile } from "node:fs/promises";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { BUNDLE, cleanPushed, fileDigest } from "../fixed-usb-qualification/contract.mjs";
import { canonical } from "../str005-noise-serial/files.mjs";
import { check, digest, object, uint, sha256, PERMISSION_AMENDMENT_PATH, PERMISSION_AMENDMENT_SHA256 } from "./values.mjs";

export const CORRECTION_CHECKER = "scripts/str005-v2-serial/permission-correction.mjs";
export const CORRECTION_COMMAND = Object.freeze(["bun", "test", "./web/worker-qualification-gesture.test.ts", "./web/worker-serial-admission.test.ts"]);
const LIMIT_MS = 30000, OUTPUT_BYTES = 65536;

async function publication(source, operations) {
  for (const [root, commit] of [[source.firmware_root, source.firmware_commit], [source.gate_root, source.gate_commit]])
    (operations.cleanPushed ?? cleanPushed)(root, commit);
  const pins = [...(await readFile(resolve(source.firmware_root, "MODULE.bazel"), "utf8"))
    .matchAll(/strip_prefix\s*=\s*"bitaxe-turnstile-system-([a-f0-9]{40})"/gu)];
  check(pins.length === 1 && pins[0][1] === source.gate_commit, "v2_gate_pin_mismatch");
  const bundle = await readFile(resolve(source.gate_root, BUNDLE));
  check(bundle.includes(source.gate_commit) &&
    sha256(bundle) === source.gate_bundle_sha256 &&
    await fileDigest(resolve(source.firmware_root, PERMISSION_AMENDMENT_PATH)) === PERMISSION_AMENDMENT_SHA256,
  "v2_permission_correction_source");
  const checker = await fileDigest(resolve(source.firmware_root, CORRECTION_CHECKER));
  check(checker === await fileDigest(fileURLToPath(import.meta.url)), "v2_permission_checker_changed");
  return checker;
}

/** Fixed software-only command; no caller verdict, test selection or raw output is accepted. */
export async function checkPermissionCorrection(source, operations = {}) {
  const checkerSha256 = await publication(source, operations);
  const clock = operations.correctionNow ?? (() => Math.floor(performance.now())), start = clock(); uint(start);
  let result;
  try {
    result = (operations.spawnSync ?? spawnSync)(CORRECTION_COMMAND[0], CORRECTION_COMMAND.slice(1), {
      cwd: source.gate_root, timeout: LIMIT_MS, killSignal: "SIGKILL", maxBuffer: OUTPUT_BYTES,
      stdio: ["ignore", "pipe", "pipe"], encoding: null,
      env: { PATH: process.env.PATH ?? "/usr/bin:/bin", LANG: "C", LC_ALL: "C" },
    });
    const end = clock(); uint(end);
    const bytes = [result.stdout, result.stderr].reduce((sum, value) => sum + (value?.length ?? 0), 0);
    check(!result.error && result.status === 0 && result.signal === null && bytes <= OUTPUT_BYTES &&
      end >= start && end - start <= LIMIT_MS, "v2_permission_correction_failed");
    check(await publication(source, operations) === checkerSha256, "v2_permission_checker_changed");
    return { schema: "str005-v2-permission-correction-v1", firmwareCommit: source.firmware_commit,
      gateCommit: source.gate_commit, gateBundleSha256: source.gate_bundle_sha256, checkerSha256,
      command: [...CORRECTION_COMMAND], exitCode: 0, signal: null, elapsedMs: end - start };
  } catch (error) {
    if (typeof error?.code === "string" && /^v2_[a-z_]+$/u.test(error.code)) throw error;
    check(false, "v2_permission_correction_failed");
  } finally {
    for (const value of [result?.stdout, result?.stderr]) if (Buffer.isBuffer(value)) value.fill(0);
  }
}

/** Historical snapshots validate the sealed observation, never rerun a current command. */
export function validatePermissionCorrection(value, context) {
  object(value, ["schema", "firmwareCommit", "gateCommit", "gateBundleSha256", "checkerSha256", "command", "exitCode", "signal", "elapsedMs"]);
  digest(value.checkerSha256); digest(value.gateBundleSha256); uint(value.elapsedMs, LIMIT_MS);
  check(typeof value.firmwareCommit === "string" && /^[a-f0-9]{40}$/u.test(value.firmwareCommit) &&
    typeof value.gateCommit === "string" && /^[a-f0-9]{40}$/u.test(value.gateCommit), "v2_permission_correction_binding");
  check(value.schema === "str005-v2-permission-correction-v1" && value.firmwareCommit === context.firmware_commit &&
    value.gateCommit === context.gate_commit && value.gateBundleSha256 === context.gate_bundle_sha256 &&
    value.checkerSha256 === context.evaluator.find(item => item.path === CORRECTION_CHECKER)?.sha256 &&
    canonical(value.command) === canonical(CORRECTION_COMMAND) && value.exitCode === 0 && value.signal === null,
  "v2_permission_correction_binding");
  return value;
}
