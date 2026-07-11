#!/usr/bin/env node

import assert from "node:assert/strict";
import { spawn, spawnSync } from "node:child_process";
import fs from "node:fs";
import net from "node:net";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const frameHelper = path.join(
  repoRoot,
  "scripts/phase28.1.1-lifecycle-frame.pl",
);
const tempRoot = fs.mkdtempSync("/tmp/phase28-frame-");
const activeChildren = new Set();

function delay(milliseconds) {
  return new Promise((resolve) => setTimeout(resolve, milliseconds));
}

function childResult(child) {
  let stdout = "";
  let stderr = "";
  child.stdout?.setEncoding("utf8");
  child.stderr?.setEncoding("utf8");
  child.stdout?.on("data", (chunk) => {
    stdout += chunk;
  });
  child.stderr?.on("data", (chunk) => {
    stderr += chunk;
  });

  return new Promise((resolve, reject) => {
    child.once("error", reject);
    child.once("close", (code, signal) => resolve({ code, signal, stderr, stdout }));
  });
}

async function waitForSocket(socketPath, receiverResult) {
  for (let attempt = 0; attempt < 200; attempt += 1) {
    try {
      if (fs.lstatSync(socketPath).isSocket()) {
        return;
      }
    } catch (error) {
      if (error.code !== "ENOENT") {
        throw error;
      }
    }

    const maybeFinished = await Promise.race([
      receiverResult.then((result) => ({ result })),
      delay(5).then(() => null),
    ]);
    if (maybeFinished) {
      assert.fail(`receiver exited before creating socket: ${maybeFinished.result.stderr}`);
    }
  }

  assert.fail("receiver did not create its Unix socket");
}

function startReceiver(caseName) {
  const caseRoot = path.join(tempRoot, caseName);
  fs.mkdirSync(caseRoot, { mode: 0o700 });
  const socketPath = path.join(caseRoot, "lifecycle.sock");
  const outputPath = path.join(caseRoot, "frame.json");
  const child = spawn(
    "perl",
    [frameHelper, "receive", "--socket", "lifecycle.sock", "--output", "frame.json"],
    { cwd: caseRoot, stdio: ["ignore", "pipe", "pipe"] },
  );
  activeChildren.add(child);
  const result = childResult(child).finally(() => activeChildren.delete(child));
  return { caseRoot, child, outputPath, result, socketPath };
}

async function writeChunks(socketPath, chunks) {
  await new Promise((resolve, reject) => {
    const socket = net.createConnection(socketPath);
    let settled = false;

    const finish = (error) => {
      if (settled) {
        return;
      }
      settled = true;
      if (error && !["EPIPE", "ECONNRESET"].includes(error.code)) {
        reject(error);
        return;
      }
      resolve();
    };

    socket.once("error", finish);
    socket.once("connect", async () => {
      try {
        for (const { bytes, maybeDelayMs = 0 } of chunks) {
          if (maybeDelayMs > 0) {
            await delay(maybeDelayMs);
          }
          if (!socket.destroyed) {
            socket.write(bytes);
          }
        }
        socket.end(() => socket.destroy());
      } catch (error) {
        socket.destroy();
        finish(error);
      }
    });
    socket.once("close", () => finish());
  });
}

async function runRawFrameCase(caseName, chunks, maybeExpectedPayload) {
  const receiver = startReceiver(caseName);
  await waitForSocket(receiver.socketPath, receiver.result);
  assert.equal(fs.statSync(receiver.socketPath).mode & 0o777, 0o600);

  await writeChunks(receiver.socketPath, chunks);
  const result = await receiver.result;
  if (maybeExpectedPayload === undefined) {
    assert.notEqual(result.code, 0, `${caseName} unexpectedly succeeded`);
    assert.equal(fs.existsSync(receiver.outputPath), false);
    assert.match(result.stderr, /^lifecycle_frame_error=invalid_frame\n$/);
    return;
  }

  assert.equal(result.code, 0, `${caseName} failed: ${result.stderr}`);
  assert.equal(fs.readFileSync(receiver.outputPath, "utf8"), maybeExpectedPayload);
  assert.equal(fs.statSync(receiver.outputPath).mode & 0o777, 0o600);
}

async function runRepoSenderCase(caseName, payload) {
  const receiver = startReceiver(caseName);
  await waitForSocket(receiver.socketPath, receiver.result);
  const sender = spawn(
    "perl",
    [frameHelper, "send", "--socket", "lifecycle.sock"],
    { cwd: receiver.caseRoot, stdio: ["pipe", "pipe", "pipe"] },
  );
  activeChildren.add(sender);
  const senderResult = childResult(sender).finally(() => activeChildren.delete(sender));
  sender.stdin.end(payload);

  const [send, receive] = await Promise.all([senderResult, receiver.result]);
  assert.equal(send.code, 0, `${caseName} sender failed: ${send.stderr}`);
  assert.equal(receive.code, 0, `${caseName} receiver failed: ${receive.stderr}`);
  return fs.readFileSync(receiver.outputPath, "utf8");
}

try {
  const coalescedPayload = '{"response_token":"plan13-both-power-paths-removed"}';
  await runRawFrameCase(
    "coalesced-header-and-payload",
    [{ bytes: `${Buffer.byteLength(coalescedPayload)}\n${coalescedPayload}` }],
    coalescedPayload,
  );

  const headerFragmentPayload = '{"checkpoint_generation":7}';
  const header = `${Buffer.byteLength(headerFragmentPayload)}\n`;
  await runRawFrameCase(
    "header-fragmented-bytewise",
    [
      ...[...header].map((bytes) => ({ bytes, maybeDelayMs: 2 })),
      { bytes: headerFragmentPayload },
    ],
    headerFragmentPayload,
  );

  const payloadFragment = '{"lifecycle_lease_id":"0123456789abcdef0123456789abcdef"}';
  await runRawFrameCase(
    "payload-fragmented-bytewise",
    [
      { bytes: `${Buffer.byteLength(payloadFragment)}\n` },
      ...[...payloadFragment].map((bytes) => ({ bytes })),
    ],
    payloadFragment,
  );

  const delayedPayload = '{"checkpoint_token":"plan13-barrel-usb-restore-v1"}';
  await runRawFrameCase(
    "delayed-header-and-payload-fragments",
    [
      { bytes: `${Buffer.byteLength(delayedPayload)}`, maybeDelayMs: 15 },
      { bytes: "\n", maybeDelayMs: 15 },
      { bytes: delayedPayload.slice(0, 7), maybeDelayMs: 15 },
      { bytes: delayedPayload.slice(7), maybeDelayMs: 15 },
    ],
    delayedPayload,
  );

  await runRawFrameCase("eof-after-header", [{ bytes: "10\n" }]);
  await runRawFrameCase("eof-mid-body", [{ bytes: "10\n{}" }]);
  await runRawFrameCase("oversized-length", [{ bytes: "4097\n{}" }]);
  await runRawFrameCase("header-without-newline", [{ bytes: "12345" }]);

  for (const malformedHeader of ["0", "01", "+1", "-1", " 1", "1 ", "1\r", "x1"]) {
    const caseName = `malformed-header-${Buffer.from(malformedHeader).toString("hex")}`;
    await runRawFrameCase(caseName, [{ bytes: `${malformedHeader}\n{}` }]);
  }

  await runRawFrameCase("invalid-json", [{ bytes: "1\n{" }]);
  await runRawFrameCase("extra-byte", [{ bytes: "2\n{}x" }]);
  await runRawFrameCase("two-frames", [{ bytes: "2\n{}2\n{}" }]);

  const acceptedTokens = [];
  for (const [name, responseToken] of [
    ["removal-token", "plan13-both-power-paths-removed"],
    ["restore-token", "plan13-barrel-then-usb-restored"],
  ]) {
    const payload = JSON.stringify({ response_token: responseToken });
    const accepted = await runRepoSenderCase(name, payload);
    acceptedTokens.push(JSON.parse(accepted).response_token);
  }
  assert.deepEqual(acceptedTokens, [
    "plan13-both-power-paths-removed",
    "plan13-barrel-then-usb-restored",
  ]);
  assert.equal(new Set(acceptedTokens).size, 2);

  const pathLimitRoot = path.join(tempRoot, "path-limit");
  fs.mkdirSync(pathLimitRoot, { mode: 0o700 });
  const overlongSocketName = "s".repeat(104);
  const pathLimit = spawnSync(
    "perl",
    [frameHelper, "receive", "--socket", overlongSocketName, "--output", "frame.json"],
    { cwd: pathLimitRoot, encoding: "utf8" },
  );
  assert.notEqual(pathLimit.status, 0);
  assert.equal(pathLimit.stderr, "lifecycle_frame_error=socket_path_invalid\n");
  assert.equal(fs.existsSync(path.join(pathLimitRoot, overlongSocketName)), false);

  process.stdout.write("phase28.1.1 lifecycle frame tests: passed\n");
} finally {
  for (const child of activeChildren) {
    child.kill("SIGTERM");
  }
  fs.rmSync(tempRoot, { force: true, recursive: true });
}
