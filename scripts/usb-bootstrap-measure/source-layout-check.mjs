import { spawnSync } from "node:child_process";
import { nodeRuntimeEnvironment } from "../str005-noise-serial/node-runtime.mjs";
import { canonical } from "../str005-noise-serial/files.mjs";
import { verifyCurrent } from "./sources.mjs";
import { check, object, sha256 } from "./values.mjs";
export const COMMAND = Object.freeze(["--test", "--test-concurrency=1", "scripts/usb-bootstrap-measure/source-snapshot.test.mjs"]);
export function validateLayoutCheck(value, context) {
  object(value, ["schema", "command", "nodeSha256", "sourceInventorySha256", "exitCode", "signal"]);
  check(value.schema === "usb-bootstrap-source-layout-check-v1" && value.command === "source-snapshot-regression" &&
    value.nodeSha256 === context.hostTools.node.sha256 && value.sourceInventorySha256 === sha256(canonical(context.sourceInventory)) &&
    value.exitCode === 0 && value.signal === null, "bootstrap_layout_check"); return value;
}
export async function runLayoutCheck(context, operations = {}) {
  await verifyCurrent(context, operations);
  const result = (operations.runSourceLayoutCheck ?? spawnSync)(context.hostTools.node.path, [...COMMAND], {
    cwd: context.firmwareRoot, encoding: "buffer", timeout: 60000, maxBuffer: 65536, stdio: ["ignore", "pipe", "pipe"],
    env: { PATH: process.env.PATH ?? "/usr/bin:/bin", LANG: "C", LC_ALL: "C", ...nodeRuntimeEnvironment() },
  });
  try {
    check(!result.error && result.status === 0 && result.signal === null && (result.stdout?.length ?? 0) + (result.stderr?.length ?? 0) <= 65536, "bootstrap_layout_check_failed");
    await verifyCurrent(context, operations);
    return validateLayoutCheck({ schema: "usb-bootstrap-source-layout-check-v1", command: "source-snapshot-regression", nodeSha256: context.hostTools.node.sha256,
      sourceInventorySha256: sha256(canonical(context.sourceInventory)), exitCode: 0, signal: null }, context);
  } finally { for (const bytes of [result.stdout, result.stderr]) if (Buffer.isBuffer(bytes)) bytes.fill(0); }
}
