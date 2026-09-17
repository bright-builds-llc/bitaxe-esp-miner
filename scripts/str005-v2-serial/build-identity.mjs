import { readFile, writeFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { resolve } from "node:path";
import { fixtureBuildIdentity } from "../str005-noise-serial/build-identity.mjs";
import { sha256 } from "./values.mjs";

/** Separate receipt namespace; the shared canonical source-status reader remains unchanged. */
export async function v2FixtureBuildIdentity(statusPath, binaryPath) {
  const identity = await fixtureBuildIdentity(statusPath, binaryPath);
  return { ...identity, schema: "str005-v2-fixture-build-v1",
    writerSha256: sha256(await readFile(fileURLToPath(import.meta.url))) };
}
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const [status, binary, output, ...extra] = process.argv.slice(2);
  if (!status || !binary || !output || extra.length) throw new Error("v2_build_identity_arguments");
  const executionRoot = process.env.JS_BINARY__EXECROOT ?? process.cwd();
  const identity = await v2FixtureBuildIdentity(resolve(executionRoot, status), resolve(executionRoot, binary));
  await writeFile(resolve(executionRoot, output), `${JSON.stringify(identity)}\n`, { flag: "wx" });
}
