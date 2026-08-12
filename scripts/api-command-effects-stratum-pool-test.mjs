#!/usr/bin/env node
import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import fs from "node:fs";
import net from "node:net";
import os from "node:os";
import path from "node:path";

const repoRoot = path.resolve(path.dirname(new URL(import.meta.url).pathname), "..");
const serverScript = path.join(repoRoot, "scripts", "api-command-effects-stratum-pool.mjs");

async function waitForJson(input) {
  for (let attempt = 0; attempt < 100; attempt += 1) {
    if (fs.existsSync(input)) return JSON.parse(fs.readFileSync(input, "utf8"));
    await new Promise((resolve) => setTimeout(resolve, 50));
  }
  throw new Error("fixture file timeout");
}

async function exercise(port) {
  return new Promise((resolve, reject) => {
    const socket = net.createConnection({ host: "127.0.0.1", port });
    const responses = [];
    let buffer = "";
    socket.setEncoding("utf8");
    socket.on("connect", () => socket.write([
      JSON.stringify({ id: 1, method: "mining.configure", params: [] }),
      JSON.stringify({ id: 2, method: "mining.subscribe", params: [] }),
      JSON.stringify({ id: 3, method: "mining.authorize", params: ["PRIVATE_USER", "PRIVATE_PASSWORD"] }),
      JSON.stringify({ id: 4, method: "mining.submit", params: ["PRIVATE_USER"] }),
    ].join("\n") + "\n"));
    socket.on("data", (chunk) => {
      buffer += chunk;
      let newline = buffer.indexOf("\n");
      while (newline !== -1) {
        const line = buffer.slice(0, newline).trim();
        buffer = buffer.slice(newline + 1);
        if (line) responses.push(JSON.parse(line));
        newline = buffer.indexOf("\n");
      }
      if (responses.some((response) => response.method === "mining.notify")) socket.end();
    });
    socket.on("end", () => resolve(responses));
    socket.on("error", reject);
  });
}

const tempRoot = fs.mkdtempSync(path.join(os.tmpdir(), "api-command-fixture-"));
const ready = path.join(tempRoot, "ready.json");
const report = path.join(tempRoot, "report.json");
const stop = path.join(tempRoot, "stop");

try {
  const child = spawn(process.execPath, [
    serverScript,
    "--host", "127.0.0.1",
    "--port", "0",
    "--fixture", "api-command-effects-v1",
    "--session-label", "command-effects",
    "--ready-json", ready,
    "--report-json", report,
    "--duration-seconds", "30",
    "--stop-file", stop,
  ], { cwd: repoRoot, encoding: "utf8" });
  let output = "";
  child.stdout.on("data", (chunk) => { output += chunk.toString(); });
  child.stderr.on("data", (chunk) => { output += chunk.toString(); });
  const readiness = await waitForJson(ready);
  assert.equal(fs.statSync(ready).mode & 0o777, 0o600);
  const responses = await exercise(readiness.bound_port);
  assert.deepEqual(
    responses.find((response) => response.method === "mining.set_difficulty")?.params,
    [1],
  );
  const notification = responses.find((response) => response.method === "mining.notify");
  assert.equal(notification?.params?.[6], "207fffff");
  assert.equal(notification?.params?.[8], true);
  fs.writeFileSync(stop, "stop\n", { mode: 0o600 });
  await new Promise((resolve, reject) => {
    child.on("exit", resolve);
    child.on("error", reject);
  });
  const result = await waitForJson(report);
  assert.equal(result.submit_observed, true);
  assert.equal(result.accepted_submit_count, 1);
  assert.equal(result.raw_messages_committed, false);
  assert.equal(fs.statSync(report).mode & 0o777, 0o600);
  const publicSurface = `${output}\n${JSON.stringify(result)}`;
  assert(!publicSurface.includes("PRIVATE_USER"));
  assert(!publicSurface.includes("PRIVATE_PASSWORD"));
  process.stdout.write("api command-effects Stratum fixture tests passed\n");
} finally {
  fs.rmSync(tempRoot, { recursive: true, force: true });
}
