// File-system shell for staged web UI variants.
import { createHash } from "node:crypto";
import { mkdir, readFile, readdir, stat, writeFile } from "node:fs/promises";
import path from "node:path";

import { maybeWebUiVariant, webUiStageSchema, type StagedFile, type WebUiStageMetadata } from "./web-ui-stage.js";

/** Staged layout: `<stage>/www/**` is packed into `www.bin`; `<stage>/web-ui.json` describes it. */
export const stageWwwDirectory = "www";
export const stageMetadataFile = "web-ui.json";

/** Resolves an action or `bazel run` argument against the Bazel exec root when one is active. */
export function resolveBazelPath(value: string): string {
  const maybeRoot = process.env["JS_BINARY__EXECROOT"];
  return path.resolve(maybeRoot ?? process.cwd(), value);
}

/** Reads every regular file below `root`, with `/`-separated relative paths, sorted. */
export async function readTree(root: string, prefix = ""): Promise<StagedFile[]> {
  const entries = await readdir(path.join(root, prefix), { withFileTypes: true });
  const nested = await Promise.all(entries.map(async (entry) => {
    const relative = prefix === "" ? entry.name : `${prefix}/${entry.name}`;
    const absolute = path.join(root, relative);
    if ((await stat(absolute)).isDirectory()) return readTree(root, relative);
    return [{ path: relative, bytes: await readFile(absolute) }];
  }));
  return nested.flat().sort((left, right) => left.path.localeCompare(right.path));
}

/** Reads explicit input files, naming each relative to `stripPrefix`. */
export async function readInputs(inputs: readonly string[], stripPrefix: string): Promise<StagedFile[]> {
  const files: StagedFile[][] = await Promise.all(inputs.map(async (input) => {
    const absolute = resolveBazelPath(input);
    if ((await stat(absolute)).isDirectory()) return readTree(absolute);
    const normalized = input.split(path.sep).join("/");
    const index = normalized.indexOf(stripPrefix);
    if (stripPrefix === "" || index < 0) throw new Error(`web UI input is outside the source root: ${input}`);
    return [{ path: normalized.slice(index + stripPrefix.length), bytes: await readFile(absolute) }];
  }));
  return files.flat();
}

export async function writeStage(outDir: string, files: readonly StagedFile[], metadata: WebUiStageMetadata): Promise<void> {
  for (const file of files) {
    const target = path.join(outDir, stageWwwDirectory, ...file.path.split("/"));
    await mkdir(path.dirname(target), { recursive: true });
    await writeFile(target, file.bytes);
  }
  await writeFile(path.join(outDir, stageMetadataFile), `${JSON.stringify(metadata, null, 2)}\n`);
}

/** Reads a staged directory and checks that its metadata describes exactly its files. */
export async function readStage(stageDir: string): Promise<{ files: StagedFile[]; metadata: WebUiStageMetadata }> {
  const metadata = JSON.parse(await readFile(path.join(stageDir, stageMetadataFile), "utf8")) as WebUiStageMetadata;
  if (metadata.schema_version !== webUiStageSchema || maybeWebUiVariant(String(metadata.variant)) === null) {
    throw new Error("staged web UI metadata is invalid");
  }
  const files = await readTree(path.join(stageDir, stageWwwDirectory));
  const described = metadata.files.map((file) => `${file.path}:${String(file.bytes)}:${file.sha256}`).sort();
  const actual = files
    .map((file) => `${file.path}:${String(file.bytes.length)}:${createHash("sha256").update(file.bytes).digest("hex")}`)
    .sort();
  if (JSON.stringify(described) !== JSON.stringify(actual)) throw new Error("staged web UI files do not match their metadata");
  return { files, metadata };
}
