import assert from "node:assert/strict";
import { chmod, mkdtemp, mkdir, readFile, stat, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import {
  captureLogBufferEvidence,
  LogBufferEvidenceError,
  type LogBufferEvidenceOptions,
} from "./log-buffer-evidence.js";
import {
  createFakeProcessPort,
  createLocalProcessPort,
  type ProcessOutcome,
  type ProcessPort,
} from "./process.js";
import type { WebSocketFactory } from "./websocket.js";

const sourceCommit = "a".repeat(40);
const referenceCommit = "b".repeat(40);
const appElfSha256 = "c".repeat(64);
const bootSession = "1".repeat(32);
const rawLogMarker = "axeos_websocket_logs=connected\n";
const privateOrigin = "http://private-device.test";
const trace = [
  `runtime_boot_identity session=${bootSession} ordinal=1`,
  "safe_state: mining=disabled asic_work_submission=disabled hardware_control=disabled",
  `runtime_origin session=${bootSession} device_url=${privateOrigin} redacted=true`,
  "",
].join("\n");
const ok = (): ProcessOutcome => ({ exitCode: 0, stdout: "", stderr: "", timedOut: false });
const failed = (): ProcessOutcome => ({ exitCode: 1, stdout: "", stderr: "", timedOut: false });
const nodeProgram = process.env["JS_BINARY__NODE_BINARY"] ?? process.execPath;

type Fixture = {
  readonly root: string;
  readonly manifest: string;
  readonly credentials: string;
  readonly projection: string;
};

async function fixture(name: string): Promise<Fixture> {
  const root = await mkdtemp(path.join(os.tmpdir(), `bitaxe-log-buffer-${name}-`));
  await writeFile(path.join(root, "MODULE.bazel"), "module(name = \"fixture\")\n");
  await mkdir(path.join(root, "inputs"));
  const manifest = path.join(root, "inputs", "package.json");
  const credentials = path.join(root, "inputs", "wifi.json");
  await writeFile(manifest, JSON.stringify({
    source_commit: sourceCommit,
    reference_commit: referenceCommit,
    app_elf_sha256: appElfSha256,
    artifacts: [{ kind: "factory_merged_image", sha256: "d".repeat(64) }],
  }));
  await writeFile(credentials, "{}\n", { mode: 0o600 });
  return { root, manifest, credentials, projection: path.join(root, "docs", "projection.json") };
}

function options(value: Fixture): LogBufferEvidenceOptions {
  return {
    privateRoot: "scratch/attempt",
    packageManifest: value.manifest,
    wifiCredentials: value.credentials,
    port: "/dev/private-sensitive-port",
    projection: value.projection,
    captureTimeoutSeconds: 240,
  };
}

async function writeCompletedEffect(spec: Parameters<ProcessPort["run"]>[0]): Promise<void> {
  const maybeEffectPath = spec.environment?.["PHASE36_EFFECT_RESULT_PATH"];
  assert.notEqual(maybeEffectPath, undefined);
  const effectPath = String(maybeEffectPath);
  await writeFile(effectPath, `${JSON.stringify({
    schema_version: "phase36-effect-result-v1",
    operation: "exact_package_flash",
    status: "completed",
    failure: null,
    package_identity_digest: spec.environment?.["PHASE36_EFFECT_PACKAGE_IDENTITY_DIGEST"],
    factory_image_digest: spec.environment?.["PHASE36_EFFECT_FACTORY_IMAGE_DIGEST"],
  })}\n`, { mode: 0o600 });
  await chmod(effectPath, 0o600);
}

function fakePort(configuration: {
  readonly flash?: ProcessOutcome;
  readonly validator?: ProcessOutcome;
  readonly launchFailure?: boolean;
} = {}): ProcessPort {
  return createFakeProcessPort(async (spec) => {
    if (spec.args[0] === "flash-monitor") {
      if (configuration.launchFailure === true) throw new Error("private launch failure");
      await writeCompletedEffect(spec);
      const root = String(spec.args[spec.args.indexOf("--evidence-dir") + 1]);
      await writeFile(path.join(root, "flash-monitor.classifier-input.log"), trace, { mode: 0o600 });
      return configuration.flash ?? ok();
    }
    if (spec.program === "validator") return configuration.validator ?? ok();
    throw new Error("unexpected child process");
  });
}

function installDeviceApi(configuration: {
  readonly wrongHeaders?: boolean;
  readonly retainMarker?: boolean;
  readonly frame?: unknown;
} = {}) {
  let retained = "booted\n";
  let closed = false;
  const originalFetch = globalThis.fetch;
  globalThis.fetch = async (input) => {
    const target = new URL(String(input));
    if (target.pathname === "/api/system/info") {
      return new Response(JSON.stringify({
        sourceCommit,
        referenceCommit,
        appElfSha256,
        bootSession,
      }), { status: 200, headers: { "content-type": "application/json" } });
    }
    if (target.pathname === "/api/system/logs") {
      return new Response(retained, {
        status: 200,
        headers: {
          "content-type": configuration.wrongHeaders === true ? "application/octet-stream" : "text/plain",
          "content-disposition": "attachment; filename=\"bitaxe-logs.txt\"",
        },
      });
    }
    throw new Error("unexpected same-origin request");
  };
  const websocketFactory: WebSocketFactory = (target) => {
    assert.equal(target, "ws://private-device.test/api/ws");
    if (configuration.retainMarker !== false) retained += rawLogMarker;
    return {
      addEventListener(type, listener) {
        if (type === "message") {
          queueMicrotask(() => listener({ data: configuration.frame ?? rawLogMarker }));
        }
      },
      close() {
        closed = true;
      },
    };
  };
  return {
    closed: () => closed,
    restore() {
      globalThis.fetch = originalFetch;
    },
    websocketFactory,
  };
}

async function capture(value: Fixture, processPort: ProcessPort, websocketFactory: WebSocketFactory) {
  return captureLogBufferEvidence(
    value.root,
    options(value),
    processPort,
    "flash",
    "validator",
    websocketFactory,
  );
}

async function captureError(promise: Promise<unknown>): Promise<LogBufferEvidenceError> {
  try {
    await promise;
    assert.fail("expected log buffer capture failure");
  } catch (error) {
    assert.ok(error instanceof LogBufferEvidenceError);
    return error;
  }
}

test("one raw marker is correlated with two retained downloads", async () => {
  // Arrange
  const value = await fixture("ready");
  const device = installDeviceApi();

  try {
    // Act
    const evidence = await capture(value, fakePort(), device.websocketFactory);
    const projection = await readFile(value.projection, "utf8");
    const privateRoot = path.join(value.root, "scratch", "attempt");

    // Assert
    assert.equal(evidence.log_buffer["new_marker_count"], 1);
    assert.equal(evidence.log_buffer["baseline_is_exact_prefix"], true);
    assert.equal(device.closed(), true);
    assert.doesNotMatch(projection, /private-device|private-sensitive-port|axeos_websocket_logs|booted/u);
    assert.equal((await stat(privateRoot)).mode & 0o777, 0o700);
    assert.equal((await stat(path.join(privateRoot, "final-evidence.private.json"))).mode & 0o777, 0o600);
    assert.equal((await stat(path.join(privateRoot, "raw-log-frame.private.txt"))).mode & 0o777, 0o600);
  } finally {
    device.restore();
  }
});

test("wrong retained download headers withhold public evidence", async () => {
  // Arrange
  const value = await fixture("headers");
  const device = installDeviceApi({ wrongHeaders: true });

  try {
    // Act
    const error = await captureError(capture(value, fakePort(), device.websocketFactory));

    // Assert
    assert.equal(error.category, "evidence_invalid");
    assert.equal(error.publicValue["flash_effect_completed"], true);
    await assert.rejects(readFile(value.projection, "utf8"), { code: "ENOENT" });
  } finally {
    device.restore();
  }
});

test("a streamed marker not retained by the download path fails correlation", async () => {
  // Arrange
  const value = await fixture("missing-retained-marker");
  const device = installDeviceApi({ retainMarker: false });

  try {
    // Act
    const error = await captureError(capture(value, fakePort(), device.websocketFactory));

    // Assert
    assert.equal(error.category, "evidence_invalid");
    await assert.rejects(readFile(value.projection, "utf8"), { code: "ENOENT" });
  } finally {
    device.restore();
  }
});

test("a binary WebSocket marker cannot claim a plain-text frame", async () => {
  // Arrange
  const value = await fixture("binary-frame");
  const device = installDeviceApi({ frame: Buffer.from(rawLogMarker, "utf8") });

  try {
    // Act
    const error = await captureError(capture(value, fakePort(), device.websocketFactory));

    // Assert
    assert.equal(error.category, "evidence_invalid");
    await assert.rejects(readFile(value.projection, "utf8"), { code: "ENOENT" });
  } finally {
    device.restore();
  }
});

test("flash launch timeout readiness and validator failures stay closed", async () => {
  for (const testCase of [
    { name: "launch", port: fakePort({ launchFailure: true }), category: "process_failed" },
    { name: "timeout", port: fakePort({ flash: { ...ok(), timedOut: true } }), category: "timeout" },
    { name: "readiness", port: fakePort({ flash: failed() }), category: "hardware_blocked" },
    { name: "validator", port: fakePort({ validator: failed() }), category: "evidence_invalid" },
  ] as const) {
    // Arrange
    const value = await fixture(testCase.name);
    const device = installDeviceApi();

    try {
      // Act
      const error = await captureError(capture(value, testCase.port, device.websocketFactory));

      // Assert
      assert.equal(error.category, testCase.category);
      await assert.rejects(readFile(value.projection, "utf8"), { code: "ENOENT" });
    } finally {
      device.restore();
    }
  }
});

test("an existing private root fails before a flash child launches", async () => {
  // Arrange
  const value = await fixture("no-clobber");
  await mkdir(path.join(value.root, "scratch", "attempt"), { recursive: true });
  let childCalled = false;
  const processPort = createFakeProcessPort(async () => {
    childCalled = true;
    return ok();
  });

  // Act
  const error = await captureError(captureLogBufferEvidence(
    value.root,
    options(value),
    processPort,
    "flash",
    "validator",
  ));

  // Assert
  assert.equal(error.category, "evidence_invalid");
  assert.equal(childCalled, false);
});

test("real child processes preserve flash-effect and validation file boundaries", async () => {
  // Arrange
  const value = await fixture("real-child");
  const device = installDeviceApi();
  const child = path.join(value.root, "child.mjs");
  await writeFile(child, `#!${nodeProgram}\nimport { chmod, writeFile } from "node:fs/promises"; import path from "node:path"; const args=process.argv.slice(2); if(args[0]==="flash-monitor"){const root=args[args.indexOf("--evidence-dir")+1]; await writeFile(path.join(root,"flash-monitor.classifier-input.log"),${JSON.stringify(trace)},{mode:0o600}); const effect=process.env.PHASE36_EFFECT_RESULT_PATH; await writeFile(effect,JSON.stringify({schema_version:"phase36-effect-result-v1",operation:"exact_package_flash",status:"completed",failure:null,package_identity_digest:process.env.PHASE36_EFFECT_PACKAGE_IDENTITY_DIGEST,factory_image_digest:process.env.PHASE36_EFFECT_FACTORY_IMAGE_DIGEST})+"\\n",{mode:0o600}); await chmod(effect,0o600);}\n`);
  await chmod(child, 0o700);

  try {
    // Act
    const evidence = await captureLogBufferEvidence(
      value.root,
      options(value),
      createLocalProcessPort({ cwd: value.root, timeoutMs: 5_000 }),
      child,
      child,
      device.websocketFactory,
    );

    // Assert
    assert.equal(evidence.schema_version, "bitaxe-log-buffer-evidence-v1");
    assert.equal(evidence.cleanup_complete, true);
  } finally {
    device.restore();
  }
});
