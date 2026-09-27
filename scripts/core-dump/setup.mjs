import { execFileSync } from "node:child_process";
import { mkdir, readFile } from "node:fs/promises";
import { join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { managedTools } from "./main.mjs";
import { check, privateDirectory } from "./files.mjs";
import { privateProcess } from "./process.mjs";

/** Install only the GDB version selected by the already pinned ESP-IDF tools manifest. */
export async function setup() {
  process.umask(0o077);
  check(process.argv.length === 2, "setup_arguments");
  const repo = process.env.BUILD_WORKSPACE_DIRECTORY ?? execFileSync("git", ["rev-parse", "--show-toplevel"],
    { encoding: "utf8", stdio: ["ignore", "pipe", "ignore"] }).trim();
  const tools = await managedTools(repo);
  const manifest = JSON.parse(await readFile(join(tools.idf, "tools/tools.json"), "utf8"));
  const selected = manifest.tools.find(tool => tool.name === "xtensa-esp-elf-gdb");
  const recommended = selected?.versions.filter(version => version.status === "recommended");
  check(recommended?.length === 1 && /^[0-9._]+$/.test(recommended[0].name), "setup_manifest");
  const parent = join(repo, "scratch/core-dump-tools");
  await mkdir(parent, { recursive: true, mode: 0o700 }); await privateDirectory(parent);
  const root = join(parent, `setup-${Date.now()}`); await mkdir(root, { mode: 0o700 });
  await privateProcess(tools.python, [join(tools.idf, "tools/idf_tools.py"), "install", "xtensa-esp-elf-gdb"], root, "install",
    { ...process.env, IDF_TOOLS_PATH: join(repo, ".embuild/espressif"), IDF_PATH: tools.idf }, 300000);
  return { status: "installed", tool: "xtensa-esp-elf-gdb", version: recommended[0].name, device_effects: false };
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  setup().then(value => console.log(JSON.stringify(value))).catch(() => {
    console.error('{"status":"blocked","category":"managed_gdb_setup_failed"}'); process.exitCode = 1;
  });
}
