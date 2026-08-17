import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { chmod, mkdir, mkdtemp, readFile, rm, stat, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import {
  captureEmc2101ThermalEvidence,
  Emc2101ThermalEvidenceError,
  validateEmc2101SourceSemantics,
} from "./emc2101-thermal-evidence.js";
import {
  createFakeProcessPort,
  createLocalProcessPort,
  type ProcessOutcome,
  type ProcessPort,
} from "./process.js";
import type { WebSocketClient, WebSocketFactory } from "./websocket.js";

const sourceCommit = "a".repeat(40);
const referenceCommit = "c1915b0a63bfabebdb95a515cedfee05146c1d50";
const appElfSha256 = "c".repeat(64);
const session = "1".repeat(32);
const nodeProgram = process.env["JS_BINARY__NODE_BINARY"] ?? process.execPath;
const fixtureSources = new Map<string, string>([
  ["crates/bitaxe-safety/src/sensor_acquisition/emc2101.rs", [
    "pub const ULTRA205_EMC2101_TEMP_OFFSET_C: f64 = 5.0;",
    "pub fn apply_ultra205_emc2101_temperature_offset(",
    "validate_temperature(temperature_celsius + ULTRA205_EMC2101_TEMP_OFFSET_C)",
  ].join("\n")],
  ["crates/bitaxe-safety/src/thermal.rs", [
    "pub const ASIC_THROTTLE_TEMP_C: f64 = 75.0;",
    "pub const MIN_PLAUSIBLE_TEMP_C: f64 = -40.0;",
    "pub const MAX_PLAUSIBLE_TEMP_C: f64 = 150.0;",
  ].join("\n")],
  ["firmware/bitaxe/src/safety_adapter/emc2101.rs", [
    "Self::InternalTemperature => 0x00,",
    "read_internal_temperature_acquisition(bus)",
    "apply_ultra205_emc2101_temperature_offset(temperature)",
  ].join("\n")],
  ["firmware/bitaxe/src/safety_adapter/i2c_bus.rs", [
    "const EMC2101_I2C_ADDRESS: u8 = 0x4c;",
    "pub(crate) struct ReadOnlySensorBus",
    "self.read_register(EMC2101_I2C_ADDRESS, register.address(), output)",
  ].join("\n")],
  ["firmware/bitaxe/src/operator_sensor_runtime.rs", "chip_temp_celsius: project_observation(\n"],
  ["crates/bitaxe-api/src/observation.rs", [
    "pub chip_temp_celsius: Observation<f64>,",
    "(MIN_PLAUSIBLE_TEMP_C..ASIC_THROTTLE_TEMP_C).contains(&chip_temp_celsius)",
  ].join("\n")],
  ["crates/bitaxe-api/src/wire.rs", [
    "pub temp: f64,",
    "pub chip_temp_status: ObservationTruthWire,",
    "temp: safe_telemetry.chip_temp_celsius,",
  ].join("\n")],
  ["reference/esp-miner/main/device_config.h",
    '.board_version = "205",  .family = FAMILY_ULTRA,       .EMC2101 = true, .emc_internal_temp = true,                                  .temp_offset = 5,'],
  ["reference/esp-miner/main/thermal/EMC2101.c", [
    "float EMC2101_get_internal_temp(void)",
    "EMC2101_INTERNAL_TEMP",
    "return (float) temp + temp_offset;",
  ].join("\n")],
]);
const ok = (stdout = ""): ProcessOutcome => ({
  exitCode: 0,
  stdout,
  stderr: "",
  timedOut: false,
});

type Contract = {
  readonly fields: Readonly<Record<string, {
    readonly type: string;
    readonly presence: string;
  }>>;
};

function runtimeHealth(sequence: number) {
  return {
    selfTestState: "unavailable",
    supervisorAvailability: "available",
    checkpointCategory: "telemetry",
    checkpointSequence: sequence,
    checkpointAgeMillis: 100,
    checkpointHealth: "healthy",
    taskWatchdogParticipation: "participating",
    taskWatchdogReason: "feed_fresh",
    taskWatchdogFeedSequence: sequence + 2,
    taskWatchdogFeedAgeMillis: 50,
    taskWatchdogOwnerPhase: "waiting_inbox",
    taskWatchdogOwnerSubphase: "unavailable",
    taskWatchdogReadOutcome: "stable",
    taskWatchdogWaitState: "within_deadline",
  };
}

function retained(revision: number, sequence: number): string {
  return `runtime_health boot_session=${session} operator_snapshot_revision=${String(revision)} self_test=unavailable supervisor=available checkpoint_category=telemetry checkpoint_sequence=${String(sequence)} checkpoint_age_millis=100 checkpoint_health=healthy task_watchdog_participation=participating task_watchdog_reason=feed_fresh task_watchdog_feed_sequence=${String(sequence + 2)} task_watchdog_feed_age_millis=50 task_watchdog_read_outcome=stable task_watchdog_owner_phase=waiting_inbox task_watchdog_owner_subphase=unavailable task_watchdog_wait_state=within_deadline redacted=true`;
}

function sampleFor(type: string): unknown {
  if (type === "array") return [];
  if (type === "boolean") return false;
  if (type === "number") return 0;
  if (type === "object") return {};
  return "";
}

function snapshot(
  contract: Contract,
  revision: number,
  sequence: number,
  temperature = 50,
): Record<string, unknown> {
  const value: Record<string, unknown> = {};
  for (const [field, rule] of Object.entries(contract.fields)) {
    if (rule.presence === "always") value[field] = sampleFor(rule.type);
  }
  return {
    ...value,
    blockFound: 0,
    bootSession: session,
    operatorSnapshotRevision: revision,
    sourceCommit,
    referenceCommit,
    appElfSha256,
    runtimeHealth: runtimeHealth(sequence),
    temp: temperature,
    chipTempStatus: {
      state: "fresh",
      stamp: { bootSession: 9_007_199_254_740_992, sequence: 11, acquiredAtMs: 500 },
    },
  };
}

async function sourceContract(): Promise<string> {
  const relative = "crates/bitaxe-api/fixtures/api/system-info-contract-v1.json";
  const maybeRunfiles = process.env["RUNFILES_DIR"];
  const candidates = [
    ...(maybeRunfiles === undefined ? [] : [path.join(maybeRunfiles, "_main", relative)]),
    path.join(process.cwd(), relative),
    path.resolve(process.cwd(), "..", "..", relative),
  ];
  for (const candidate of candidates) {
    try {
      return await readFile(candidate, "utf8");
    } catch (error) {
      if ((error as NodeJS.ErrnoException).code !== "ENOENT") throw error;
    }
  }
  throw new Error("system info field contract test input is missing");
}

async function fixture(name: string) {
  const root = await mkdtemp(path.join(os.tmpdir(), `bitaxe-emc2101-${name}-`));
  await writeFile(path.join(root, "MODULE.bazel"), "module(name = \"fixture\")\n");
  for (const [relative, document] of fixtureSources) {
    const destination = path.join(root, relative);
    await mkdir(path.dirname(destination), { recursive: true });
    await writeFile(destination, `${document}\n`);
  }
  const planDocument = [
    "- Parity row: `THR-001`",
    "- Active task: `task-parity-thr001-emc2101-live-thermal`",
    "",
  ].join("\n");
  const planRelative = "docs/parity/work-plans/20260813T015631Z-THR-001/PLAN.md";
  await mkdir(path.dirname(path.join(root, planRelative)), { recursive: true });
  await writeFile(path.join(root, planRelative), planDocument);
  await writeFile(path.join(root, "TASKS.md"), [
    "### task-parity-thr001-emc2101-live-thermal | fixture",
    "Plan: `docs/parity/work-plans/20260813T015631Z-THR-001/PLAN.md`.",
    "Schema: `bitaxe-emc2101-thermal-evidence-v1`.",
    "Attempt: `attempt-003`.",
    "",
  ].join("\n"));
  const contractRelative = "crates/bitaxe-api/fixtures/api/system-info-contract-v1.json";
  const contractDocument = await sourceContract();
  await mkdir(path.dirname(path.join(root, contractRelative)), { recursive: true });
  await writeFile(path.join(root, contractRelative), contractDocument);
  const contract = JSON.parse(contractDocument) as Contract;
  await mkdir(path.join(root, "inputs"));
  const manifest = path.join(root, "inputs/package.json");
  const credentials = path.join(root, "inputs/wifi.json");
  await writeFile(manifest, JSON.stringify({
    source_commit: sourceCommit,
    reference_commit: referenceCommit,
    app_elf_sha256: appElfSha256,
  }));
  await writeFile(credentials, "{}\n", { mode: 0o600 });
  const wrapper = path.join(root, "scratch/thr001-emc2101/wrapper-003");
  await mkdir(wrapper, { recursive: true, mode: 0o700 });
  await chmod(wrapper, 0o700);
  for (const name of ["detector.stdout", "detector.stderr", "capture.stdout", "capture.stderr"]) {
    const output = path.join(wrapper, name);
    await writeFile(output, "", { mode: 0o600 });
    await chmod(output, 0o600);
  }
  return {
    root,
    contract,
    admittedPlanSha256: createHash("sha256").update(planDocument).digest("hex"),
    projection: path.join(
      root,
      "docs/parity/evidence/thr001-emc2101-thermal/thermal-projection.json",
    ),
    options: {
      privateRoot: "scratch/thr001-emc2101/attempt-003",
      packageManifest: manifest,
      wifiCredentials: credentials,
      detectorOutput: "scratch/thr001-emc2101/wrapper-003/detector.stdout",
      port: "/dev/private-port",
      projection: "docs/parity/evidence/thr001-emc2101-thermal/thermal-projection.json",
      captureTimeoutSeconds: 360,
    },
  };
}

function installHttp(contract: Contract, temperature = 50) {
  const original = globalThis.fetch;
  globalThis.fetch = async (input) => {
    const target = new URL(String(input));
    if (target.pathname.endsWith("/logs")) {
      return new Response(`${retained(7, 9)}\n${retained(8, 10)}\n`, { status: 200 });
    }
    return new Response(JSON.stringify(snapshot(contract, 7, 9, temperature)), {
      status: 200,
      headers: { "content-type": "application/json" },
    });
  };
  return () => {
    globalThis.fetch = original;
  };
}

function websocketFactory(value: Record<string, unknown>): WebSocketFactory {
  return () => {
    const listeners = new Map<string, (event: { readonly data: unknown }) => void>();
    const client: WebSocketClient = {
      addEventListener(type, listener): void {
        listeners.set(type, listener);
      },
      close(): void {},
    };
    queueMicrotask(() => listeners.get("message")?.({
      data: JSON.stringify({ event: "update", data: value }),
    }));
    return client;
  };
}

function fakePort(configuration: {
  readonly flash?: ProcessOutcome;
  readonly inputValidation?: ProcessOutcome;
  readonly inputLaunchFailure?: boolean;
  readonly launchFailure?: boolean;
} = {}): ProcessPort {
  return createFakeProcessPort(async (spec) => {
    if (spec.args[0] === "flash-monitor") {
      if (configuration.launchFailure === true) throw new Error("launch canary");
      const root = String(spec.args[spec.args.indexOf("--evidence-dir") + 1]);
      await writeFile(path.join(root, "flash-monitor.classifier-input.log"), [
        "safe_state: mining=disabled asic_work_submission=disabled hardware_control=disabled",
        `runtime_origin session=${session} device_url=http://private-device.test redacted=true`,
        "",
      ].join("\n"), { mode: 0o600 });
      await writeFile(
        path.join(root, "flash-command-evidence.private.json"),
        "{}\n",
        { mode: 0o600 },
      );
      return configuration.flash ?? ok();
    }
    if (spec.args[0] === "flash") return ok();
    if (spec.program === "thermal-input-validator") {
      if (configuration.inputLaunchFailure === true) throw new Error("input launch canary");
      assert.equal(spec.args.length, 2);
      const documents = await Promise.all(spec.args.map((candidate) => readFile(candidate, "utf8")));
      assert.ok(documents.every((document) => document.includes('"chipTempStatus"')));
      return configuration.inputValidation ?? ok();
    }
    if (spec.program === "system-validator" || spec.program === "thermal-validator") return ok();
    if (spec.program === "git") {
      if (spec.args[0] === "status") return ok();
      if (spec.args[0] === "-C") return ok(`${referenceCommit}\n`);
      if (spec.args[0] === "rev-parse") return ok(`${sourceCommit}\n`);
    }
    throw new Error("unexpected child process");
  });
}

async function capture(
  value: Awaited<ReturnType<typeof fixture>>,
  processPort: ProcessPort,
  websocketTemperature = 50,
) {
  return captureEmc2101ThermalEvidence(
    value.root,
    value.options,
    processPort,
    "flash",
    "git",
    "system-validator",
    "thermal-input-validator",
    "thermal-validator",
    websocketFactory(snapshot(value.contract, 8, 10, websocketTemperature)),
    value.admittedPlanSha256,
  );
}

async function captureError(promise: Promise<unknown>): Promise<Emc2101ThermalEvidenceError> {
  try {
    await promise;
    assert.fail("expected thermal capture failure");
  } catch (error) {
    assert.ok(error instanceof Emc2101ThermalEvidenceError);
    return error;
  }
}

test("ready correlated thermal capture publishes aggregate-only v1 evidence", async () => {
  // Arrange
  const value = await fixture("ready");
  const restore = installHttp(value.contract);

  try {
    // Act
    const evidence = await capture(value, fakePort());
    const document = await readFile(value.projection, "utf8");

    // Assert
    assert.equal(evidence.thermal.temperature_offset_celsius, 5);
    assert.equal(evidence.thermal.same_acquisition_stamp, true);
    assert.equal((await stat(value.projection)).mode & 0o777, 0o644);
    assert.doesNotMatch(document, /private-device|private-port|bootSession|acquiredAtMs|"temp"/u);
  } finally {
    restore();
    await rm(value.root, { recursive: true });
  }
});

test("checked-in EMC2101 source semantics admit the simplified reducer", async () => {
  // Arrange
  const maybeRunfiles = process.env["RUNFILES_DIR"];
  const workspaceRoot = maybeRunfiles === undefined
    ? process.cwd()
    : path.join(maybeRunfiles, "_main");
  const reducer = await readFile(
    path.join(workspaceRoot, "crates/bitaxe-safety/src/sensor_acquisition/emc2101.rs"),
    "utf8",
  );

  // Act
  const result = validateEmc2101SourceSemantics(workspaceRoot);

  // Assert
  await assert.doesNotReject(result);
  assert.doesNotMatch(
    reducer,
    /let adjusted = temperature_celsius \+ ULTRA205_EMC2101_TEMP_OFFSET_C;/u,
  );
});

test("source semantics reject the stale attempt-001 intermediate statement", async () => {
  // Arrange
  const value = await fixture("stale-source-fragment");
  const reducer = path.join(
    value.root,
    "crates/bitaxe-safety/src/sensor_acquisition/emc2101.rs",
  );
  const current = await readFile(reducer, "utf8");
  await writeFile(
    reducer,
    current.replace(
      "validate_temperature(temperature_celsius + ULTRA205_EMC2101_TEMP_OFFSET_C)",
      [
        "let adjusted = temperature_celsius + ULTRA205_EMC2101_TEMP_OFFSET_C;",
        "validate_temperature(adjusted)",
      ].join("\n"),
    ),
  );

  try {
    // Act
    const result = validateEmc2101SourceSemantics(value.root);

    // Assert
    await assert.rejects(result, Emc2101ThermalEvidenceError);
  } finally {
    await rm(value.root, { recursive: true });
  }
});

test("unsafe or uncorrelated thermal samples withhold final evidence", async () => {
  for (const testCase of [
    { name: "threshold", http: 75, websocket: 75, category: "evidence_invalid" },
    { name: "uncorrelated", http: 50, websocket: 51, category: "evidence_invalid" },
  ] as const) {
    // Arrange
    const value = await fixture(testCase.name);
    const restore = installHttp(value.contract, testCase.http);

    try {
      // Act
      const inputValidation = { ...ok(), exitCode: 1 };
      const error = await captureError(
        capture(value, fakePort({ inputValidation }), testCase.websocket),
      );

      // Assert
      assert.equal(error.category, testCase.category);
      await assert.rejects(readFile(value.projection, "utf8"), { code: "ENOENT" });
    } finally {
      restore();
      await rm(value.root, { recursive: true });
    }
  }
});

test("base timeout and launch failure preserve their typed categories", async () => {
  for (const testCase of [
    { name: "timeout", port: fakePort({ flash: { ...ok(), timedOut: true } }), category: "timeout" },
    { name: "launch", port: fakePort({ launchFailure: true }), category: "process_failed" },
  ] as const) {
    // Arrange
    const value = await fixture(testCase.name);
    const restore = installHttp(value.contract);

    try {
      // Act
      const error = await captureError(capture(value, testCase.port));

      // Assert
      assert.equal(error.category, testCase.category);
      assert.equal(error.publicValue["projection_published"], false);
    } finally {
      restore();
      await rm(value.root, { recursive: true });
    }
  }
});

test("lossless input validator rejection withholds final evidence", async () => {
  // Arrange
  const value = await fixture("input-validator-rejection");
  const restore = installHttp(value.contract);
  const inputValidation = { ...ok(), exitCode: 1 };

  try {
    // Act
    const error = await captureError(capture(value, fakePort({ inputValidation })));

    // Assert
    assert.equal(error.category, "evidence_invalid");
    await assert.rejects(readFile(value.projection, "utf8"), { code: "ENOENT" });
  } finally {
    restore();
    await rm(value.root, { recursive: true });
  }
});

test("lossless input validator preserves timeout and launch categories", async () => {
  for (const testCase of [
    {
      name: "input-timeout",
      port: fakePort({ inputValidation: { ...ok(), timedOut: true } }),
      category: "timeout",
    },
    {
      name: "input-launch",
      port: fakePort({ inputLaunchFailure: true }),
      category: "process_failed",
    },
  ] as const) {
    // Arrange
    const value = await fixture(testCase.name);
    const restore = installHttp(value.contract);

    try {
      // Act
      const error = await captureError(capture(value, testCase.port));

      // Assert
      assert.equal(error.category, testCase.category);
      await assert.rejects(readFile(value.projection, "utf8"), { code: "ENOENT" });
    } finally {
      restore();
      await rm(value.root, { recursive: true });
    }
  }
});

test("real child processes own flash artifacts git identity and all validator boundaries", async () => {
  // Arrange
  const value = await fixture("real-child");
  const restore = installHttp(value.contract);
  const child = path.join(value.root, "child.mjs");
  await writeFile(child, `#!${nodeProgram}
import { readFile, writeFile } from "node:fs/promises";
import path from "node:path";
const args = process.argv.slice(2);
if (args[0] === "flash-monitor") {
  const root = args[args.indexOf("--evidence-dir") + 1];
  await writeFile(path.join(root, "flash-monitor.classifier-input.log"), ${JSON.stringify(`safe_state: mining=disabled asic_work_submission=disabled hardware_control=disabled\nruntime_origin session=${session} device_url=http://private-device.test redacted=true\n`)}, { mode: 0o600 });
  await writeFile(path.join(root, "flash-command-evidence.private.json"), "{}\\n", { mode: 0o600 });
} else if (args[0] === "status") {
  process.stdout.write("");
} else if (args[0] === "-C") {
  process.stdout.write(${JSON.stringify(`${referenceCommit}\n`)});
} else if (args[0] === "rev-parse") {
  process.stdout.write(${JSON.stringify(`${sourceCommit}\n`)});
} else if (args.length === 2) {
  const documents = await Promise.all(args.map((candidate) => readFile(candidate, "utf8")));
  if (!documents.every((document) => document.includes('"chipTempStatus"'))) process.exitCode = 1;
} else {
  const value = JSON.parse(await readFile(args[0], "utf8"));
  if (!["bitaxe-system-info-evidence-v1", "bitaxe-emc2101-thermal-evidence-v1"].includes(value.schema_version)) process.exitCode = 1;
}
`);
  await chmod(child, 0o700);

  try {
    // Act
    const evidence = await captureEmc2101ThermalEvidence(
      value.root,
      value.options,
      createLocalProcessPort({ cwd: value.root, timeoutMs: 5_000 }),
      child,
      child,
      child,
      child,
      child,
      websocketFactory(snapshot(value.contract, 8, 10)),
      value.admittedPlanSha256,
    );

    // Assert
    assert.equal(evidence.schema_version, "bitaxe-emc2101-thermal-evidence-v1");
  } finally {
    restore();
    await rm(value.root, { recursive: true });
  }
});
