import test from "node:test";
import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { once } from "node:events";
import { mkdtemp, open, readFile, rm, realpath } from "node:fs/promises";
import { tmpdir } from "node:os";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
test("public main serve rejects unmanaged parent before absent authority can be read", { timeout: 120000 }, async t => {
  const root = await mkdtemp(resolve(await realpath(tmpdir()), "v2op-main-"));
  t.after(() => rm(root, { recursive: true, force: true }));
  const output = await open(resolve(root, "stdout"), "wx", 0o600), errors = await open(resolve(root, "stderr"), "wx", 0o600);
  const child = spawn(process.execPath, [fileURLToPath(new URL("./operator-managed-rejection.mjs", import.meta.url))], { stdio: ["ignore", output.fd, errors.fd] });
  const closed = once(child, "close"); await output.close(); await errors.close();
  // This subprocess constructs and verifies complete cold synthetic ancestry.
  // Bound software setup under concurrent canonical load, not hardware deadlines.
  const timer = setTimeout(() => child.kill("SIGKILL"), 110000);
  let result;
  try { result = await closed; } finally { clearTimeout(timer); }
  assert.deepEqual(result, [0, null], await readFile(resolve(root, "stderr"), "utf8"));
  assert.deepEqual(JSON.parse(await readFile(resolve(root, "stdout"), "utf8")), { rejected: true, code: "v2_operator_owner" });
});
