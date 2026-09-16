import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdtemp, readdir, realpath, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import test from "node:test";
import { main, parseArgs } from "./main.mjs";

function args(action, root) {
  const value = [action, "--private-root", root];
  if (action === "preflight") for (const [key, v] of Object.entries({ "firmware-root": "/missing/firmware", "gate-root": "/missing/gate",
    "package-manifest": "/missing/manifest", "fixture-binary": "/missing/fixture", "attempt-ordinal": "1", "predecessor-receipt": "/missing/receipt" })) value.push(`--${key}`, v);
  if (action === "recover") value.push("--attempt-root", "/missing/attempt");
  if (action === "finalize") value.push("--cleanup-receipt", "/missing/cleanup");
  return value;
}
for (const action of ["preflight", "serve", "recover", "finalize", "review"]) {
  test(`${action} rejects absent prerequisites without creating artifacts`, async (t) => {
    const root = await realpath(await mkdtemp(join(tmpdir(), "noise-closed-"))); t.after(() => rm(root, { recursive: true }));
    const input = args(action, join(root, "attempt-001"));
    await assert.rejects(main(input));
    const result = spawnSync(process.execPath, [fileURLToPath(new URL("./main.mjs", import.meta.url)), ...input], { cwd: root, encoding: "utf8" });
    assert.equal(result.status, 1, result.stderr);
    assert.deepEqual(JSON.parse(result.stdout), { ready: false, device_effects: false, hardware_qualified: false, error: action === "recover" ? "noise_recovery_unavailable" : action === "serve" ? "noise_protected_stdout_required" : "noise_operation_rejected" });
    assert.deepEqual(await readdir(root), []);
  });
  test(`${action} rejects credentials and duplicate flags without inspecting paths`, () => {
    for (const flag of ["authority-directory", "pool-credentials", "wifi-credentials", "private-root", "force"]) {
      assert.throws(() => parseArgs([...args(action, "/absent/attempt"), `--${flag}`, "/unread"]), { code: "noise_option_rejected" });
    }
  });
}
