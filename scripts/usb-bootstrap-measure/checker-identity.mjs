import { readFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { execFileSync } from "node:child_process";
import { cleanPushed, git } from "../fixed-usb-qualification/contract.mjs";
import { canonical } from "../str005-noise-serial/files.mjs";
import { sources, SOURCE_ROOTS } from "./sources.mjs";
import { sourceShape, hex } from "./shapes.mjs";
import { PREFLIGHT_AMENDMENT, check, object, sha256 } from "./values.mjs";
export const CHECKER_FILES = ["preflight-closure", "preflight-binding", "checker-identity", "preflight-corpus", "preflight-corpus-git", "source-snapshot", "source-layout-check", "snapshot", "context", "sources", "values", "shapes", "native-writer", "native-stack", "main"].map(name => `scripts/usb-bootstrap-measure/${name}.mjs`);
export function publishedSources(root, commit) {
  hex(commit, 40);
  const options = { encoding: "utf8", timeout: 30000, maxBuffer: 134217728, stdio: ["pipe", "pipe", "pipe"] };
  const paths = execFileSync("git", ["-C", root, "ls-tree", "-r", "--name-only", commit, "--", ...SOURCE_ROOTS], options).trim().split("\n").filter(Boolean).sort();
  check(paths.length > 0 && paths.every(path => /^[A-Za-z0-9_./-]+$/u.test(path) && !path.startsWith("/") && !path.split("/").includes("..")), "bootstrap_checker_paths");
  const bytes = execFileSync("git", ["-C", root, "cat-file", "--batch"], { ...options, encoding: undefined, input: paths.map(path => `${commit}:${path}\n`).join("") });
  let offset = 0;
  const records = paths.map(path => {
    const end = bytes.indexOf(10, offset); check(end >= offset, "bootstrap_checker_blobs");
    const match = /^[a-f0-9]{40,64} blob ([0-9]+)$/u.exec(bytes.subarray(offset, end).toString()); check(match, "bootstrap_checker_blobs");
    const length = Number(match[1]); check(Number.isSafeInteger(length) && length >= 0 && end + 1 + length < bytes.length && bytes[end + 1 + length] === 10, "bootstrap_checker_blobs");
    const sha = sha256(bytes.subarray(end + 1, end + 1 + length)); offset = end + length + 2; return { path, sha256: sha, length };
  });
  check(offset === bytes.length, "bootstrap_checker_blobs"); return records;
}
export async function validateChecker(value, failedContext, operations = {}) {
  object(value, ["commit", "sources"]); hex(value.commit, 40); sourceShape(value.sources);
  check(CHECKER_FILES.every(path => value.sources.some(row => row.path === path)), "bootstrap_checker_membership");
  check(value.commit !== failedContext.package.firmware_commit && value.sources.find(row => row.path === PREFLIGHT_AMENDMENT.path)?.sha256 === PREFLIGHT_AMENDMENT.sha256,
    "bootstrap_checker_identity");
  const expected = await (operations.publishedSources ?? publishedSources)(failedContext.firmwareRoot, value.commit);
  check(canonical(expected) === canonical(value.sources), "bootstrap_checker_identity"); return value;
}
export async function createChecker(failedContext, operations = {}) {
  const root = failedContext.firmwareRoot, commit = (operations.git ?? git)(root, ["rev-parse", "HEAD"]);
  (operations.cleanPushed ?? cleanPushed)(root, commit);
  const value = { commit, sources: await sources(root) };
  for (const path of CHECKER_FILES) check(value.sources.find(row => row.path === path)?.sha256 ===
    sha256(await readFile(fileURLToPath(new URL(`./${path.split("/").at(-1)}`, import.meta.url)))), "bootstrap_running_checker_changed");
  await validateChecker(value, failedContext, operations);
  (operations.cleanPushed ?? cleanPushed)(root, commit); return value;
}
