import assert from "node:assert/strict";
import { mkdtemp, mkdir, readFile, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import { createFakeProcessPort, type ProcessOutcome } from "./process.js";
import { captureSettingsDurability } from "./settings-durability.js";

const ok = (stdout = ""): ProcessOutcome => ({ exitCode: 0, stdout, stderr: "", timedOut: false });

test("settings durability capture restores the private original hostname and emits closed facts", async () => {
  // Arrange
  const root = await mkdtemp(path.join(os.tmpdir(), "bitaxe-settings-"));
  await writeFile(path.join(root, "MODULE.bazel"), "module(name = \"fixture\")\n");
  await mkdir(path.join(root, "inputs"));
  const manifest = path.join(root, "inputs", "package.json");
  const credentials = path.join(root, "inputs", "wifi.json");
  await writeFile(manifest, JSON.stringify({ source_commit: "a".repeat(40), reference_commit: "b".repeat(40) }));
  await writeFile(credentials, "{}\n");
  let currentHostname = "private-original";
  const originalFetch = globalThis.fetch;
  globalThis.fetch = async (input, init) => {
    const target = new URL(String(input));
    if (init?.method === "PATCH") {
      const body = JSON.parse(String(init.body)) as { hostname: string };
      currentHostname = body.hostname;
      return new Response("", { status: 200 });
    }
    if (init?.method === "POST" && target.pathname.endsWith("/restart")) {
      return new Response("{}", { status: 200 });
    }
    return new Response(JSON.stringify({ hostname: currentHostname }), {
      status: 200,
      headers: { "content-type": "application/json" },
    });
  };
  const trace = "safe_state: mining=disabled asic_work_submission=disabled hardware_control=disabled\nruntime_origin session=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa device_url=http://127.0.0.1:8080 redacted=true\n";
  const processPort = createFakeProcessPort(async (spec) => {
    if (spec.args[0] === "flash-monitor") {
      const evidenceIndex = spec.args.indexOf("--evidence-dir");
      const evidenceRoot = spec.args[evidenceIndex + 1];
      assert.notEqual(evidenceRoot, undefined);
      await writeFile(path.join(String(evidenceRoot), "flash-monitor.classifier-input.log"), trace);
      return ok();
    }
    if (spec.args[0] === "monitor") {
      const evidenceIndex = spec.args.indexOf("--evidence-dir");
      const evidenceRoot = spec.args[evidenceIndex + 1];
      assert.notEqual(evidenceRoot, undefined);
      await writeFile(path.join(String(evidenceRoot), "flash-monitor.log"), trace);
      return ok();
    }
    if (spec.args[0] === "verify-settings-durability") {
      const mode = spec.args[spec.args.indexOf("--mode") + 1];
      return ok(JSON.stringify({
        status: "passed",
        category: "none",
        session: mode === "baseline" ? "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" : "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        boot_ordinal: mode === "baseline" ? 4 : 5,
      }));
    }
    throw new Error(`unexpected process ${spec.args.join(" ")}`);
  });
  const projection = path.join(root, "docs", "projection.json");

  try {
    // Act
    await captureSettingsDurability(root, {
      privateRoot: "scratch/attempt",
      packageManifest: manifest,
      wifiCredentials: credentials,
      port: "/dev/test",
      projection,
      captureTimeoutSeconds: 360,
    }, processPort, "flash", "classifier", async () => undefined);

    // Assert
    const publicDocument = await readFile(projection, "utf8");
    assert.equal(currentHostname, "private-original");
    assert.doesNotMatch(publicDocument, /private-original|bitaxe-parity-205/u);
    assert.equal(JSON.parse(publicDocument).post_restart_persistence, true);
    assert.equal(JSON.parse(publicDocument).restoration_complete, true);
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("settings durability capture restores the original hostname after a persistence mismatch", async () => {
  // Arrange
  const root = await mkdtemp(path.join(os.tmpdir(), "bitaxe-settings-failure-"));
  await writeFile(path.join(root, "MODULE.bazel"), "module(name = \"fixture\")\n");
  await mkdir(path.join(root, "inputs"));
  const manifest = path.join(root, "inputs", "package.json");
  const credentials = path.join(root, "inputs", "wifi.json");
  await writeFile(manifest, JSON.stringify({ source_commit: "a".repeat(40), reference_commit: "b".repeat(40) }));
  await writeFile(credentials, "{}\n");
  let currentHostname = "private-original";
  let restarted = false;
  const originalFetch = globalThis.fetch;
  globalThis.fetch = async (input, init) => {
    const target = new URL(String(input));
    if (init?.method === "PATCH") {
      currentHostname = (JSON.parse(String(init.body)) as { hostname: string }).hostname;
      return new Response("", { status: 200 });
    }
    if (init?.method === "POST" && target.pathname.endsWith("/restart")) {
      restarted = true;
      return new Response("{}", { status: 200 });
    }
    const observed = restarted && currentHostname !== "private-original" ? "persistence-mismatch" : currentHostname;
    return new Response(JSON.stringify({ hostname: observed }), {
      status: 200,
      headers: { "content-type": "application/json" },
    });
  };
  const trace = "safe_state: mining=disabled asic_work_submission=disabled hardware_control=disabled\nruntime_origin session=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa device_url=http://127.0.0.1:8080 redacted=true\n";
  const processPort = createFakeProcessPort(async (spec) => {
    const evidenceIndex = spec.args.indexOf("--evidence-dir");
    if (spec.args[0] === "flash-monitor" && evidenceIndex >= 0) {
      await writeFile(path.join(String(spec.args[evidenceIndex + 1]), "flash-monitor.classifier-input.log"), trace);
      return ok();
    }
    if (spec.args[0] === "monitor" && evidenceIndex >= 0) {
      await writeFile(path.join(String(spec.args[evidenceIndex + 1]), "flash-monitor.log"), trace);
      return ok();
    }
    if (spec.args[0] === "verify-settings-durability") {
      return ok(JSON.stringify({ status: "passed", session: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", boot_ordinal: 4 }));
    }
    throw new Error(`unexpected process ${spec.args.join(" ")}`);
  });

  try {
    // Act
    await assert.rejects(captureSettingsDurability(root, {
      privateRoot: "scratch/attempt",
      packageManifest: manifest,
      wifiCredentials: credentials,
      port: "/dev/test",
      projection: path.join(root, "docs", "projection.json"),
      captureTimeoutSeconds: 360,
    }, processPort, "flash", "classifier", async () => undefined), /post-restart hostname readback mismatch/u);

    // Assert
    assert.equal(currentHostname, "private-original");
  } finally {
    globalThis.fetch = originalFetch;
  }
});
