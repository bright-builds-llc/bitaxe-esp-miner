import { readFile, lstat } from "node:fs/promises";
import { resolve } from "node:path";
import { canonical, proof, protectedPath } from "../str005-noise-serial/files.mjs";
import { nodeRuntimeEnvironment } from "../str005-noise-serial/node-runtime.mjs";
import { verifyCurrent } from "./sources.mjs";
import { check, object, sha256, uint } from "./values.mjs";
export const READER_COMMANDS = Object.freeze([
  ["--exact", "reader_correction_identity::reports_build_identity", "--nocapture", "--test-threads=1"],
  ["usb::early_capture::tests::", "--test-threads=1"],
  ["--test", "--test-concurrency=1", "scripts/usb-bootstrap-measure/host-timing.test.mjs", "scripts/usb-bootstrap-measure/correction-judge.test.mjs"],
]);
export const READER_RUNS = ["identity", "rust-capture", "node-judges"];
function identity(value, context) {
  object(value, ["schema", "sourceCommit", "sourceDirty"]);
  check(value.schema === "usb-bootstrap-reader-test-identity-v1" && value.sourceCommit === context.package.firmware_commit && value.sourceDirty === false, "bootstrap_reader_identity");
}
export function testCounts(text, node) {
  if (!node) {
    const rows = [...text.matchAll(/^test result: ok\. ([0-9]+) passed; ([0-9]+) failed; ([0-9]+) ignored;/gmu)];
    check(rows.length === 1, "bootstrap_reader_test_summary"); return { passed: Number(rows[0][1]), failed: Number(rows[0][2]), ignored: Number(rows[0][3]) };
  }
  const count = name => { const rows = [...text.matchAll(new RegExp(`^(?:#|ℹ) ${name} ([0-9]+)\\s*$`, "gmu"))]; check(rows.length === 1, "bootstrap_reader_test_summary"); return Number(rows[0][1]); };
  check(count("cancelled") === 0, "bootstrap_reader_test_summary"); return { passed: count("pass"), failed: count("fail"), ignored: count("skipped") + count("todo") };
}
export function validateReaderCheck(value, context) {
  object(value, ["schema", "command", "firmwareCommit", "sourceInventorySha256", "nodeSha256", "testBinarySha256", "identity", "runs"]);
  check(value.schema === "usb-bootstrap-reader-correction-check-v1" && value.command === "reader-correction-regression" && value.firmwareCommit === context.package.firmware_commit &&
    value.sourceInventorySha256 === sha256(canonical(context.sourceInventory)) && value.nodeSha256 === context.hostTools.node.sha256 && /^[a-f0-9]{64}$/u.test(value.testBinarySha256), "bootstrap_reader_check");
  identity(value.identity, context); check(Array.isArray(value.runs) && value.runs.length === 3, "bootstrap_reader_check");
  value.runs.forEach((run, index) => { object(run, ["name", "exitCode", "signal", "passed", "failed", "ignored"]); [run.passed, run.failed, run.ignored].forEach(n => uint(n));
    check(run.name === READER_RUNS[index] && run.exitCode === 0 && run.signal === null && run.passed > 0 && run.failed === 0 && run.ignored === 0 && (index !== 0 || run.passed === 1), "bootstrap_reader_check"); });
  return value;
}
function provenance(value, context) {
  object(value, ["schema", "sourceCommit", "sourceDirty", "writerSha256"]);
  const { writerSha256, ...embedded } = value; identity(embedded, context);
  check(writerSha256 === context.sourceInventory.find(row => row.path === "scripts/usb-bootstrap-measure/test-provenance.mjs")?.sha256, "bootstrap_reader_provenance");
}
async function binary(path, executable) {
  if (!executable) await protectedPath(path);
  const stat = await lstat(path); check(stat.isFile() && !stat.isSymbolicLink() && stat.size > 0 && stat.size <= 64 * 1024 * 1024 && (!executable || (stat.mode & 0o111) !== 0), "bootstrap_reader_binary");
  return readFile(path);
}
export async function runReaderCheck(context, operations = {}) {
  const started = performance.now();
  await verifyCurrent(context, operations);
  const path = resolve(context.firmwareRoot, "bazel-bin/tools/device-session/tests"), provenancePath = resolve(context.firmwareRoot, "bazel-bin/tools/device-session/reader-test-build.json");
  const bytes = await binary(path, true), provenanceBytes = await readFile(provenancePath), built = JSON.parse(provenanceBytes); provenance(built, context);
  const runs = []; let outputBytes = 0, embedded;
  for (const [index, args] of READER_COMMANDS.entries()) {
    const remaining = Math.floor(120000 - (performance.now() - started)); check(remaining > 0, "bootstrap_reader_test_timeout");
    check(outputBytes < 65536, "bootstrap_reader_test_failed");
    const execute = operations.runReaderRegression ?? (await import("./regression-process.mjs")).runRegression;
    const result = await execute(index === 2 ? context.hostTools.node.path : path, [...args], {
      cwd: context.firmwareRoot, nodePath: context.hostTools.node.path, timeout: remaining, maxBuffer: 65536 - outputBytes, stdio: ["ignore", "pipe", "pipe"],
      env: { PATH: process.env.PATH ?? "/usr/bin:/bin", LANG: "C", LC_ALL: "C", ...nodeRuntimeEnvironment() },
    });
    try {
      outputBytes += (result.stdout?.length ?? 0) + (result.stderr?.length ?? 0);
      check(!result.error && result.status === 0 && result.signal === null && outputBytes <= 65536 && performance.now() - started <= 120000, "bootstrap_reader_test_failed");
      const text = String(result.stdout ?? ""), counts = testCounts(text, index === 2);
      if (index === 0) {
        const lines = text.split(/\r?\n/u).filter(line => line.startsWith('{"schema":"usb-bootstrap-reader-test-identity-v1"'));
        check(lines.length === 1, "bootstrap_reader_identity"); embedded = JSON.parse(lines[0]); identity(embedded, context);
      }
      runs.push({ name: READER_RUNS[index], exitCode: result.status, signal: result.signal, ...counts });
    } finally { for (const value of [result.stdout, result.stderr]) if (Buffer.isBuffer(value)) value.fill(0); }
  }
  check(sha256(await binary(path, true)) === sha256(bytes) && sha256(await readFile(provenancePath)) === sha256(provenanceBytes), "bootstrap_reader_artifact_changed");
  await verifyCurrent(context, operations);
  check(performance.now() - started <= 120000, "bootstrap_reader_test_timeout");
  const receipt = validateReaderCheck({ schema: "usb-bootstrap-reader-correction-check-v1", command: "reader-correction-regression", firmwareCommit: context.package.firmware_commit,
    sourceInventorySha256: sha256(canonical(context.sourceInventory)), nodeSha256: context.hostTools.node.sha256, testBinarySha256: sha256(bytes), identity: embedded, runs }, context);
  return { receipt, binary: bytes, provenance: provenanceBytes };
}
export async function verifyReaderSnapshot(root, context) {
  const receipt = (await proof(root, "snapshot/reader-correction-check.json")).value; validateReaderCheck(receipt, context);
  check(sha256(await binary(resolve(root, "snapshot/reader-tests.bin"), false)) === receipt.testBinarySha256, "bootstrap_reader_artifact_changed");
  provenance((await proof(root, "snapshot/reader-test-build.json")).value, context);
}
