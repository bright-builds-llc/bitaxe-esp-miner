import assert from "node:assert/strict";
import test from "node:test";
import { spawn, spawnSync } from "node:child_process";
import { mkdtemp, chmod, writeFile, readFile, stat, readdir, realpath, rm, mkdir, symlink } from "node:fs/promises";
import { tmpdir } from "node:os";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { proof } from "../str005-noise-serial/files.mjs";
import { nodeRuntimeEnvironment } from "../str005-noise-serial/node-runtime.mjs";
const HERE = dirname(fileURLToPath(import.meta.url));
const quote = value => `'${value.replaceAll("'", "'\\''")}'`;
async function writer(root) {
  // Canonical tests receive the real Rust executable as declared runfiles data.
  // Never consult an ambient compiler from a Bazel test process.
  let maybeBuilt;
  try { maybeBuilt = await realpath(resolve(HERE, "../../tools/flash/private_evidence_writer_test_fixture")); }
  catch (error) { if (error.code !== "ENOENT") throw error; }
  if (maybeBuilt) { assert((await stat(maybeBuilt)).isFile()); return maybeBuilt; }
  const canonical = ["JS_BINARY__NODE_BINARY", "RUNFILES_DIR", "TEST_SRCDIR"].some(name => process.env[name] !== undefined);
  assert.equal(canonical, false, "canonical Rust writer must be present in declared runfiles");
  const binary = resolve(root, "rust-writer");
  // Resolve only an already installed toolchain, then bypass the rustup shim.
  // A missing toolchain fails; this test never installs or updates one.
  const selected = spawnSync("rustup", ["which", "--toolchain", "stable", "rustc"], {
    timeout: 5000, maxBuffer: 4096, encoding: "utf8", stdio: ["ignore", "pipe", "pipe"],
  });
  assert.equal(selected.error?.code ?? null, null, "installed stable compiler resolver must launch");
  assert.equal(selected.status, 0, "an installed stable compiler is required"); assert.equal(selected.signal, null);
  const compiler = await realpath(selected.stdout.trim());
  const result = spawnSync(compiler, ["--edition=2021", "--crate-name", "operator_private_output_writer",
    resolve(HERE, "operator-private-output-writer.rs"), "-o", binary], { timeout: 30000, maxBuffer: 65536, stdio: ["ignore", "pipe", "pipe"] });
  assert.equal(result.error?.code ?? null, null, "real Rust writer compiler must launch");
  assert.equal(result.status, 0, "real Rust writer must compile"); assert.equal(result.signal, null);
  for (const bytes of [result.stdout, result.stderr]) bytes?.fill(0);
  return binary;
}
async function fixture(t) {
  const root = await realpath(await mkdtemp(resolve(tmpdir(), "operator-private-output-")));
  await chmod(root, 0o700); t.after(() => rm(root, { recursive: true, force: true }));
  await writeFile(resolve(root, ".operator-private-output-test"), "synthetic-filesystem-only\n", { mode: 0o600 });
  const binary = await writer(root);
  await writeFile(resolve(root, "just"), `#!/bin/sh\nexec ${quote(binary)} ${quote(root)}\n`, { mode: 0o700 });
  return { root, binary };
}
async function execute(root) {
  const original = process.umask(0o022); let child;
  try {
    child = spawn(process.execPath, ["--import", resolve(HERE, "operator-private-output-hook.mjs"),
      resolve(HERE, "operator-child.mjs"), root, "flash", "0"], {
      stdio: ["ignore", "pipe", "pipe", "ipc"],
      env: { PATH: `${root}:${process.env.PATH ?? "/usr/bin:/bin"}`, TMPDIR: tmpdir(), LANG: "C", LC_ALL: "C", ...nodeRuntimeEnvironment() },
    });
    assert.equal(process.umask(), 0o022, "child policy cannot mutate its parent's mask");
  } finally { process.umask(original); }
  let outputBytes = 0, timer;
  for (const stream of [child.stdout, child.stderr]) stream.on("data", bytes => { outputBytes += bytes.length; bytes.fill(0); });
  child.once("message", message => { if (message.ready === true) child.send({ kind: "synthetic-test" }); });
  try {
    return await new Promise((done, reject) => {
      timer = setTimeout(() => { child.kill("SIGKILL"); reject(Error("operator_private_output_timeout")); }, 10000);
      child.once("error", () => reject(Error("operator_private_output_spawn")));
      child.once("close", (code, signal) => done({ code, signal, outputBytes }));
    });
  } finally { clearTimeout(timer); }
}

test("actual operator child confines the real Rust writer beneath a protected ancestor from umask022", { skip: process.platform === "win32" }, async t => {
  // Arrange: only admission is synthetic; operator entrypoint, spawn, Rust writer and files are real.
  const f = await fixture(t);
  // Act
  assert.deepEqual(await execute(f.root), { code: 0, signal: null, outputBytes: 0 });
  // Assert: nested creation and atomic rename retain private modes and exact bytes.
  for (const path of ["install-0", "install-0/nested"]) assert.equal((await stat(resolve(f.root, path))).mode & 0o777, 0o700);
  for (const [path, value] of [["flash-command-evidence.json", '{"synthetic":true}\n'],
    ["nested/flash-monitor.log", "synthetic-runtime-only\n"], ["nested/atomic.json", "synthetic-atomic-output\n"]]) {
    assert.equal((await stat(resolve(f.root, "install-0", path))).mode & 0o777, 0o600);
    assert.equal(await readFile(resolve(f.root, "install-0", path), "utf8"), value);
  }
  assert.deepEqual((await proof(resolve(f.root, "install-0"), "flash-command-evidence.json")).value, { synthetic: true });
  assert.deepEqual((await readdir(resolve(f.root, "install-0/nested"))).sort(), ["atomic.json", "flash-monitor.log"]);
});

for (const kind of ["partial", "symlink"]) test(`owned operator cannot repair or replace an existing ${kind} output tree`, async t => {
  // Arrange
  const f = await fixture(t), output = resolve(f.root, "install-0"), canary = resolve(f.root, "canary");
  await mkdir(canary, { mode: 0o700 }); await writeFile(resolve(canary, "retained"), "unchanged", { mode: 0o600 });
  if (kind === "partial") await mkdir(output, { mode: 0o755 }); else await symlink(canary, output);
  const before = (await stat(kind === "partial" ? output : canary)).mode;
  // Act / Assert: no chmod, recursive repair, or Rust effect follows rejected admission.
  assert.notEqual((await execute(f.root)).code, 0);
  assert.equal((await stat(kind === "partial" ? output : canary)).mode, before);
  assert.equal(await readFile(resolve(canary, "retained"), "utf8"), "unchanged");
  assert.deepEqual(await readdir(kind === "partial" ? output : canary), kind === "partial" ? [] : ["retained"]);
});
