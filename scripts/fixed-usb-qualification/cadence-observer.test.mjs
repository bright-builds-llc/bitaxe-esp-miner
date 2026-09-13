import assert from "node:assert/strict";
import { EventEmitter } from "node:events";
import { chmod, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { PassThrough, Writable } from "node:stream";
import test from "node:test";
import { createCadenceObserver } from "./cadence-observer.mjs";
import { fileDigest } from "./contract.mjs";

const event = (kind, options = {}) => ({ schema: "cpu0-cadence-observer-v1", event: kind, elapsedMs: 1,
  messageCount: 0, totalBytes: 0, byteCount: 0, reason: null, ...options });
class Child extends EventEmitter {
  constructor({ invalid = false, stderr = false, noStop = false } = {}) {
    super(); this.exitCode = null; this.signalCode = null; this.kills = []; this.stdout = new PassThrough(); this.stderr = new PassThrough();
    this.inputs = [];
    this.stdin = new Writable({ write: (bytes, _encoding, done) => {
      this.inputs.push(JSON.parse(bytes.toString()));
      const first = this.inputs.length === 1;
      queueMicrotask(() => {
        if (first) {
          if (stderr) this.stderr.write(Buffer.from("private-fixture"));
          else this.send(invalid ? { ...event("connected"), extra: "private-fixture" } : event("connected"));
        } else if (!noStop) { this.send(event("closed", { reason: "requested" })); this.close(0); }
      });
      done();
    } });
  }
  send(value) { this.stdout.write(Buffer.from(JSON.stringify(value) + "\n")); }
  close(code) { this.exitCode = code; this.stdout.end(); this.stderr.end(); this.emit("close", code); }
  kill(signal) { this.kills.push(signal); this.signalCode = signal; this.close(null); return true; }
  unref() {}
}
async function fixture(options = {}) {
  const root = await mkdtemp(join(tmpdir(), "cadence-observer-test-")); await chmod(root, 0o700);
  const path = join(root, "observer"); await writeFile(path, "fixture binary", { mode: 0o700 });
  const context = { cadence_observer: { path, sha256: await fileDigest(path) } };
  const child = new Child(options); let calls = 0;
  const observer = createCadenceObserver(root, context, { spawn: (program, args, config) => {
    assert.equal(program, path); assert.deepEqual(args, []); assert.deepEqual(config.env, {}); calls += 1; return child;
  }, now: () => 10000 });
  return { root, context, child, observer, calls: () => calls, cleanup: () => rm(root, { recursive: true, force: true }) };
}
const endpoint = { ipv4: "192.168.1.2", httpPort: 80 };

test("single-use observer privately hands off endpoint and seals only metadata after actual exit", async () => {
  // Arrange
  const f = await fixture();
  try {
    // Act
    assert.deepEqual(await f.observer.start(endpoint), { observer_connected: true });
    const first = await f.observer.finish(); const second = await f.observer.finish();
    // Assert
    assert.deepEqual(first, second);
    assert.equal(first.connected, true); assert.equal(first.closed, true); assert.equal(first.exitCode, 0);
    assert.equal(first.reason, "requested"); assert.equal(first.cleanupComplete, true);
    assert.equal(first.journalSha256, await fileDigest(join(f.root, "cadence-observer.jsonl")));
    assert.equal(f.child.inputs[0].ipv4, endpoint.ipv4); assert.equal(f.child.inputs[0].expiresUnixMs, 15000);
    assert.equal(f.observer.status().alive, false);
    await assert.rejects(f.observer.start(endpoint), { code: "observer_already_used" });
    assert.equal(f.calls(), 1);
    for (const name of ["cadence-observer.jsonl", "cadence-observer-result.json"]) {
      const text = await readFile(join(f.root, name), "utf8"); assert(!text.includes(endpoint.ipv4)); assert(!text.includes("bindingVerified"));
    }
  } finally { await f.cleanup(); }
});

test("binary mismatch prevents spawn and cannot restart the same owner", async () => {
  // Arrange
  const f = await fixture(); await writeFile(f.context.cadence_observer.path, "changed binary");
  try {
    // Act / Assert
    await assert.rejects(f.observer.start(endpoint), { code: "observer_binary" });
    assert.equal(f.calls(), 0); assert.equal(f.observer.status().failed, true);
    await assert.rejects(f.observer.start(endpoint), { code: "observer_already_used" });
  } finally { await f.cleanup(); }
});

for (const [name, option] of [["unallowlisted metadata", { invalid: true }], ["stderr bytes", { stderr: true }]]) {
  test(`${name} stays private, fails sticky and reaps the owned observer`, async () => {
    // Arrange
    const f = await fixture(option);
    try {
      // Act
      await assert.rejects(f.observer.start(endpoint)); const result = await f.observer.finish();
      // Assert
      assert(f.observer.status().failed); assert(result.cleanupComplete); assert.notEqual(result.reason, "requested");
      const rows = await readFile(join(f.root, "cadence-observer.jsonl"), "utf8"); assert(!rows.includes("private-fixture"));
    } finally { await f.cleanup(); }
  });
}

test("arrival counter regression is rejected without persisting its line", async () => {
  // Arrange
  const f = await fixture();
  try {
    await f.observer.start(endpoint);
    // Act
    f.child.send(event("arrival", { messageCount: 4, byteCount: 20, totalBytes: 20 }));
    const result = await f.observer.finish();
    // Assert
    assert.equal(result.reason, "observer_event");
    const rows = (await readFile(join(f.root, "cadence-observer.jsonl"), "utf8")).trim().split("\n").map(JSON.parse);
    assert.deepEqual(rows.map(row => row.event.event), ["connected"]);
  } finally { await f.cleanup(); }
});

test("noncooperating observer is terminated and reaped with failed stop evidence", async () => {
  // Arrange
  const f = await fixture({ noStop: true });
  try {
    await f.observer.start(endpoint);
    // Act
    const result = await f.observer.finish();
    // Assert
    assert.equal(result.reason, "observer_stop_timeout"); assert.equal(result.cleanupComplete, true);
    assert.deepEqual(f.child.kills, ["SIGTERM"]); assert.equal(result.exitCode, null);
  } finally { await f.cleanup(); }
});

test("output over the line bound fails closed without persisting private bytes", async () => {
  // Arrange
  const f = await fixture();
  try {
    await f.observer.start(endpoint);
    // Act
    f.child.stdout.write(Buffer.from("private-fixture".repeat(400)));
    const result = await f.observer.finish();
    // Assert
    assert.equal(result.reason, "observer_output_bound"); assert(result.cleanupComplete);
    assert(!(await readFile(join(f.root, "cadence-observer.jsonl"), "utf8")).includes("private-fixture"));
  } finally { await f.cleanup(); }
});

test("unreaped child yields explicit failed cleanup after both termination steps", async () => {
  // Arrange
  const f = await fixture({ noStop: true });
  f.child.kill = signal => { f.child.kills.push(signal); return true; };
  try {
    await f.observer.start(endpoint);
    // Act
    const result = await f.observer.finish();
    // Assert
    assert.equal(result.cleanupComplete, false); assert.equal(result.closed, false);
    assert.notEqual(result.reason, "requested"); assert.deepEqual(f.child.kills, ["SIGTERM", "SIGKILL"]);
  } finally { await f.cleanup(); }
});

test("closing supervisor before observer use creates no failed receipt or process", async () => {
  // Arrange
  const f = await fixture();
  try {
    // Act
    const result = await f.observer.finish();
    // Assert
    assert.equal(f.calls(), 0); assert.equal(result.cleanupComplete, true); assert.equal(result.journalSha256, null);
    await assert.rejects(readFile(join(f.root, "cadence-observer-result.json")), { code: "ENOENT" });
    await assert.rejects(readFile(join(f.root, "cadence-observer.jsonl")), { code: "ENOENT" });
  } finally { await f.cleanup(); }
});
