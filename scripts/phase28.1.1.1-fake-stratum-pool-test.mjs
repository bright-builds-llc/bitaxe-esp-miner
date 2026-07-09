#!/usr/bin/env node
import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import fs from "node:fs";
import net from "node:net";
import os from "node:os";
import path from "node:path";

const repoRoot = path.resolve(path.dirname(new URL(import.meta.url).pathname), "..");
const serverScript = path.join(repoRoot, "scripts/phase28.1.1.1-fake-stratum-pool.mjs");

function readJson(filePath) {
  return JSON.parse(fs.readFileSync(filePath, "utf8"));
}

async function waitForFile(filePath) {
  for (let attempt = 0; attempt < 100; attempt += 1) {
    if (fs.existsSync(filePath)) {
      return readJson(filePath);
    }
    await new Promise((resolve) => setTimeout(resolve, 50));
  }
  throw new Error(`timed out waiting for ${filePath}`);
}

async function sendClientMessages(port) {
  return new Promise((resolve, reject) => {
    const socket = net.createConnection({ host: "127.0.0.1", port });
    const responses = [];
    let buffer = "";

    socket.setEncoding("utf8");
    socket.on("connect", () => {
      socket.write(
        [
          JSON.stringify({
            id: 1,
            method: "mining.configure",
            params: [["version-rolling"], { "version-rolling.mask": "ffffffff" }],
          }),
          JSON.stringify({
            id: 2,
            method: "mining.subscribe",
            params: ["bitaxe/ultra/205"],
          }),
          JSON.stringify({
            id: 3,
            method: "mining.authorize",
            params: ["PHASE28_SENTINEL_USER", "PHASE28_SENTINEL_PASSWORD"],
          }),
          JSON.stringify({ id: 4, method: "mining.suggest_difficulty", params: [42] }),
          JSON.stringify({
            id: 5,
            method: "mining.submit",
            params: ["PHASE28_SENTINEL_USER", "job", "00000000", "647025b5", "12345678", "0"],
          }),
        ].join("\n") + "\n",
      );
    });
    socket.on("data", (chunk) => {
      buffer += chunk;
      let newlineIndex = buffer.indexOf("\n");
      while (newlineIndex !== -1) {
        const line = buffer.slice(0, newlineIndex);
        buffer = buffer.slice(newlineIndex + 1);
        if (line.trim()) {
          responses.push(JSON.parse(line));
        }
        newlineIndex = buffer.indexOf("\n");
      }
      if (responses.some((response) => response.method === "mining.notify")) {
        socket.end();
      }
    });
    socket.on("end", () => resolve(responses));
    socket.on("error", reject);
  });
}

const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "phase28-fake-pool-"));
const readyJson = path.join(tempDir, "ready.json");
const reportJson = path.join(tempDir, "report.json");

try {
  const child = spawn(
    process.execPath,
    [
      serverScript,
      "--host",
      "127.0.0.1",
      "--port",
      "0",
      "--fixture",
      "phase28-source-work-v1",
      "--session-label",
      "upstream",
      "--ready-json",
      readyJson,
      "--report-json",
      reportJson,
      "--duration-seconds",
      "30",
    ],
    { cwd: repoRoot, encoding: "utf8" },
  );

  let stdout = "";
  let stderr = "";
  child.stdout.on("data", (chunk) => {
    stdout += chunk.toString();
  });
  child.stderr.on("data", (chunk) => {
    stderr += chunk.toString();
  });

  const ready = await waitForFile(readyJson);
  assert.equal(ready.status, "ready");
  assert.equal(ready.fixture, "phase28-source-work-v1");
  assert.match(ready.source_work_fingerprint, /^[0-9a-f]{64}$/);

  const responses = await sendClientMessages(ready.bound_port);
  assert(responses.some((response) => response.result?.["version-rolling"] === true));
  assert(responses.some((response) => Array.isArray(response.result)));
  assert(responses.some((response) => response.method === "mining.set_difficulty"));
  assert(responses.some((response) => response.method === "mining.notify"));

  child.kill("SIGTERM");
  await new Promise((resolve, reject) => {
    child.on("exit", resolve);
    child.on("error", reject);
  });

  const report = await waitForFile(reportJson);
  assert.equal(report.status, "stopped");
  assert.equal(report.configure_observed, true);
  assert.equal(report.subscribe_observed, true);
  assert.equal(report.authorize_observed, true);
  assert.equal(report.submit_observed, true);
  assert.equal(report.notify_sent_count, 1);
  assert.equal(report.source_work_fingerprint, ready.source_work_fingerprint);

  const combinedOutput = `${stdout}\n${stderr}\n${JSON.stringify(report)}`;
  assert(!combinedOutput.includes("PHASE28_SENTINEL_PASSWORD"));
  assert(!combinedOutput.includes("PHASE28_SENTINEL_USER"));

  console.log("phase28.1.1.1 fake Stratum pool tests passed");
} finally {
  fs.rmSync(tempDir, { recursive: true, force: true });
}
