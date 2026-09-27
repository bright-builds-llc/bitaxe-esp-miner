import { spawn, execFile } from "node:child_process";
import { promisify } from "node:util";
import { check, sha256 } from "./values.mjs";
const run = promisify(execFile);
/** One bounded read-only Git batch; published blob contents never enter tool output. */
export async function verifyPublishedSources(firmwareRoot, commit, entries) {
  const env = { PATH: "/usr/bin:/bin", LANG: "C", LC_ALL: "C", GIT_OPTIONAL_LOCKS: "0" };
  await run("git", ["merge-base", "--is-ancestor", commit, "refs/remotes/origin/main"], { cwd: firmwareRoot, env, timeout: 10000 });
  const output = await new Promise((accept, reject) => {
    const child = spawn("git", ["cat-file", "--batch"], { cwd: firmwareRoot, env, stdio: ["pipe", "pipe", "pipe"] });
    const chunks = []; let bytes = 0, maybeError;
    const fail = error => { maybeError ??= error; child.kill("SIGKILL"); };
    const timer = setTimeout(() => fail(Error("bootstrap_corpus_git_timeout")), 30000);
    child.on("error", fail); child.stdin.on("error", fail);
    child.stdout.on("data", chunk => { bytes += chunk.length; if (bytes > 64 * 1024 * 1024) fail(Error("bootstrap_corpus_git_bound")); else chunks.push(chunk); });
    child.stderr.on("data", () => fail(Error("bootstrap_corpus_git_stderr")));
    child.on("close", (code, signal) => { clearTimeout(timer); if (maybeError || code !== 0 || signal !== null) reject(maybeError ?? Error("bootstrap_corpus_git_exit")); else accept(Buffer.concat(chunks)); });
    child.stdin.end(entries.map(row => `${commit}:${row.path}\n`).join(""));
  });
  let offset = 0;
  for (const entry of entries) {
    const end = output.indexOf(10, offset); check(end >= offset, "bootstrap_corpus_git_header");
    const header = /^([a-f0-9]{40}) blob (\d+)$/u.exec(output.subarray(offset, end).toString("ascii"));
    check(header && Number(header[2]) === entry.length, "bootstrap_corpus_git_blob");
    offset = end + 1; const bytes = output.subarray(offset, offset + entry.length);
    check(bytes.length === entry.length && sha256(bytes) === entry.sha256 && output[offset + entry.length] === 10, "bootstrap_corpus_git_bytes");
    offset += entry.length + 1;
  }
  check(offset === output.length, "bootstrap_corpus_git_trailing");
}
