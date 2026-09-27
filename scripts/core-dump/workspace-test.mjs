import { spawnSync } from "node:child_process";
import { resolve } from "node:path";

// The explicit local integration target needs the managed IDF environment, not a device.
const repo = process.env.BUILD_WORKSPACE_DIRECTORY ?? process.env.CORE_DUMP_WORKSPACE;
if (!repo) throw Error("core_dump_workspace_required");
const result = spawnSync(process.execPath, ["--test", resolve(repo, "scripts/core-dump/main.test.mjs"), resolve(repo, "scripts/core-dump/cutoff.test.mjs")], {
  cwd: repo, env: { ...process.env, BUILD_WORKSPACE_DIRECTORY: repo }, stdio: "inherit", timeout: 120000,
});
if (result.error || result.signal || result.status !== 0) process.exitCode = 1;
