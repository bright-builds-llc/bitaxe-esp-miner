import { createHash } from "node:crypto";
import { readFile, writeFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { resolve } from "node:path";
const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
/** Bazel action only: its declared fixture input is built by the same source graph. */
export async function fixtureBuildIdentity(statusPath, binaryPath) {
  const status = await readFile(statusPath, "utf8");
  function field(name) {
    const rows = status.split(/\r?\n/u).filter((line) => line.startsWith(`${name} `));
    if (rows.length !== 1) throw new Error("noise_build_status_field");
    return rows[0].slice(name.length + 1);
  }
  const commit = field("STABLE_BITAXE_SOURCE_COMMIT"), dirty = field("STABLE_BITAXE_SOURCE_DIRTY");
  if (!/^[a-f0-9]{40}$/u.test(commit) || !["true", "false"].includes(dirty)) throw new Error("noise_build_status_invalid");
  return { schema: "noise-serial-fixture-build-v2", sourceCommit: commit, sourceDirty: dirty === "true",
    fixtureSha256: hash(await readFile(binaryPath)), writerSha256: hash(await readFile(fileURLToPath(import.meta.url))) };
}
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const [status, binary, output, ...extra] = process.argv.slice(2);
  if (!status || !binary || !output || extra.length) throw new Error("noise_build_identity_arguments");
  const executionRoot = process.env.JS_BINARY__EXECROOT ?? process.cwd();
  await writeFile(resolve(executionRoot, output), `${JSON.stringify(await fixtureBuildIdentity(resolve(executionRoot, status), resolve(executionRoot, binary)))}\n`, { flag: "wx" });
}
