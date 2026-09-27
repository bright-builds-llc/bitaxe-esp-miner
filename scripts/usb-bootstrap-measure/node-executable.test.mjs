import test from "node:test";
import assert from "node:assert/strict";
import { mkdtemp, writeFile, chmod, rm, realpath } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { actualNodePath } from "./node-executable.mjs";
test("Bazel runtime binding resolves the actual native executable rather than its shell wrapper", async t => {
  // Arrange: the wrapper is never executed or treated as the runtime identity.
  const root = await mkdtemp(join(tmpdir(), "bootstrap-node-")); t.after(() => rm(root, { recursive: true, force: true }));
  const wrapper = join(root, "node-wrapper"); await writeFile(wrapper, '#!/bin/sh\nexec "$JS_BINARY__NODE_BINARY" "$@"\n'); await chmod(wrapper, 0o700);
  const native = await actualNodePath();
  // Act / Assert.
  assert.equal(await actualNodePath({ executable: wrapper, environment: { JS_BINARY__NODE_BINARY: native } }), await realpath(native));
  await assert.rejects(actualNodePath({ executable: wrapper, environment: {} }), { code: "bootstrap_node_executable" });
});
test("relative runtime overrides reject without falling back to a different executable", async () => {
  await assert.rejects(actualNodePath({ environment: { JS_BINARY__NODE_BINARY: "node" } }), { code: "bootstrap_node_executable" });
});
