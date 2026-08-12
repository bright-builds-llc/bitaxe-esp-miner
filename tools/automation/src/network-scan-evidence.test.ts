import assert from "node:assert/strict";
import { chmod, mkdtemp, mkdir, readFile, stat, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import {
  captureNetworkScanEvidence,
  NetworkScanEvidenceError,
  stationAddressKind,
} from "./network-scan-evidence.js";
import { createFakeProcessPort, createLocalProcessPort, type ProcessOutcome } from "./process.js";

const sourceCommit = "a".repeat(40);
const referenceCommit = "b".repeat(40);
const appElfSha256 = "c".repeat(64);
const session = "1".repeat(32);
const ok = (stdout = ""): ProcessOutcome => ({ exitCode: 0, stdout, stderr: "", timedOut: false });

function monitorLog(): string {
  return [
    "safe_state: mining=disabled asic_work_submission=disabled hardware_control=disabled",
    `runtime_boot_attestation schema_version=1 session=${session} boot_ordinal=7 reset_reason=other uptime_ms=500 board=205 asic=BM1366 mining=disabled work_submission=disabled hardware_control=disabled firmware_commit=${sourceCommit} reference_commit=${referenceCommit} app_elf_sha256=${appElfSha256} esp_idf_version=v5.5.4 ota_boot_validation=complete spiffs_mount=available api_route_shell=started redacted=true`,
    `runtime_origin session=${session} device_url=http://private-device.test redacted=true`,
  ].join("\n") + "\n";
}

async function fixture(name: string) {
  const root = await mkdtemp(path.join(os.tmpdir(), `bitaxe-network-scan-${name}-`));
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
    projection: path.join(root, "docs", "network-scan.json"),
    options: {
      privateRoot: "scratch/attempt",
      packageManifest: manifest,
      wifiCredentials: credentials,
      port: "/dev/private-port",
      projection: path.join(root, "docs", "network-scan.json"),
      captureTimeoutSeconds: 240,
    },
  };
}

function systemInfo(uptimeSeconds: number, ipv6 = "fe80::1234%7") {
  return {
    bootSession: session,
    wifiStatus: "connected",
    apEnabled: 0,
    sourceCommit,
    referenceCommit,
    appElfSha256,
    uptimeSeconds,
    ipv6,
  };
}

function installHttp(options: {
  readonly maybeAddress?: string;
  readonly emptyScan?: boolean;
  readonly failAfter?: boolean;
} = {}) {
  const original = globalThis.fetch;
  let systemCalls = 0;
  const routes: string[] = [];
  globalThis.fetch = async (input) => {
    const url = new URL(String(input));
    routes.push(url.pathname);
    if (url.pathname === "/api/system/wifi/scan") {
      return new Response(JSON.stringify({
        networks: options.emptyScan === true ? [] : [
          { ssid: "private-nearby-network", rssi: -42, authmode: 3 },
          { ssid: "", rssi: -71, authmode: 9 },
        ],
      }), { status: 200, headers: { "content-type": "application/json" } });
    }
    systemCalls += 1;
    if (options.failAfter === true && systemCalls === 2) return new Response("unavailable", { status: 503 });
    return new Response(JSON.stringify(systemInfo(systemCalls + 10, options.maybeAddress)), {
      status: 200,
      headers: { "content-type": "application/json" },
    });
  };
  return { routes, restore: () => { globalThis.fetch = original; } };
}

function fakePort(recoveryExit = 0) {
  let recoveryCount = 0;
  const port = createFakeProcessPort(async (spec) => {
    if (spec.args[0] === "flash-monitor") {
      const index = spec.args.indexOf("--evidence-dir");
      const evidenceDir = spec.args[index + 1];
      assert.ok(evidenceDir !== undefined);
      await writeFile(path.join(evidenceDir, "flash-monitor.classifier-input.log"), monitorLog(), { mode: 0o600 });
      return ok();
    }
    if (spec.args[0] === "flash") {
      recoveryCount += 1;
      return { ...ok(), exitCode: recoveryExit };
    }
    return ok();
  });
  return { port, recoveryCount: () => recoveryCount };
}

async function captureError(promise: Promise<unknown>): Promise<NetworkScanEvidenceError> {
  try {
    await promise;
    assert.fail("expected capture failure");
  } catch (error) {
    assert.ok(error instanceof NetworkScanEvidenceError);
    return error;
  }
}

test("station address classification admits only reportable v6 forms", () => {
  // Arrange / Act / Assert
  assert.equal(stationAddressKind("fe80::1234%7"), "link_local");
  assert.equal(stationAddressKind("fd00::1234"), "unique_local");
  assert.equal(stationAddressKind("2001:db8::1234"), "global");
  assert.throws(() => stationAddressKind("fd00::1234%7"), NetworkScanEvidenceError);
  assert.throws(() => stationAddressKind("::1"), NetworkScanEvidenceError);
});

test("one live-shaped scan emits aggregate-only same-session evidence", async () => {
  // Arrange
  const value = await fixture("ready");
  const http = installHttp();
  const process = fakePort();

  try {
    // Act
    const evidence = await captureNetworkScanEvidence(
      value.root,
      value.options,
      process.port,
      "flash",
      "validator",
    );

    // Assert
    assert.equal(evidence.scan.record_count, 2);
    assert.equal(evidence.scan.address_kind, "link_local");
    assert.deepEqual(http.routes, ["/api/system/info", "/api/system/wifi/scan", "/api/system/info"]);
    assert.equal(process.recoveryCount(), 0);
    const projection = await readFile(value.projection, "utf8");
    assert.doesNotMatch(
      projection,
      /private-nearby-network|private-device|private-port|device_url|ssid|ipv[46]|fe80|hostname|credential/iu,
    );
    assert.equal((await stat(path.join(value.root, "scratch", "attempt"))).mode & 0o777, 0o700);
    assert.equal(
      (await stat(path.join(value.root, "scratch", "attempt", "scan.private.json"))).mode & 0o777,
      0o600,
    );
  } finally {
    http.restore();
  }
});

test("environmental scan and address gaps stay typed and withhold evidence", async () => {
  for (const testCase of [
    { name: "empty", http: { emptyScan: true }, category: "hardware_blocked" },
    { name: "address", http: { maybeAddress: "" }, category: "hardware_blocked" },
  ] as const) {
    // Arrange
    const value = await fixture(testCase.name);
    const http = installHttp(testCase.http);
    const process = fakePort();
    try {
      // Act
      const error = await captureError(captureNetworkScanEvidence(
        value.root,
        value.options,
        process.port,
        "flash",
        "validator",
      ));

      // Assert
      assert.equal(error.category, testCase.category);
      assert.equal(error.publicValue["recovery_complete"], true);
      assert.equal(process.recoveryCount(), 1);
      await assert.rejects(readFile(value.projection, "utf8"), { code: "ENOENT" });
    } finally {
      http.restore();
    }
  }
});

test("primary service failure survives a failed recovery", async () => {
  // Arrange
  const value = await fixture("recovery-precedence");
  const http = installHttp({ failAfter: true });
  const process = fakePort(1);
  try {
    // Act
    const error = await captureError(captureNetworkScanEvidence(
      value.root,
      value.options,
      process.port,
      "flash",
      "validator",
    ));

    // Assert
    assert.equal(error.category, "service_recovery_failed");
    assert.equal(error.publicValue["secondary_recovery_failure"], true);
    assert.equal(process.recoveryCount(), 1);
  } finally {
    http.restore();
  }
});

test("a real child must create the production monitor artifact", async () => {
  // Arrange
  const value = await fixture("real-child");
  const http = installHttp();
  const child = path.join(value.root, "child.sh");
  await writeFile(child, `#!/bin/sh
if [ "$1" = "flash-monitor" ]; then
  while [ "$1" != "--evidence-dir" ]; do shift; done
  shift
  printf '%s' '${monitorLog().replaceAll("'", "'\\''")}' > "$1/flash-monitor.classifier-input.log"
  chmod 600 "$1/flash-monitor.classifier-input.log"
fi
`);
  await chmod(child, 0o700);
  try {
    // Act
    const evidence = await captureNetworkScanEvidence(
      value.root,
      value.options,
      createLocalProcessPort({ cwd: value.root, timeoutMs: 5_000 }),
      child,
      child,
    );

    // Assert
    assert.equal(evidence.schema_version, "bitaxe-network-scan-evidence-v1");
  } finally {
    http.restore();
  }
});
