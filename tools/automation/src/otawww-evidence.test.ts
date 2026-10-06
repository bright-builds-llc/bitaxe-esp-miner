import assert from "node:assert/strict";
import { chmod, mkdir, mkdtemp, readFile, writeFile } from "node:fs/promises";
import http from "node:http";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import {
  captureOtawwwEvidence, OtawwwEvidenceError, postBinaryOnce, type OtawwwEvidenceOptions, type OtawwwPhase, type OtawwwTiming,
} from "./otawww-evidence.js";
import {
  isRecoveryPage, parseOtawwwInputs, parseStationEndpoint, privateIpv4, settingsDigest, sha256, OtawwwInputError,
} from "./otawww-evidence-model.js";
import { createFakeProcessPort, type ProcessOutcome, type ProcessPort } from "./process.js";
import { verifySemanticEvidenceRedaction } from "./redaction.js";

const sourceCommit = "a".repeat(40);
const referenceCommit = "b".repeat(40);
const app = "c".repeat(64);
const buildLabel = "abcdefabcdef-dev";
const probeLabel = "abcdefabcdef-www-probe";
const safeState = "safe_state: mining=disabled asic_work_submission=disabled hardware_control=disabled";
const indexHtml = "<!doctype html><html><body>operator</body></html>";
const recoveryHtml = "<!doctype html><html><body><script>post('/api/system/OTAWWW')</script></body></html>";
const packageWww = Buffer.alloc(8_192, 0x11);
const probeWww = Buffer.alloc(8_192, 0x22);
const fastTiming: OtawwwTiming = {
  interruptionPollCount: 20, interruptionPollDelayMs: 25, baselineRetryDelayMs: 10, now: Date.now,
  admitAddress: (ipv4) => ipv4 === "127.0.0.1",
};
const ok = (): ProcessOutcome => ({ exitCode: 0, stdout: "", stderr: "", timedOut: false });

type Assets = "package" | "probe" | "erased";

type DeviceBehavior = {
  readonly ignoreInterruption?: boolean;
  readonly rejectUploads?: boolean;
  readonly changeSettingsOnRestart?: boolean;
};

/** A local stand-in for the device's HTTP API, static files and OTAWWW handler. */
class FakeDevice {
  public assets: Assets = "package";
  public pending: Assets = "package";
  public ordinal = 7;
  public session = "1".repeat(32);
  public stratumUser = "private-worker";
  public uploads = 0;
  public logLines: string[] = [safeState];
  public readonly server: http.Server;

  public constructor(private readonly behavior: DeviceBehavior = {}) {
    this.server = http.createServer((request, response) => this.handle(request, response));
  }

  public async listen(): Promise<string> {
    await new Promise<void>((resolve) => this.server.listen(0, "127.0.0.1", resolve));
    const address = this.server.address() as { port: number };
    return `http://127.0.0.1:${String(address.port)}`;
  }

  public async close(): Promise<void> {
    this.server.closeAllConnections();
    await new Promise<void>((resolve) => this.server.close(() => resolve()));
  }

  public restart(): void {
    this.assets = this.pending;
    this.ordinal += 1;
    this.session = String(this.ordinal % 10).repeat(32);
    this.logLines = [safeState];
    if (this.behavior.changeSettingsOnRestart === true) this.stratumUser = "changed-worker";
  }

  private handle(request: http.IncomingMessage, response: http.ServerResponse): void {
    const route = new URL(request.url ?? "/", "http://device").pathname;
    if (request.method === "POST" && route === "/api/system/OTAWWW") {
      this.upload(request, response);
      return;
    }
    const send = (body: string, type = "text/plain"): void => {
      response.writeHead(200, { "content-type": type });
      response.end(body);
    };
    if (route === "/api/system/info") {
      send(JSON.stringify({
        sourceCommit, referenceCommit, appElfSha256: app, hostname: "private-hostname",
        ssid: "private-ssid", stratumUser: this.stratumUser, bootSession: this.session,
        bootOrdinal: this.ordinal, runningPartition: "factory",
        axeOSVersion: this.assets === "erased" ? "Unavailable" : this.assets === "probe" ? probeLabel : buildLabel,
      }), "application/json");
      return;
    }
    if (route === "/api/system/logs") return send(`${this.logLines.join("\n")}\n`);
    if (route === "/recovery" || this.assets === "erased") return send(recoveryHtml, "text/html");
    if (route === "/version.txt") return send(`${this.assets === "probe" ? probeLabel : buildLabel}\n`);
    if (route === "/index.html") return send(indexHtml, "text/html");
    response.writeHead(404);
    response.end();
  }

  private upload(request: http.IncomingMessage, response: http.ServerResponse): void {
    this.uploads += 1;
    const chunks: Buffer[] = [];
    request.on("data", (chunk: Buffer) => chunks.push(chunk));
    request.on("error", () => {});
    request.on("close", () => {
      if (request.complete) return;
      if (this.behavior.ignoreInterruption === true) return;
      this.pending = "erased";
      this.logLines.push("www_update_status=Protocol Error");
    });
    request.on("end", () => {
      if (this.behavior.rejectUploads === true || request.headers.origin === undefined) {
        response.writeHead(400);
        response.end("File provided is too small for device");
        return;
      }
      const body = Buffer.concat(chunks);
      this.pending = body.equals(probeWww) ? "probe" : "package";
      this.logLines.push("www_update_status=Finished...");
      response.writeHead(200, { "content-type": "text/plain" });
      response.end("WWW update complete\n");
    });
  }
}

type Fixture = { readonly root: string; readonly manifest: string; readonly probeManifest: string; readonly projection: string };

async function fixture(): Promise<Fixture> {
  const root = await mkdtemp(path.join(os.tmpdir(), "bitaxe-otawww-"));
  await writeFile(path.join(root, "MODULE.bazel"), "module(name = \"fixture\")\n");
  const inputs = path.join(root, "inputs");
  await mkdir(inputs);
  const manifest = path.join(inputs, "package.json");
  const probeManifest = path.join(inputs, "www-probe.json");
  await writeFile(path.join(inputs, "www.bin"), packageWww);
  await writeFile(path.join(inputs, "www-probe.bin"), probeWww);
  await writeFile(manifest, JSON.stringify({
    source_commit: sourceCommit,
    reference_commit: referenceCommit,
    app_elf_sha256: app,
    build_identity: { label: buildLabel, source_dirty: false },
    artifacts: [
      { kind: "www_spiffs_image", path: "www.bin", sha256: sha256(packageWww) },
      { kind: "factory_merged_image", path: "factory.bin", sha256: "e".repeat(64) },
    ],
  }));
  await writeFile(probeManifest, JSON.stringify(probeMetadata()));
  return { root, manifest, probeManifest, projection: path.join(root, "docs", "otawww-projection.json") };
}

function probeMetadata(): Record<string, unknown> {
  return {
    schema_version: "bitaxe-www-probe-v1",
    source_commit: sourceCommit,
    reference_commit: referenceCommit,
    build_label: buildLabel,
    probe_label: probeLabel,
    package_www_sha256: sha256(packageWww),
    probe_www_sha256: sha256(probeWww),
    probe_www_bytes: probeWww.length,
    package_version_txt_sha256: sha256(`${buildLabel}\n`),
    probe_version_txt_sha256: sha256(`${probeLabel}\n`),
    index_html_sha256: sha256(indexHtml),
  };
}

function options(value: Fixture, phase: OtawwwPhase): OtawwwEvidenceOptions {
  return {
    phase,
    maybeEndpointInput: "scratch/attempt/endpoint.private.json",
    privateRoot: "scratch/attempt",
    packageManifest: value.manifest,
    wwwProbeManifest: value.probeManifest,
    port: "/dev/private-sensitive-port",
    projection: value.projection,
    captureTimeoutSeconds: 420,
  };
}

function readySession(): Record<string, unknown> {
  return {
    schema_version: "esp-device-session-v1", terminal_category: "ready", platform_category: "macos",
    board_category: "205", same_physical_device: true, stable_enumeration: true, reenumerated: true,
    reader_armed: true, pre_restart_serial_delivery: true, post_restart_serial_delivery: true,
    serial_delivery: "correlated", request_outcome: "response_received", request_attempt_count: 1,
    service_loss_observed: true, trusted_origin_preserved: true, application_recovered: true,
    build_identity_matches: true, boot_session_changed: true, boot_ordinal_advanced_by_one: true,
    software_reset_observed: true, postcondition_matches: true, cleanup_complete: true,
    usb_disappearance_count: 1, enumeration_change_count: 1, serial_byte_count: 500,
    http_observation_count: 2, duration_millis: 1_000,
  };
}

function fakePort(device: FakeDevice, commands: string[]): ProcessPort {
  return createFakeProcessPort(async (spec) => {
    const command = spec.program === "validator" ? "validator" : String(spec.args[0]);
    commands.push(command);
    if (command === "flash-monitor") {
      assert.equal(spec.args.includes("--wifi-credentials"), false);
      assert.equal(spec.args.includes("--factory-reset"), false);
      const effectPath = String(spec.environment?.["PHASE36_EFFECT_RESULT_PATH"]);
      await writeFile(effectPath, `${JSON.stringify({
        schema_version: "phase36-effect-result-v1", operation: "exact_package_flash", status: "completed", failure: null,
        package_identity_digest: spec.environment?.["PHASE36_EFFECT_PACKAGE_IDENTITY_DIGEST"],
        factory_image_digest: spec.environment?.["PHASE36_EFFECT_FACTORY_IMAGE_DIGEST"],
      })}\n`, { mode: 0o600 });
      const root = String(spec.args[spec.args.indexOf("--evidence-dir") + 1]);
      await writeFile(path.join(root, "flash-monitor.classifier-input.log"), "usb_boot_profile\n", { mode: 0o600 });
      return ok();
    }
    if (command === "reboot-live") {
      device.restart();
      const projection = String(spec.args[spec.args.indexOf("--projection-output") + 1]);
      await writeFile(projection, `${JSON.stringify(readySession())}\n`, { mode: 0o600 });
      await chmod(projection, 0o600);
      return ok();
    }
    if (command === "flash" || command === "validator") return ok();
    throw new Error(`unexpected child process ${command}`);
  });
}

async function phase(value: Fixture, device: FakeDevice, commands: string[], selected: OtawwwPhase) {
  return captureOtawwwEvidence(value.root, options(value, selected), fakePort(device, commands),
    "flash", "device-session", "validator", fastTiming);
}

/** Writes the Gate handoff the endpoint supervisor would produce for the installed boot. */
async function handoff(value: Fixture, origin: string, bootOrdinal: number, receivedAtMs = Date.now()): Promise<void> {
  const url = new URL(origin);
  await writeFile(path.join(value.root, "scratch", "attempt", "endpoint.private.json"), JSON.stringify({
    schema: "otawww-station-endpoint-v1", ipv4: url.hostname, httpPort: Number(url.port), bootOrdinal, generation: 3,
    observedAtUs: 5_000_000, hostReceivedAtMs: receivedAtMs,
  }), { mode: 0o600 });
}

async function run(value: Fixture, device: FakeDevice, commands: string[], withHandoff = true) {
  const origin = await device.listen();
  try {
    await phase(value, device, commands, "install");
    if (withHandoff) await handoff(value, origin, device.ordinal);
    return await phase(value, device, commands, "run");
  } finally {
    await device.close();
  }
}

async function captureError(promise: Promise<unknown>): Promise<OtawwwEvidenceError> {
  try {
    await promise;
  } catch (error) {
    assert.ok(error instanceof OtawwwEvidenceError);
    return error;
  }
  assert.fail("expected an OTAWWW capture failure");
}

test("complete OTAWWW run publishes aggregate-only evidence", async () => {
  // Arrange
  const value = await fixture();
  const device = new FakeDevice();
  const commands: string[] = [];

  // Act
  const evidence = await run(value, device, commands);

  // Assert
  assert.ok("otawww" in evidence);
  const projection = await readFile(value.projection, "utf8");
  assert.equal(evidence.otawww.interruption_protocol_error_retained, true);
  assert.equal(device.uploads, 3);
  assert.equal(device.assets, "package");
  assert.deepEqual(commands, ["flash-monitor", "reboot-live", "reboot-live", "reboot-live", "validator"]);
  assert.doesNotMatch(projection, /127\.0\.0\.1|private-hostname|private-ssid|private-worker|private-sensitive-port/u);
  assert.deepEqual(await verifySemanticEvidenceRedaction(path.dirname(value.projection)), { checked: 1 });
});

test("missing station endpoint handoff stops before any OTAWWW request", async () => {
  // Arrange
  const value = await fixture();
  const device = new FakeDevice();
  const commands: string[] = [];

  // Act
  const error = await captureError(run(value, device, commands, false));

  // Assert
  assert.equal(error.category, "origin_unavailable");
  assert.equal(device.uploads, 0);
  assert.deepEqual(commands, ["flash-monitor"]);
});

test("a handoff from another boot is refused before any OTAWWW request", async () => {
  // Arrange
  const value = await fixture();
  const device = new FakeDevice();
  const commands: string[] = [];
  const origin = await device.listen();

  try {
    await phase(value, device, commands, "install");
    await handoff(value, origin, device.ordinal - 1);

    // Act
    const error = await captureError(phase(value, device, commands, "run"));

    // Assert
    assert.equal(error.category, "evidence_invalid");
    assert.equal(device.uploads, 0);
  } finally {
    await device.close();
  }
});

test("station endpoint admission requires a fresh private LAN address", () => {
  // Arrange
  const now = 1_000_000_000;
  const endpoint = (ipv4: string, receivedAt: number) => JSON.stringify({
    schema: "otawww-station-endpoint-v1", ipv4, httpPort: 80, bootOrdinal: 7, generation: 3, observedAtUs: 1, hostReceivedAtMs: receivedAt,
  });

  // Act
  const admitted = parseStationEndpoint(endpoint("192.168.1.20", now - 1_000), now);

  // Assert
  assert.equal(admitted.origin.origin, "http://192.168.1.20");
  assert.throws(() => parseStationEndpoint(endpoint("8.8.8.8", now), now), OtawwwInputError);
  assert.throws(() => parseStationEndpoint(endpoint("192.168.1.20", now - 16 * 60_000), now), OtawwwInputError);
  assert.equal(privateIpv4("172.32.0.1"), false);
  assert.equal(privateIpv4("10.0.0.300"), false);
});

test("unretained interruption stops and restores the package with one flash", async () => {
  // Arrange
  const value = await fixture();
  const device = new FakeDevice({ ignoreInterruption: true });
  const commands: string[] = [];

  // Act
  const error = await captureError(run(value, device, commands));

  // Assert
  assert.equal(error.category, "interruption_not_observed");
  assert.equal(error.publicValue["recovery_flash_used"], true);
  assert.equal(commands.filter((command) => command === "flash").length, 1);
});

test("refused probe upload is classified as an unobserved update", async () => {
  // Arrange
  const value = await fixture();
  const device = new FakeDevice({ rejectUploads: true });
  const commands: string[] = [];

  // Act
  const error = await captureError(run(value, device, commands));

  // Assert
  assert.equal(error.category, "update_not_observed");
});

test("changed stored settings fail the run as NVS not preserved", async () => {
  // Arrange
  const value = await fixture();
  const device = new FakeDevice({ changeSettingsOnRestart: true });
  const commands: string[] = [];

  // Act
  const error = await captureError(run(value, device, commands));

  // Assert
  assert.equal(error.category, "nvs_not_preserved");
  assert.equal(error.publicValue["recovery_flash_used"], false);
});

test("an existing private root is refused before any device effect", async () => {
  // Arrange
  const value = await fixture();
  await mkdir(path.join(value.root, "scratch", "attempt"), { recursive: true });
  const device = new FakeDevice();
  const commands: string[] = [];

  // Act
  const error = await captureError(run(value, device, commands));

  // Assert
  assert.equal(error.category, "evidence_invalid");
  assert.deepEqual(commands, []);
});

test("install records the exact package and stops before any HTTP effect", async () => {
  // Arrange
  const value = await fixture();
  const device = new FakeDevice();
  const commands: string[] = [];

  // Act
  const result = await phase(value, device, commands, "install");

  // Assert
  assert.deepEqual(result, { phase: "install", installed: true });
  assert.deepEqual(commands, ["flash-monitor"]);
  const record = JSON.parse(await readFile(path.join(value.root, "scratch", "attempt", "install.private.json"), "utf8")) as Record<string, unknown>;
  assert.equal(record["schema"], "bitaxe-otawww-install-v1");
});

test("probe metadata bound to another package is refused", () => {
  // Arrange
  const manifest = JSON.stringify({
    source_commit: sourceCommit, reference_commit: referenceCommit, app_elf_sha256: app,
    build_identity: { label: buildLabel, source_dirty: false },
    artifacts: [
      { kind: "www_spiffs_image", sha256: sha256(packageWww) },
      { kind: "factory_merged_image", sha256: "e".repeat(64) },
    ],
  });
  const probe = JSON.stringify({ ...probeMetadata(), source_commit: "f".repeat(40) });

  // Act / Assert
  assert.throws(() => parseOtawwwInputs(manifest, probe, packageWww, probeWww), OtawwwInputError);
});

test("settings digest ignores runtime fields and tracks stored settings", () => {
  // Arrange
  const base = { hostname: "h", stratumUser: "u", bootOrdinal: 1 };

  // Act
  const same = settingsDigest({ ...base, bootOrdinal: 2 });
  const changed = settingsDigest({ ...base, stratumUser: "v" });

  // Assert
  assert.equal(same, settingsDigest(base));
  assert.notEqual(changed, settingsDigest(base));
});

test("only the embedded recovery page is recognized as the recovery fallback", () => {
  // Act / Assert
  assert.equal(isRecoveryPage(recoveryHtml), true);
  assert.equal(isRecoveryPage(indexHtml), false);
  assert.equal(isRecoveryPage(`${buildLabel}\n`), false);
});

test("binary upload sends the origin header and the whole body once", async () => {
  // Arrange
  let received = { origin: "", bytes: 0 };
  const server = http.createServer((request, response) => {
    const chunks: Buffer[] = [];
    request.on("data", (chunk: Buffer) => chunks.push(chunk));
    request.on("end", () => {
      received = { origin: String(request.headers.origin), bytes: Buffer.concat(chunks).length };
      response.end("WWW update complete\n");
    });
  });
  await new Promise<void>((resolve) => server.listen(0, "127.0.0.1", resolve));
  const origin = new URL(`http://127.0.0.1:${String((server.address() as { port: number }).port)}`);

  try {
    // Act
    const reply = await postBinaryOnce(origin, "/api/system/OTAWWW", probeWww, 5_000);

    // Assert
    assert.deepEqual(reply, { status: 200, body: "WWW update complete\n" });
    assert.deepEqual(received, { origin: origin.origin, bytes: probeWww.length });
  } finally {
    server.close();
  }
});
