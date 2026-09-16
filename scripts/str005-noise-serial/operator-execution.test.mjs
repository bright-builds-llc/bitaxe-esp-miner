import assert from "node:assert/strict";
import { readFile, writeFile } from "node:fs/promises";
import { resolve } from "node:path";
import test from "node:test";
import { claimedFixture } from "./completed-fixture.mjs";
import { admitExecution, quoteJustArgument } from "./operator-execution.mjs";
import { digest } from "./files.mjs";

test("child execution admits only exact parent-bound command data", async (t) => {
  const f = await claimedFixture(t);
  const admitted = await admitExecution(f.root, "flash", 0, f.permit, f.operations);
  assert.equal(admitted.argv[0], "flash-monitor");
  assert.equal(admitted.argv.includes("--factory-reset"), false);
});
for (const [name, mutation, error] of [
  ["extra factory flag", (v) => v.argv.push("--factory-reset"), "noise_execute_argv_changed"],
  ["duplicate manifest", (v) => v.argv.push("--manifest", v.argv[7]), "noise_execute_argv_changed"],
  ["port not from detector", (v) => { v.detector.port = "/dev/cu.changed"; v.argv[4] = v.detector.port; }, "noise_execute_detector_fields"],
]) test(`child rejects ${name} even under an independently pinned malformed claim`, async (t) => {
  const f = await claimedFixture(t), path = resolve(f.root, "install-0.claim.json");
  const claim = JSON.parse(await readFile(path)); mutation(claim);
  const bytes = `${JSON.stringify(claim)}\n`; await writeFile(path, bytes);
  await assert.rejects(admitExecution(f.root, "flash", 0, { ...f.permit, claimSha256: digest(bytes) }, f.operations), { code: error });
});
test("self-rehashing changed context cannot replace the parent's original context digest", async (t) => {
  const f = await claimedFixture(t), path = resolve(f.root, "context.json");
  const record = JSON.parse(await readFile(path)); record.context.manifest = "/changed";
  record.sha256 = digest(JSON.stringify(record.context)); await writeFile(path, JSON.stringify(record));
  await assert.rejects(admitExecution(f.root, "flash", 0, f.permit, f.operations), { code: "noise_execute_context_changed" });
});
test("claim modification after server permit cannot be authorized by rereading its new hash", async (t) => {
  const f = await claimedFixture(t), path = resolve(f.root, "install-0.claim.json");
  const claim = JSON.parse(await readFile(path)); claim.argv.push("--wifi-credentials", "/never-read");
  await writeFile(path, JSON.stringify(claim));
  await assert.rejects(admitExecution(f.root, "flash", 0, f.permit, f.operations), { code: "noise_execute_claim_changed" });
});
test("just interpolation receives shell-safe single arguments", () => {
  assert.equal(quoteJustArgument("a b;$x`id`'tail"), "'a b;$x`id`'\\''tail'");
});
