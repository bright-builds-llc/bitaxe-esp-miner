import test from "node:test";
import assert from "node:assert/strict";
import { fixture } from "./test-fixture.mjs";
import { proof } from "../str005-noise-serial/files.mjs";
import { validateWriter } from "./native-writer.mjs";
test("writer receipt cannot hide invalid frames, sums, roles or omitted source dependencies", async t => {
  const f = await fixture(t), stored = (await proof(f.root, "snapshot/native-writer.json")).value;
  assert.equal(validateWriter(stored, f.context), stored);
  for (const change of [v => { v.selectedFrameSumBytes = -1; }, v => { v.frames[0].entryBytes = Number.NaN; },
    v => { v.selectedFrameSumBytes -= 1; }, v => { v.sources.pop(); }, v => { v.roles.emit = "standalone_measured"; },
    v => { v.sources[0].secret = "private"; }, v => { v.frames[0].secret = "private"; }]) {
    const value = structuredClone(stored); change(value); assert.throws(() => validateWriter(value, f.context));
  }
});
test("writer validator accepts distinct native addresses with equal demangled names and rejects address aliases", async t => {
  // Arrange
  const f = await fixture(t), value = (await proof(f.root, "snapshot/native-writer.json")).value;
  const symbol = "bitaxe_firmware::usb_runtime::write_measured_if";
  value.frames.push({address:8192,symbol,entryBytes:128},{address:12288,symbol,entryBytes:256});
  value.selectedFrameSumBytes += 384; value.remainingBytes -= 384; value.roles.write = "standalone_measured";
  // Act / Assert
  assert.equal(validateWriter(value, f.context), value);
  const alias = structuredClone(value); alias.frames[2].address = alias.frames[1].address;
  assert.throws(() => validateWriter(alias, f.context), /bootstrap_writer_audit/u);
  const missing = structuredClone(value); delete missing.frames[1].address;
  assert.throws(() => validateWriter(missing, f.context));
});
