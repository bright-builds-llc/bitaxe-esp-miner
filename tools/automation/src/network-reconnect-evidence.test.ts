import assert from "node:assert/strict";
import { chmod, mkdtemp, mkdir, readFile, stat, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import {
  captureNetworkReconnectEvidence,
  NetworkReconnectEvidenceError,
} from "./network-reconnect-evidence.js";
import { createFakeProcessPort, createLocalProcessPort, type ProcessOutcome } from "./process.js";

const sourceCommit = "a".repeat(40);
const referenceCommit = "b".repeat(40);
const appElfSha256 = "c".repeat(64);
const session = "1".repeat(32);
const nodeProgram = process.env["JS_BINARY__NODE_BINARY"] ?? process.execPath;

const ok = (stdout = ""): ProcessOutcome => ({ exitCode: 0, stdout, stderr: "", timedOut: false });

function monitorLog(attemptUptime = 8_000): string {
  return [
    "safe_state: mining=disabled asic_work_submission=disabled hardware_control=disabled",
    `runtime_boot_attestation schema_version=1 session=${session} boot_ordinal=7 reset_reason=other uptime_ms=500 board=205 asic=BM1366 mining=disabled work_submission=disabled hardware_control=disabled firmware_commit=${sourceCommit} reference_commit=${referenceCommit} app_elf_sha256=${appElfSha256} esp_idf_version=v5.5.4 ota_boot_validation=complete spiffs_mount=available api_route_shell=started redacted=true`,
    `runtime_origin session=${session} device_url=http://private-device.test redacted=true`,
    "wifi_reconnect_probe=armed uptime_ms=1000",
    "wifi_reconnect=disconnected reason=other retry_ordinal=1 fallback=true retry_delay_ms=5000 uptime_ms=3000",
    `wifi_reconnect=attempt_started retry_ordinal=1 uptime_ms=${String(attemptUptime)}`,
    "wifi_reconnect=connected completed_retry_ordinal=1 retry_ordinal=0 fallback=false uptime_ms=9000",
    "wifi_reconnect_probe=recovered completed_retry_ordinal=1 uptime_ms=9028",
    "wifi_reconnect_probe=stable completed_retry_ordinal=1 stability_ms=15000 uptime_ms=24054",
  ].join("\n") + "\n";
}

async function fixture(name: string) {
  const root = await mkdtemp(path.join(os.tmpdir(), `bitaxe-network-reconnect-${name}-`));
  await writeFile(path.join(root, "MODULE.bazel"), "module(name = \"fixture\")\n");
  await mkdir(path.join(root, "inputs"));
  const manifest = path.join(root, "inputs", "package.json");
  const credentials = path.join(root, "inputs", "wifi.json");
  await writeFile(manifest, JSON.stringify({
    source_commit: sourceCommit,
    reference_commit: referenceCommit,
    app_elf_sha256: appElfSha256,
  }));
  await writeFile(credentials, "{}\n", { mode: 0o600 });
  return {
    root,
    projection: path.join(root, "docs", "network-reconnect.json"),
    options: {
      privateRoot: "scratch/attempt",
      packageManifest: manifest,
      wifiCredentials: credentials,
      port: "/dev/private-port",
      projection: path.join(root, "docs", "network-reconnect.json"),
      captureTimeoutSeconds: 90,
    },
  };
}

function installHttp() {
  const original = globalThis.fetch;
  globalThis.fetch = async () => new Response(JSON.stringify({
    bootSession: session,
    wifiStatus: "connected",
    apEnabled: 0,
    sourceCommit,
    referenceCommit,
    appElfSha256,
  }), { status: 200, headers: { "content-type": "application/json" } });
  return () => { globalThis.fetch = original; };
}

async function captureError(promise: Promise<unknown>): Promise<NetworkReconnectEvidenceError> {
  try {
    await promise;
    assert.fail("expected capture failure");
  } catch (error) {
    assert.ok(error instanceof NetworkReconnectEvidenceError);
    return error;
  }
}

test("ready stdout lifecycle emits aggregate-only reconnect evidence", async () => {
  const value = await fixture("ready");
  const restore = installHttp();
  const commands: string[] = [];
  const port = createFakeProcessPort(async (spec) => {
    commands.push(spec.program);
    if (spec.args[0] === "flash-monitor") return ok(monitorLog());
    return ok();
  });
  try {
    const evidence = await captureNetworkReconnectEvidence(
      value.root,
      value.options,
      port,
      "flash",
      "validator",
    );
    const projection = await readFile(value.projection, "utf8");
    assert.equal(evidence.reconnect.observed_retry_delay_ms, 5_000);
    assert.deepEqual(commands, ["flash", "validator"]);
    assert.doesNotMatch(projection, /private-device|private-port|device_url|hostname|ssid|credential/u);
    assert.equal((await stat(path.join(value.root, "scratch", "attempt"))).mode & 0o777, 0o700);
    assert.equal(
      (await stat(path.join(value.root, "scratch", "attempt", "flash-monitor.private.log"))).mode & 0o777,
      0o600,
    );
  } finally {
    restore();
  }
});

test("missing lifecycle and early retry preserve typed failure through recovery", async () => {
  for (const testCase of [
    { name: "missing", log: monitorLog().replace(/wifi_reconnect_probe=stable.*\n/u, ""), category: "reconnect_not_observed" },
    { name: "early", log: monitorLog(7_999), category: "reconnect_timing_invalid" },
  ] as const) {
    const value = await fixture(testCase.name);
    const restore = installHttp();
    let recoveryCount = 0;
    const port = createFakeProcessPort(async (spec) => {
      if (spec.args[0] === "flash-monitor") return ok(testCase.log);
      if (spec.args[0] === "flash") {
        recoveryCount += 1;
        assert.ok(!spec.args.includes("--network-reconnect-probe"));
        return ok();
      }
      return ok();
    });
    try {
      const error = await captureError(captureNetworkReconnectEvidence(
        value.root,
        value.options,
        port,
        "flash",
        "validator",
      ));
      assert.equal(error.category, testCase.category);
      assert.equal(error.publicValue["recovery_complete"], true);
      assert.equal(recoveryCount, 1);
      await assert.rejects(readFile(value.projection, "utf8"), { code: "ENOENT" });
    } finally {
      restore();
    }
  }
});

test("real child stdout is consumed without an invented monitor artifact", async () => {
  const value = await fixture("real-child");
  const restore = installHttp();
  const child = path.join(value.root, "child.mjs");
  await writeFile(child, `#!${nodeProgram}\nconst args=process.argv.slice(2); if(args[0]==="flash-monitor") process.stdout.write(${JSON.stringify(monitorLog())});\n`);
  await chmod(child, 0o700);
  try {
    const evidence = await captureNetworkReconnectEvidence(
      value.root,
      value.options,
      createLocalProcessPort({ cwd: value.root, timeoutMs: 5_000 }),
      child,
      child,
    );
    assert.equal(evidence.schema_version, "bitaxe-network-reconnect-evidence-v1");
    await assert.rejects(
      readFile(path.join(value.root, "scratch", "attempt", "flash-monitor.log"), "utf8"),
      { code: "ENOENT" },
    );
  } finally {
    restore();
  }
});
