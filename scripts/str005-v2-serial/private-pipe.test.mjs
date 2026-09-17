import assert from "node:assert/strict";
import { PassThrough } from "node:stream";
import test from "node:test";
import { readPrivateRecord, writePrivateRecord } from "./private-pipe.mjs";

test("fragmented private input remains memory-only and requires EOF", async () => {
  // Arrange
  const stream = new PassThrough(); let settled = false;
  const result = readPrivateRecord(stream, { deadlineMs: performance.now() + 1000 }).then((value) => { settled = true; return value; });
  // Act
  stream.write('{"private":"synthetic-'); stream.write('value"}\n');
  await Promise.resolve();
  // Assert
  assert.equal(settled, false);
  stream.end(); assert.deepEqual(await result, { private: "synthetic-value" });
});

test("trailing private records are rejected rather than ignored", async () => {
  const stream = new PassThrough(), result = readPrivateRecord(stream, { deadlineMs: performance.now() + 1000 });
  stream.end('{}\n{}\n');
  await assert.rejects(result, { code: "v2_private_pipe_record" });
});

test("oversized private output cannot turn into an unbounded retained buffer", async () => {
  const stream = new PassThrough(), result = readPrivateRecord(stream, { deadlineMs: performance.now() + 1000 });
  stream.end(Buffer.alloc(4097, 65));
  await assert.rejects(result, { code: "v2_private_pipe_bound" });
});

test("invalid UTF8 is not normalized into a different runtime input", async () => {
  const stream = new PassThrough(), result = readPrivateRecord(stream, { deadlineMs: performance.now() + 1000 });
  stream.end(Buffer.from([123, 34, 120, 34, 58, 34, 0xff, 34, 125, 10]));
  await assert.rejects(result, { code: "v2_private_pipe_record" });
});

test("private pipe timeout is absolute despite partial progress", async () => {
  const stream = new PassThrough(), result = readPrivateRecord(stream, { deadlineMs: performance.now() + 15 });
  stream.write('{"x":');
  await assert.rejects(result, { code: "v2_private_pipe_timeout" });
  stream.destroy();
});

test("private input writer produces exactly one closed record", async () => {
  const stream = new PassThrough(), result = readPrivateRecord(stream, { deadlineMs: performance.now() + 1000 });
  await writePrivateRecord(stream, { schema: "test", value: 4 });
  assert.deepEqual(await result, { schema: "test", value: 4 });
});
