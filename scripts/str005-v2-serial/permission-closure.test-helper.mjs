// Real protected v1 snapshots with explicit synthetic publication/predecessor adapters.
import { readFile, writeFile } from "node:fs/promises";
import { resolve } from "node:path";
import { legacyContextFixture } from "./context-fixtures.mjs";
import { loadContext } from "./context.mjs";
import { state } from "../str005-noise-serial/test-fixture.mjs";
import { createJournal } from "./journal.mjs";
import { proof, writeNew } from "../str005-noise-serial/files.mjs";
import { BUNDLE } from "../fixed-usb-qualification/contract.mjs";
import { sha256 } from "./values.mjs";

export async function permissionFixture(t) {
  const f = await legacyContextFixture(t), { root, context } = f, contextSha256 = sha256(JSON.stringify(context));
  const journal = await createJournal(root, context), statuses = ["configured", "configured", "configured", "failed", "closing", "closed"];
  for (const [index, status] of statuses.entries()) {
    const value = state(context, "before"); delete value.preservation;
    Object.assign(value, { status, connected: false, running: false, heartbeatSuppressed: false, renewalsConfirmed: 0,
      deviceRestorationConfirmed: false, deviceBaselineConfirmed: false, deviceLeaseInactive: false, serialOwnershipReleased: index !== 1 });
    if (index >= 2) Object.assign(value, { serialFailureCategory: "operation_failed", admissionFailureStage: "permission" });
    if (index >= 3) value.failure = "connect_failed";
    await journal.state("before", value, (index + 1) * 10);
  }
  const owner = { pid: 22111, pgid: 22111, startedAt: "synthetic-permission-supervisor" }, origin = "http://127.0.0.1:44445";
  await writeNew(resolve(root, "server.claim.json"), { schema: "str005-v2-server-claim-v1", contextSha256 });
  await writeNew(resolve(root, "server-owner.json"), { schema: "str005-v2-server-owner-v1", contextSha256, owner, origin, port: 44445, atHostMs: 1 });
  await writeNew(resolve(root, "failure.json"), { schema: "str005-v2-first-failure-v1", contextSha256, code: "v2_browser_failed", atHostMs: 45,
    deviceCause: null, sourceSequence: 4 });
  const operator = `${root}.operator`;
  await f.put(resolve(operator, "parent.mjs"), "// Synthetic parent; no device effect. Browser witness is independently parent-observed.\n");
  await f.put(resolve(operator, "initial-detection.log"), `port: /dev/cu.synthetic\nusb_profile: serial_jtag_runtime\nphysical_identity_sha256: ${"a".repeat(64)}\n`);
  await writeNew(resolve(operator, "browser.json"), { schema: "noise-serial-browser-closure-v2", source: "parent-observed", contextSha256,
    closed: true, lastSequence: 6, lastStateSha256: sha256(JSON.stringify(journal.lastState())), observedAtUnixMs: 900 });
  await writeNew(resolve(operator, "supervisor-root.json"), owner);
  await writeNew(resolve(operator, "supervisor-exit.json"), { schema: "noise-serial-process-exit-v2", source: "parent-observed", contextSha256,
    owner, code: 0, observedAtUnixMs: 800, clock: "node-hrtime-ms-v1", stopRequestedAtMs: 100, exitedAtMs: 113 });
  await writeNew(resolve(operator, "no-admission-resources.json"), { schema: "str005-v2-no-admission-resources-v1", source: "parent-observed",
    contextSha256, observedAtUnixMs: 1000, observations: [
      { program: "lsof", args: ["-nP", "-iTCP:44445", "-sTCP:LISTEN", "-t"], exitCode: 1, stdoutBytes: 0, stderrBytes: 0 },
      { program: "lsof", args: ["-Fpn", "--", "/dev/cu.synthetic", "/dev/tty.synthetic"], exitCode: 1, stdoutBytes: 0, stderrBytes: 0 }] });
  await f.put(`${root}.serve.stdout.log`, `qualification_url=${origin}/\n{"supervisor":"closed","hardware_qualified":false}\n`);
  await f.put(`${root}.serve.stderr.log`, "");
  // Current published correction differs from the failed pair. Historical snapshot bytes stay untouched.
  await writeFile(resolve(context.firmware_root, "MODULE.bazel"), `strip_prefix = "bitaxe-turnstile-system-${"e".repeat(40)}"\n`);
  await writeFile(resolve(context.gate_root, BUNDLE), `${"e".repeat(40)} explicit synthetic corrected bundle`);
  const kernelCalls = [], operations = { ...f.operations,
    inspectFailedContext: path => loadContext(path, { historical: true, operations: f.operations }),
    git: path => path === context.firmware_root ? "f".repeat(40) : "e".repeat(40),
    readPublishedSource: (repo, _commit, path) => readFile(resolve(repo, path)),
    spawnSync: (program, args) => { kernelCalls.push([program, ...args]); return { status: 0, signal: null, stdout: Buffer.alloc(0), stderr: Buffer.alloc(0) }; },
    correctionNow: () => 100, now: () => 2000,
    processSnapshot: async () => { kernelCalls.push(["ps"]); return []; },
    execFileSync: (program, args) => { kernelCalls.push([program, ...args]); throw Object.assign(new Error("synthetic absence"), { status: 1, signal: null, stdout: "", stderr: "" }); } };
  return { ...f, operator, owner, operations, kernelCalls, contextSha256,
    read: async name => (await proof(root, name)).value };
}
