import test from "node:test";
import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { chmod, mkdir, mkdtemp, readFile, realpath, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { isLive, liveIdentity } from "./host.mjs";
import { awaitPending, launch, send, stopHolder } from "./noise-operator.mjs";
import { parseCommand, replyAfter } from "./noise-protocol.mjs";

const parentProgram = fileURLToPath(new URL("./noise-fake-parent.test-helper.mjs", import.meta.url));
const options = { parentProgram, readyTimeoutMs: 10_000, replyTimeoutMs: 10_000 };

async function attemptRoot(mode = "ready") {
  const parent = await realpath(await mkdtemp(resolve(process.env.TEST_TMPDIR ?? tmpdir(), "noise-operator-")));
  const root = resolve(parent, "attempt-001");
  await mkdir(root, { mode: 0o700 });
  await writeFile(resolve(root, "fake-parent-mode"), mode, { mode: 0o600 });
  return root;
}

async function launchRecord(root) { return JSON.parse(await readFile(`${root}.operator/launch.json`, "utf8")); }

async function cleanup(root) {
  const record = await launchRecord(root).catch(() => null);
  for (const identity of [record?.parent, record?.holder]) {
    if (identity && await isLive(identity)) process.kill(-identity.pgid, "SIGKILL");
  }
}

async function rejectsWith(promise, code) { await assert.rejects(promise, (error) => error.code === code); }

test("launch waits for the parent's readiness line", async (t) => {
  // Arrange
  const root = await attemptRoot();
  t.after(() => cleanup(root));
  // Act
  const result = await launch(root, options);
  // Assert
  assert.equal(result.reply.event, "supervisor_ready");
});

test("a second launch for the same attempt is refused", async (t) => {
  // Arrange
  const root = await attemptRoot();
  t.after(() => cleanup(root));
  await launch(root, options);
  // Act / Assert
  await rejectsWith(launch(root, options), "noise_operator_already_launched");
});

test("launch refuses an attempt root that is not owner-only", async () => {
  // Arrange
  const root = await attemptRoot();
  await chmod(root, 0o755);
  // Act / Assert
  await rejectsWith(launch(root, options), "private_directory_mode");
});

test("a parent that fails before readiness reports its error and its holder is stopped", async (t) => {
  // Arrange
  const root = await attemptRoot("fail");
  t.after(() => cleanup(root));
  // Act
  const error = await launch(root, options).catch((caught) => caught);
  // Assert
  assert.equal(error.code, "noise_operator_not_ready");
  assert.equal(error.reply.code, "supervisor_start_failed");
  assert.equal(await isLive((await launchRecord(root)).holder), false);
});

test("send returns exactly the reply to its own command across several commands", async (t) => {
  // Arrange
  const root = await attemptRoot();
  t.after(() => cleanup(root));
  await launch(root, options);
  // Act
  const status = await send(root, '{"action":"status"}', options);
  const install = await send(root, '{"action":"install","index":3}', options);
  // Assert
  assert.deepEqual([status, install], [{ event: "echo", action: "status" }, { event: "echo", action: "install", index: 3 }]);
});

test("send records each command and its reply event privately", async (t) => {
  // Arrange
  const root = await attemptRoot();
  t.after(() => cleanup(root));
  await launch(root, options);
  // Act
  await send(root, '{"action":"status"}', options);
  // Assert
  const rows = (await readFile(`${root}.operator/commands.jsonl`, "utf8")).trim().split("\n").map((line) => JSON.parse(line));
  assert.deepEqual(rows.map((row) => [row.kind, row.sequence, row.command ?? row.replyEvent]), [["sent", 1, { action: "status" }], ["reply", 1, "echo"]]);
});

test("a concurrent send is refused while another send owns the lock", async (t) => {
  // Arrange
  const root = await attemptRoot("slow");
  t.after(() => cleanup(root));
  await launch(root, options);
  const first = send(root, '{"action":"status"}', options);
  for (let index = 0; index < 100 && !(await readFile(`${root}.operator/send.lock`).catch(() => null)); index++) await new Promise((done) => setTimeout(done, 20));
  // Act / Assert
  await rejectsWith(send(root, '{"action":"status"}', options), "noise_operator_send_busy");
  await first;
});

test("a lock left by an exited sender is taken over", async (t) => {
  // Arrange
  const root = await attemptRoot();
  t.after(() => cleanup(root));
  await launch(root, options);
  const sender = spawn("/bin/sleep", ["0"]);
  await new Promise((done) => sender.once("close", done));
  await writeFile(`${root}.operator/send.lock`, JSON.stringify({ pid: sender.pid, pgid: sender.pid, startedAt: "Thu Jan  1 00:00:00 1970" }), { mode: 0o600 });
  // Act
  const reply = await send(root, '{"action":"status"}', options);
  // Assert
  assert.equal(reply.event, "echo");
});

test("the holder cannot be stopped while the parent still reads commands", async (t) => {
  // Arrange
  const root = await attemptRoot();
  t.after(() => cleanup(root));
  await launch(root, options);
  // Act / Assert
  await rejectsWith(stopHolder(root, options), "noise_operator_parent_live");
});

test("after exit the holder stops and further sends are refused", async (t) => {
  // Arrange
  const root = await attemptRoot();
  t.after(() => cleanup(root));
  await launch(root, options);
  const exit = await send(root, '{"action":"exit"}', options);
  const record = await launchRecord(root);
  for (let index = 0; index < 40 && await isLive(record.parent); index++) await new Promise((done) => setTimeout(done, 100));
  // Act
  const stopped = await stopHolder(root, options);
  // Assert
  assert.deepEqual([exit.event, stopped.event, await isLive(record.holder)], ["parent_exiting", "holder_stopped", false]);
  await rejectsWith(send(root, '{"action":"status"}', options), "noise_operator_parent_not_running");
});

test("send before any launch is refused", async () => {
  // Arrange
  const root = await attemptRoot();
  // Act / Assert
  await rejectsWith(send(root, '{"action":"status"}', options), "noise_operator_not_launched");
});

test("commands outside the parent's closed set are refused before any effect", () => {
  // Arrange
  const lines = ['{"action":"install","index":5}', '{"action":"install"}', '{"action":"status","extra":1}', '{"action":"flash"}', "not json"];
  // Act
  const codes = lines.map((line) => { try { parseCommand(line); return "accepted"; } catch (error) { return error.code; } });
  // Assert
  assert.deepEqual(codes, ["noise_command_install_index", "noise_command_install_index", "noise_command_fields", "noise_command_action", "noise_command_json"]);
});

test("a partially written reply line is not yet a reply", () => {
  // Arrange
  const text = '{"event":"supervisor_ready"}\n{"event":"ec';
  // Act
  const reply = replyAfter(text, 1);
  // Assert
  assert.equal(reply, null);
});

test("the test process identity is readable", async () => {
  // Arrange / Act
  const identity = await liveIdentity(process.pid);
  // Assert
  assert.equal(identity.pid, process.pid);
});

test("the real parent loads its whole module graph and refuses malformed arguments", async () => {
  // Arrange
  const realParent = fileURLToPath(new URL("./noise-parent.mjs", import.meta.url));
  // Act
  const output = await new Promise((done) => {
    const child = spawn(process.execPath, [realParent, "--private-root", "relative"], { stdio: ["ignore", "pipe", "ignore"] });
    let text = ""; child.stdout.on("data", (chunk) => { text += chunk; }); child.once("close", () => done(text));
  });
  // Assert
  assert.deepEqual(JSON.parse(output), { event: "operator_error", code: "noise_parent_arguments" });
});

test("the real parent reports an attempt without a preflight context and leaves no holder", async (t) => {
  // Arrange
  const root = await attemptRoot();
  t.after(() => cleanup(root));
  const realParent = fileURLToPath(new URL("./noise-parent.mjs", import.meta.url));
  // Act
  const error = await launch(root, { ...options, parentProgram: realParent }).catch((caught) => caught);
  // Assert
  assert.deepEqual([error.code, error.reply?.event], ["noise_operator_not_ready", "operator_error"]);
  assert.equal(await isLive((await launchRecord(root)).holder), false);
});

test("a parent that reports an error but has not exited yet still has its holder stopped", async (t) => {
  // Arrange
  const root = await attemptRoot("fail-linger");
  t.after(() => cleanup(root));
  // Act
  const error = await launch(root, options).catch((caught) => caught);
  // Assert
  assert.equal(error.code, "noise_operator_not_ready");
  assert.equal(await isLive((await launchRecord(root)).holder), false);
});

test("after a sender times out, a new send is refused until the outstanding reply is awaited", async (t) => {
  // Arrange
  const root = await attemptRoot("slow");
  t.after(() => cleanup(root));
  await launch(root, options);
  await rejectsWith(send(root, '{"action":"install","index":1}', { ...options, replyTimeoutMs: 200 }), "noise_operator_reply_timeout");
  // Act
  const refused = await send(root, '{"action":"status"}', options).catch((error) => error.code);
  const late = await awaitPending(root, options);
  const next = await send(root, '{"action":"status"}', options);
  // Assert
  assert.deepEqual([refused, late, next], ["noise_operator_reply_pending", { event: "echo", action: "install", index: 1 }, { event: "echo", action: "status" }]);
});

test("awaiting with no outstanding command is refused", async (t) => {
  // Arrange
  const root = await attemptRoot();
  t.after(() => cleanup(root));
  await launch(root, options);
  await send(root, '{"action":"status"}', options);
  // Act / Assert
  await rejectsWith(awaitPending(root, options), "noise_operator_no_pending_reply");
});
