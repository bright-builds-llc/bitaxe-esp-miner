// `just web-ui-sizes`: prints raw, gzip and served bytes per file, variant
// totals and measured SPIFFS use; `--append-history` also appends CSV rows.
import { execFileSync } from "node:child_process";
import { appendFile, mkdir, mkdtemp, readFile, rm, stat, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";

import { readStage } from "./web-ui-files.js";
import { formatSizeReport, sizeHistoryHeader, sizeHistoryRows, spiffsUsage, variantSize, type SpiffsUsage } from "./web-ui-sizes.js";
import type { StagedFile, WebUiVariant } from "./web-ui-stage.js";

const spiffsgenPath = ".embuild/espressif/esp-idf/v5.5.4/components/spiffs/spiffsgen.py";
const wwwPartitionSize = "0x300000";
/** A 22-character build label is the longest `version.txt` the packager writes. */
const versionTxtPlaceholder = `${"0".repeat(22)}\n`;

type SizeArguments = { stages: string[]; maybeHistory: string | null };

function parseArguments(argv: readonly string[]): SizeArguments {
  const parsed: SizeArguments = { stages: [], maybeHistory: null };
  for (let index = 0; index < argv.length; index += 1) {
    const flag = argv[index];
    const value = argv[index + 1];
    if (flag === "--stage" && value !== undefined) parsed.stages.push(value);
    else if (flag === "--append-history" && value !== undefined) parsed.maybeHistory = value;
    else throw new Error(`unknown or incomplete argument ${String(flag)}`);
    index += 1;
  }
  return parsed;
}

function workspaceRoot(): string {
  const maybeWorkspace = process.env["BUILD_WORKSPACE_DIRECTORY"];
  if (maybeWorkspace === undefined) throw new Error("run through `just web-ui-sizes` (bazel run)");
  return maybeWorkspace;
}

function runfilePath(value: string): string {
  const maybeRunfiles = process.env["JS_BINARY__RUNFILES"] ?? process.env["RUNFILES_DIR"];
  return maybeRunfiles === undefined ? path.resolve(value) : path.join(maybeRunfiles, "_main", value);
}

/** Packs the staged files plus a placeholder `version.txt` exactly as the packager does, then measures the image. */
async function measureSpiffs(root: string, files: readonly StagedFile[]): Promise<SpiffsUsage> {
  const spiffsgen = path.join(root, spiffsgenPath);
  if (!(await stat(spiffsgen).catch(() => null))?.isFile()) throw new Error("spiffsgen.py is missing; run `just bootstrap-esp` or a firmware build first");
  const scratch = await mkdtemp(path.join(tmpdir(), "bitaxe-web-ui-sizes-"));
  try {
    const staged = path.join(scratch, "www");
    for (const file of [...files, { path: "version.txt", bytes: Buffer.from(versionTxtPlaceholder) }]) {
      const target = path.join(staged, ...file.path.split("/"));
      await mkdir(path.dirname(target), { recursive: true });
      await writeFile(target, file.bytes, { mode: 0o600 });
    }
    const image = path.join(scratch, "www.bin");
    execFileSync("python3", [spiffsgen, "--obj-name-len", "64", wwwPartitionSize, staged, image], { stdio: "inherit" });
    return spiffsUsage(await readFile(image));
  } finally {
    await rm(scratch, { recursive: true, force: true });
  }
}

function gitText(root: string, args: readonly string[]): string {
  return execFileSync("git", ["-C", root, ...args], { encoding: "utf8" }).trim();
}

async function appendHistory(root: string, relativeFile: string, rows: readonly string[]): Promise<void> {
  const file = path.resolve(root, relativeFile);
  if (!file.startsWith(`${root}${path.sep}`)) throw new Error("history file must be inside the workspace");
  const exists = (await stat(file).catch(() => null)) !== null;
  await appendFile(file, `${exists ? "" : `${sizeHistoryHeader}\n`}${rows.join("\n")}\n`);
}

async function main(): Promise<void> {
  const parsed = parseArguments(process.argv.slice(2));
  const root = workspaceRoot();
  const sizes = [];
  const usages = new Map<WebUiVariant, SpiffsUsage>();
  for (const stagePath of parsed.stages) {
    const stageDir = runfilePath(stagePath);
    const { files, metadata } = await readStage(stageDir);
    const size = variantSize(metadata.variant, files);
    const usage = await measureSpiffs(root, files);
    sizes.push(size);
    usages.set(metadata.variant, usage);
  }
  process.stdout.write(formatSizeReport(sizes, usages));
  if (parsed.maybeHistory === null) return;
  const dirty = gitText(root, ["status", "--porcelain", "--untracked-files=no"]) !== "";
  const rows = sizeHistoryRows(new Date().toISOString(), gitText(root, ["rev-parse", "--short=12", "HEAD"]), dirty, sizes, usages);
  await appendHistory(root, parsed.maybeHistory, rows);
  process.stdout.write(`web_ui_sizes=history_appended file=${parsed.maybeHistory} rows=${String(rows.length)}\n`);
}

main().catch((error: unknown) => {
  process.stderr.write(`web_ui_sizes=failed reason=${error instanceof Error ? error.message : String(error)}\n`);
  process.exitCode = 1;
});
