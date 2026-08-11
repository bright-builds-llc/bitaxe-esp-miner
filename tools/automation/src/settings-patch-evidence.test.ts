import assert from "node:assert/strict";
import { chmod, mkdtemp, mkdir, readFile, stat, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import { createFakeProcessPort, createLocalProcessPort, type ProcessOutcome, type ProcessPort } from "./process.js";
import { captureSettingsPatchEvidence, SettingsPatchEvidenceError } from "./settings-patch-evidence.js";

const sourceCommit = "a".repeat(40);
const referenceCommit = "b".repeat(40);
const appElfSha256 = "c".repeat(64);
const trace = "safe_state: mining=disabled asic_work_submission=disabled hardware_control=disabled\nruntime_origin session=private device_url=http://private-device.test redacted=true\n";
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
  const root = await mkdtemp(path.join(os.tmpdir(), `bitaxe-settings-patch-${name}-`));
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

function options(value: Fixture) {
  return {
    privateRoot: "scratch/attempt",
    packageManifest: value.manifest,
    wifiCredentials: value.credentials,
    port: "/dev/private-sensitive-port",
    projection: value.projection,
    captureTimeoutSeconds: 240,
  } as const;
}

function installApi(configuration: { readonly mismatchMutation?: boolean; readonly failRecoveryRestore?: boolean } = {}) {
  const original = { hostname: "private-original-host", rotation: 0 };
  let current = { ...original };
  let patchCount = 0;
  const bodies: Array<Record<string, unknown>> = [];
  const originalFetch = globalThis.fetch;
  globalThis.fetch = async (_input, init) => {
    if (init?.method === "PATCH") {
      patchCount += 1;
      const body = JSON.parse(String(init.body)) as Record<string, unknown>;
      bodies.push(body);
      assert.deepEqual(Object.keys(body).sort(), ["hostname", "rotation"]);
      if (configuration.failRecoveryRestore === true && patchCount >= 2) throw new Error("private recovery failure");
      current = { hostname: String(body["hostname"]), rotation: Number(body["rotation"]) };
      return new Response("ok", { status: 200 });
    }
    const response = configuration.mismatchMutation === true && patchCount === 1
      ? { ...current, rotation: 180 }
      : current;
    return new Response(JSON.stringify({
      ...response,
      sourceCommit,
      referenceCommit,
      appElfSha256,
    }), { status: 200, headers: { "content-type": "application/json" } });
  };
  return {
    bodies,
    current: () => current,
    original,
    restore: () => { globalThis.fetch = originalFetch; },
  };
}

async function writeCompletedEffect(spec: Parameters<ProcessPort["run"]>[0]): Promise<void> {
  const effectPath = spec.environment?.["PHASE36_EFFECT_RESULT_PATH"];
  assert.notEqual(effectPath, undefined);
  await writeFile(String(effectPath), `${JSON.stringify({
    schema_version: "phase36-effect-result-v1",
    operation: "exact_package_flash",
    status: "completed",
    failure: null,
    package_identity_digest: spec.environment?.["PHASE36_EFFECT_PACKAGE_IDENTITY_DIGEST"],
    factory_image_digest: spec.environment?.["PHASE36_EFFECT_FACTORY_IMAGE_DIGEST"],
  })}\n`, { mode: 0o600 });
  await chmod(String(effectPath), 0o600);
}

function fakePort(configuration: {
  readonly flash?: ProcessOutcome;
  readonly validator?: ProcessOutcome;
  readonly recovery?: ProcessOutcome;
  readonly launchFailure?: boolean;
} = {}): ProcessPort {
  return createFakeProcessPort(async (spec) => {
    const command = spec.args[0];
    if (command === "flash-monitor") {
      if (configuration.launchFailure === true) throw new Error("private launch failure");
      await writeCompletedEffect(spec);
      const root = String(spec.args[spec.args.indexOf("--evidence-dir") + 1]);
      await writeFile(path.join(root, "flash-monitor.classifier-input.log"), trace, { mode: 0o600 });
      return configuration.flash ?? ok();
    }
    if (command === "flash") return configuration.recovery ?? ok();
    if (spec.program === "validator") return configuration.validator ?? ok();
    throw new Error("unexpected child process");
  });
}

async function capture(value: Fixture, port: ProcessPort) {
  return captureSettingsPatchEvidence(value.root, options(value), port, "flash", "validator");
}

async function captureError(promise: Promise<unknown>): Promise<SettingsPatchEvidenceError> {
  try {
    await promise;
    assert.fail("expected settings PATCH capture failure");
  } catch (error) {
    assert.ok(error instanceof SettingsPatchEvidenceError);
    return error;
  }
}

test("one atomic mutation and restoration emit aggregate-only evidence", async () => {
  // Arrange
  const value = await fixture("ready");
  const api = installApi();

  try {
    // Act
    const evidence = await capture(value, fakePort());
    const document = await readFile(value.projection, "utf8");

    // Assert
    assert.equal(evidence.settings_patch["mutation_request_field_count"], 2);
    assert.equal(evidence.settings_patch["restoration_complete"], true);
    assert.equal(api.bodies.length, 2);
    assert.deepEqual(api.current(), api.original);
    assert.doesNotMatch(document, /private-original|private-device|private-sensitive-port/u);
    assert.equal((await stat(path.join(value.root, "scratch", "attempt"))).mode & 0o777, 0o700);
    assert.equal((await stat(path.join(value.root, "scratch", "attempt", "final-evidence.private.json"))).mode & 0o777, 0o600);
  } finally {
    api.restore();
  }
});

test("mutation mismatch preserves primary category when recovery also fails", async () => {
  // Arrange
  const value = await fixture("primary-precedence");
  const api = installApi({ mismatchMutation: true, failRecoveryRestore: true });

  try {
    // Act
    const error = await captureError(capture(value, fakePort({ recovery: failed() })));

    // Assert
    assert.equal(error.category, "hardware_blocked");
    assert.equal(error.publicValue["recovery_flash_used"], true);
    assert.equal(error.publicValue["secondary_recovery_failure"], true);
    await assert.rejects(readFile(value.projection, "utf8"), { code: "ENOENT" });
  } finally {
    api.restore();
  }
});

test("recovery PATCH restoration is reported without fallback flash", async () => {
  // Arrange
  const value = await fixture("recovery-patch");
  const api = installApi({ mismatchMutation: true });

  try {
    // Act
    const error = await captureError(capture(value, fakePort()));

    // Assert
    assert.equal(error.category, "hardware_blocked");
    assert.equal(error.publicValue["restoration_complete"], true);
    assert.equal(error.publicValue["recovery_flash_used"], false);
    assert.deepEqual(api.current(), api.original);
  } finally {
    api.restore();
  }
});

test("an existing private root fails before a flash child can launch", async () => {
  // Arrange
  const value = await fixture("no-clobber");
  await mkdir(path.join(value.root, "scratch", "attempt"), { recursive: true });
  let childCalled = false;
  const port = createFakeProcessPort(async () => {
    childCalled = true;
    return ok();
  });

  // Act
  const error = await captureError(capture(value, port));

  // Assert
  assert.equal(error.category, "evidence_invalid");
  assert.equal(childCalled, false);
});

test("timeout launch failure and validator rejection remain closed", async () => {
  for (const testCase of [
    { name: "timeout", port: fakePort({ flash: { ...ok(), timedOut: true } }), category: "timeout" },
    { name: "launch", port: fakePort({ launchFailure: true }), category: "process_failed" },
    { name: "validator", port: fakePort({ validator: failed() }), category: "evidence_invalid" },
  ] as const) {
    // Arrange
    const value = await fixture(testCase.name);
    const api = installApi();
    try {
      // Act
      const error = await captureError(capture(value, testCase.port));

      // Assert
      assert.equal(error.category, testCase.category);
      await assert.rejects(readFile(value.projection, "utf8"), { code: "ENOENT" });
    } finally {
      api.restore();
    }
  }
});

test("real child process supplies flash and validation boundaries", async () => {
  // Arrange
  const value = await fixture("real-child");
  const api = installApi();
  const child = path.join(value.root, "child.mjs");
  await writeFile(child, `#!${nodeProgram}\nimport { chmod, writeFile } from "node:fs/promises"; import path from "node:path"; const args=process.argv.slice(2); if(args[0]==="flash-monitor"){const root=args[args.indexOf("--evidence-dir")+1]; await writeFile(path.join(root,"flash-monitor.classifier-input.log"),${JSON.stringify(trace)},{mode:0o600}); const effect=process.env.PHASE36_EFFECT_RESULT_PATH; await writeFile(effect,JSON.stringify({schema_version:"phase36-effect-result-v1",operation:"exact_package_flash",status:"completed",failure:null,package_identity_digest:process.env.PHASE36_EFFECT_PACKAGE_IDENTITY_DIGEST,factory_image_digest:process.env.PHASE36_EFFECT_FACTORY_IMAGE_DIGEST})+"\\n",{mode:0o600}); await chmod(effect,0o600);}\n`);
  await chmod(child, 0o700);
  try {
    // Act
    const evidence = await captureSettingsPatchEvidence(
      value.root,
      options(value),
      createLocalProcessPort({ cwd: value.root, timeoutMs: 5_000 }),
      child,
      child,
    );

    // Assert
    assert.equal(evidence.schema_version, "bitaxe-settings-patch-evidence-v1");
    assert.equal(api.bodies.length, 2);
  } finally {
    api.restore();
  }
});
