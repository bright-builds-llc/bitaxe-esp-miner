import test from "node:test";
import assert from "node:assert/strict";
import { readFile, stat, writeFile } from "node:fs/promises";
import { resolve } from "node:path";
import { fakeWatcherOperations, privateDirectory } from "./fixtures.test-helper.mjs";
import { createPresenceWatcher, parseWatcherLine } from "./watcher.mjs";

const IDENTITY = "5".repeat(64);

async function until(predicate, timeoutMs = 5000) {
  const deadline = Date.now() + timeoutMs;
  while (!predicate()) { if (Date.now() > deadline) throw new Error("timeout"); await new Promise((done) => setTimeout(done, 10)); }
}

test("the watcher journals closed events privately and stops cleanly on stdin EOF", async () => {
  // Arrange
  const root = await privateDirectory("watcher-");
  const fake = await fakeWatcherOperations(root, [{ elapsed_ms: 10, event: "present", enumeration_sha256: "a".repeat(64), holder_count: 1, ready: false },
    { gate: 1, elapsed_ms: 900, event: "absent" }]);
  const events = [];
  const watcher = createPresenceWatcher({ binary: fake.binary, physicalIdentity: IDENTITY, journalPath: resolve(root, "watcher.jsonl"),
    stderrPath: resolve(root, "watcher.stderr.log"), onEvent: (event) => events.push(event.event) }, { spawn: fake.spawn });
  // Act
  await watcher.start();
  await until(() => events.includes("present"));
  await writeFile(fake.control, "1");
  await until(() => events.includes("absent"));
  const result = await watcher.stop();
  // Assert
  assert.deepEqual(events, ["started", "present", "absent", "stopped"]);
  assert.deepEqual(result, { exit_code: 0, signal: null, stopped_on_request: true });
  assert.equal((await stat(resolve(root, "watcher.jsonl"))).mode & 0o777, 0o600);
  assert.equal((await readFile(resolve(root, "watcher.jsonl"), "utf8")).includes("/dev/"), false);
});

test("a malformed watcher line kills the watcher and reports a failure", async () => {
  // Arrange
  const root = await privateDirectory("watcher-");
  const fake = await fakeWatcherOperations(root, [{ raw: JSON.stringify({ schema: "bwg-usb-presence-watch-v1", sequence: 2, elapsed_ms: 1, event: "present", port: "/dev/cu.x" }) }]);
  const events = [];
  const watcher = createPresenceWatcher({ binary: fake.binary, physicalIdentity: IDENTITY, journalPath: resolve(root, "watcher.jsonl"),
    stderrPath: resolve(root, "watcher.stderr.log"), onEvent: (event) => events.push(event.event) }, { spawn: fake.spawn });
  // Act
  await watcher.start();
  await until(() => events.includes("failed"));
  // Assert
  assert.deepEqual(events, ["started", "failed"]);
  assert.equal(watcher.running(), false);
});

test("a binary whose digest drifted is refused before spawning", async () => {
  // Arrange
  const root = await privateDirectory("watcher-");
  const fake = await fakeWatcherOperations(root, []);
  const watcher = createPresenceWatcher({ binary: { ...fake.binary, sha256: "0".repeat(64) }, physicalIdentity: IDENTITY,
    journalPath: resolve(root, "watcher.jsonl"), stderrPath: resolve(root, "watcher.stderr.log"), onEvent: () => undefined }, { spawn: fake.spawn });
  // Act / Assert
  await assert.rejects(watcher.start(), (error) => error.code === "watcher_binary");
});

test("watcher lines are closed: a device node or an unknown field is refused", () => {
  // Arrange
  const base = { schema: "bwg-usb-presence-watch-v1", sequence: 1, elapsed_ms: 0, event: "present" };
  // Act / Assert
  assert.equal(parseWatcherLine(JSON.stringify(base)).event, "present");
  assert.throws(() => parseWatcherLine(JSON.stringify({ ...base, port: "/dev/cu.usbmodem1" })), (error) => error.code === "watcher_line");
  assert.throws(() => parseWatcherLine(JSON.stringify({ ...base, event: "opened" })), (error) => error.code === "watcher_line");
});
