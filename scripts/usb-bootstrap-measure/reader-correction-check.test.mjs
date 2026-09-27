import test from "node:test";
import assert from "node:assert/strict";
import { writeFile } from "node:fs/promises";
import { resolve } from "node:path";
import { fixture } from "./test-fixture.mjs";
import { runReaderCheck, readerCommands, testCounts } from "./reader-correction-check.mjs";
test("fixed proof binds all three exact commands and retained executable", async t => {
  const f = await fixture(t), run = f.operations.runReaderRegression, calls = [];
  f.operations.runReaderRegression = (program, args, options) => { calls.push(args); assert(options.timeout > 0 && options.timeout <= 120000); assert(options.maxBuffer > 0 && options.maxBuffer <= 65536); return run(program, args); };
  const value = await runReaderCheck(f.context, f.operations); assert.deepEqual(calls, readerCommands(f.context)); assert.equal(value.receipt.runs.length, 3);
});
test("exactly exhausted shared output cannot launch the next test", async t => {
  const f = await fixture(t), run = f.operations.runReaderRegression; let calls = 0;
  f.operations.runReaderRegression = (program, args) => { calls++; const result = run(program, args); result.stderr = Buffer.alloc(65536 - result.stdout.length, 32); return result; };
  await assert.rejects(runReaderCheck(f.context, f.operations)); assert.equal(calls, 1);
});
test("changed executable after a successful run rejects the receipt", async t => {
  const f = await fixture(t), run = f.operations.runReaderRegression;
  f.operations.runReaderRegression = async (program, args) => { const result = run(program, args); if (args[0] === "--test") await writeFile(resolve(f.context.firmwareRoot, "bazel-bin/tools/device-session/tests"), "changed"); return result; };
  await assert.rejects(runReaderCheck(f.context, f.operations), { code: "bootstrap_reader_artifact_changed" });
});
test("missing, duplicate and cancelled test summaries reject", () => {
  assert.throws(() => testCounts("", false)); assert.throws(() => testCounts("test result: ok. 1 passed; 0 failed; 0 ignored;\ntest result: ok. 1 passed; 0 failed; 0 ignored;", false));
  assert.throws(() => testCounts("# pass 1\n# fail 0\n# cancelled 1\n# skipped 0\n# todo 0\n", true));
});

test("v3 proof keeps its original two-file command while v4 adds only frozen restoration tests", () => {
  const old = readerCommands({ schema: "usb-bootstrap-measure-context-v3" }), current = readerCommands({ schema: "usb-bootstrap-measure-context-v4" });
  assert.deepEqual(current.slice(0, 2), old.slice(0, 2));
  assert.deepEqual(current[2], [...old[2], "scripts/usb-bootstrap-measure/restored-accounting.test.mjs", "scripts/usb-bootstrap-measure/restored-predecessor.test.mjs"]);
  assert.equal(old[2].length, 4);
});
