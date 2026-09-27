import { readFile, writeFile } from "node:fs/promises";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { check, sha256 } from "./values.mjs";
export function stableIdentity(status) {
  const field = name => { const rows = status.split(/\r?\n/u).filter(row => row.startsWith(`${name} `)); check(rows.length === 1, "bootstrap_test_status"); return rows[0].slice(name.length + 1); };
  const sourceCommit = field("STABLE_BITAXE_SOURCE_COMMIT"), dirty = field("STABLE_BITAXE_SOURCE_DIRTY");
  check(/^[a-f0-9]{40}$/u.test(sourceCommit) && ["true", "false"].includes(dirty), "bootstrap_test_status");
  return { schema: "usb-bootstrap-reader-test-identity-v1", sourceCommit, sourceDirty: dirty === "true" };
}
export async function writeIdentity(status, environment, output) {
  const value = stableIdentity(await readFile(status, "utf8"));
  await writeFile(environment, `BOOTSTRAP_TEST_SOURCE_COMMIT=${value.sourceCommit}\nBOOTSTRAP_TEST_SOURCE_DIRTY=${value.sourceDirty}\n`, { flag: "wx" });
  await writeFile(output, JSON.stringify({ ...value, writerSha256: sha256(await readFile(fileURLToPath(import.meta.url))) }) + "\n", { flag: "wx" });
}
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const [status, environment, output, ...extra] = process.argv.slice(2); check(status && environment && output && extra.length === 0, "bootstrap_test_status");
  const root = process.env.JS_BINARY__EXECROOT ?? process.cwd(); await writeIdentity(resolve(root, status), resolve(root, environment), resolve(root, output));
}
