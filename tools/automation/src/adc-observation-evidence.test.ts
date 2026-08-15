import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { chmod, mkdir, mkdtemp, readFile, rm, stat, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import {
  captureAdcObservationEvidence,
  AdcObservationEvidenceError,
  validateAdcObservationSourceSemantics,
} from "./adc-observation-evidence.js";
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
  ["crates/bitaxe-core/src/runtime_orchestration.rs",
    "pub const OPERATOR_OBSERVATION_CADENCE_MS: u64 = 500;"],
  ["crates/bitaxe-safety/src/core_voltage_acquisition.rs", [
    "AcquisitionOutcome::Success(millivolts)",
    "FaultReason::AdcReadFailed",
    "StaleReason::ProducerCadenceExpired",
  ].join("\n")],
  ["firmware/bitaxe/src/safety_adapter/adc.rs", [
    "ADC1<'static>",
    "Gpio2<'static>",
    "attenuation: attenuation::DB_12",
    "resolution: Resolution::new()",
    "calibration: Calibration::Curve",
  ].join("\n")],
  ["firmware/bitaxe/src/operator_sensor_runtime.rs", [
    "pub const SENSOR_SWEEP_CADENCE_MS: u64 = OPERATOR_OBSERVATION_CADENCE_MS;",
    "safety_adapter::read_core_voltage_acquisition(adc)",
    "core_voltage_state.record(core_voltage_millivolts, boot_session, acquired_at)",
  ].join("\n")],
  ["crates/bitaxe-api/src/observation.rs", "pub core_voltage_actual_mv: Observation<f64>"],
  ["crates/bitaxe-api/src/snapshot.rs", [
    "core_voltage_actual_mv: fresh_f64(&observations.core_voltage_actual_mv)",
    "core_voltage_status: (&observations.core_voltage_actual_mv).into()",
  ].join("\n")],
  ["crates/bitaxe-api/src/wire.rs", [
    '#[serde(rename = "coreVoltageActual")]',
    '#[serde(rename = "coreVoltageActualStatus")]',
    "core_voltage_actual: safe_telemetry.core_voltage_actual_mv",
  ].join("\n")],
  ["reference/esp-miner/main/adc.c", [
    "#define ADC_ATTEN   ADC_ATTEN_DB_12",
    "#define ADC_CHANNEL ADC_CHANNEL_1",
    ".unit_id = ADC_UNIT_1",
    ".bitwidth = ADC_BITWIDTH_DEFAULT",
    "adc_cali_create_scheme_curve_fitting",
    "adc_cali_raw_to_voltage(adc1_cali_chan1_handle, adc_raw, &voltage)",
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
  };
}

function retained(revision: number, sequence: number): string {
  return `runtime_health boot_session=${session} operator_snapshot_revision=${String(revision)} self_test=unavailable supervisor=available checkpoint_category=telemetry checkpoint_sequence=${String(sequence)} checkpoint_age_millis=100 checkpoint_health=healthy task_watchdog_participation=participating task_watchdog_reason=feed_fresh task_watchdog_feed_sequence=${String(sequence + 2)} task_watchdog_feed_age_millis=50 redacted=true`;
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
  adcSequence: number,
  acquiredAtMs: number,
  millivolts: number,
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
    coreVoltageActual: millivolts,
    coreVoltageActualStatus: {
      state: "fresh",
      stamp: { bootSession: 9, sequence: adcSequence, acquiredAtMs },
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
  const root = await mkdtemp(path.join(os.tmpdir(), `bitaxe-adc-${name}-`));
  await writeFile(path.join(root, "MODULE.bazel"), "module(name = \"fixture\")\n");
  for (const [relative, document] of fixtureSources) {
    const destination = path.join(root, relative);
    await mkdir(path.dirname(destination), { recursive: true });
    await writeFile(destination, `${document}\n`);
  }
  const planDocument = [
    "- Parity row: `IO-002`",
    "- Active task: `task-parity-io002-adc-observation`",
    "",
  ].join("\n");
  const planRelative = "docs/parity/work-plans/20260815T210711Z-IO-002/PLAN.md";
  await mkdir(path.dirname(path.join(root, planRelative)), { recursive: true });
  await writeFile(path.join(root, planRelative), planDocument);
  await writeFile(path.join(root, "TASKS.md"), [
    "### task-parity-io002-adc-observation | fixture",
    "Plan: `docs/parity/work-plans/20260815T210711Z-IO-002/PLAN.md`.",
    "Schema: `bitaxe-adc-observation-evidence-v1`.",
    "Attempt: `attempt-001`.",
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
  const wrapper = path.join(root, "scratch/io002-adc/wrapper-001");
  await mkdir(wrapper, { recursive: true, mode: 0o700 });
  await chmod(wrapper, 0o700);
  for (const outputName of ["detector.stdout", "detector.stderr", "capture.stdout", "capture.stderr"]) {
    const output = path.join(wrapper, outputName);
    await writeFile(output, "", { mode: 0o600 });
    await chmod(output, 0o600);
  }
  return {
    root,
    contract,
    admittedPlanSha256: createHash("sha256").update(planDocument).digest("hex"),
    projection: path.join(root, "docs/parity/evidence/io002-adc/adc-observation-projection.json"),
    options: {
      privateRoot: "scratch/io002-adc/attempt-001",
      packageManifest: manifest,
      wifiCredentials: credentials,
      detectorOutput: "scratch/io002-adc/wrapper-001/detector.stdout",
      port: "/dev/private-port",
      projection: "docs/parity/evidence/io002-adc/adc-observation-projection.json",
      captureTimeoutSeconds: 360,
    },
  };
}

function installHttp(contract: Contract, millivolts = 1_198) {
  const original = globalThis.fetch;
  globalThis.fetch = async (input) => {
    const target = new URL(String(input));
    if (target.pathname.endsWith("/logs")) {
      return new Response(`${retained(7, 9)}\n${retained(8, 10)}\n`, { status: 200 });
    }
    return new Response(JSON.stringify(snapshot(contract, 7, 9, 11, 500, millivolts)), {
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

function fakePort(inputValidation: ProcessOutcome = ok()): ProcessPort {
  return createFakeProcessPort(async (spec) => {
    if (spec.args[0] === "flash-monitor") {
      const root = String(spec.args[spec.args.indexOf("--evidence-dir") + 1]);
      await writeFile(path.join(root, "flash-monitor.classifier-input.log"), [
        "safe_state: mining=disabled asic_work_submission=disabled hardware_control=disabled",
        `runtime_origin session=${session} device_url=http://private-device.test redacted=true`,
        "",
      ].join("\n"), { mode: 0o600 });
      await writeFile(path.join(root, "flash-command-evidence.private.json"), "{}\n", { mode: 0o600 });
      return ok();
    }
    if (spec.args[0] === "flash") return ok();
    if (spec.program === "adc-input-validator") return inputValidation;
    if (spec.program === "system-validator" || spec.program === "adc-validator") return ok();
    if (spec.program === "git") {
      if (spec.args[0] === "status") return ok();
      if (spec.args[0] === "-C") return ok(`${referenceCommit}\n`);
      if (spec.args[0] === "rev-parse") return ok(`${sourceCommit}\n`);
    }
    throw new Error("unexpected child process");
  });
}

async function capture(value: Awaited<ReturnType<typeof fixture>>, port: ProcessPort) {
  return captureAdcObservationEvidence(
    value.root,
    value.options,
    port,
    "flash",
    "git",
    "system-validator",
    "adc-input-validator",
    "adc-validator",
    websocketFactory(snapshot(value.contract, 8, 10, 12, 1_000, 1_201)),
    value.admittedPlanSha256,
  );
}

test("ready ADC capture publishes aggregate-only v1 evidence", async () => {
  // Arrange
  const value = await fixture("ready");
  const restore = installHttp(value.contract);

  try {
    // Act
    const evidence = await capture(value, fakePort());
    const document = await readFile(value.projection, "utf8");

    // Assert
    assert.equal(evidence.adc.producer_cadence_ms, 500);
    assert.equal(evidence.adc.sequence_not_regressed, true);
    assert.equal((await stat(value.projection)).mode & 0o777, 0o644);
    assert.doesNotMatch(document, /private-device|private-port|bootSession|acquiredAtMs|coreVoltageActual/u);
  } finally {
    restore();
    await rm(value.root, { recursive: true });
  }
});

test("lossless ADC input rejection withholds final evidence", async () => {
  // Arrange
  const value = await fixture("invalid-input");
  const restore = installHttp(value.contract);

  try {
    // Act
    const promise = capture(value, fakePort({ ...ok(), exitCode: 1 }));

    // Assert
    await assert.rejects(promise, AdcObservationEvidenceError);
    await assert.rejects(readFile(value.projection, "utf8"), { code: "ENOENT" });
  } finally {
    restore();
    await rm(value.root, { recursive: true });
  }
});

test("checked-in ADC source semantics match the immutable contract", async () => {
  // Arrange
  const maybeRunfiles = process.env["RUNFILES_DIR"];
  const workspaceRoot = maybeRunfiles === undefined ? process.cwd() : path.join(maybeRunfiles, "_main");

  // Act
  const result = validateAdcObservationSourceSemantics(workspaceRoot);

  // Assert
  await assert.doesNotReject(result);
});

test("real child processes own flash git and validator boundaries", async () => {
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
  if (!documents.every((document) => document.includes('"coreVoltageActualStatus"'))) process.exitCode = 1;
} else {
  const value = JSON.parse(await readFile(args[0], "utf8"));
  if (!["bitaxe-system-info-evidence-v1", "bitaxe-adc-observation-evidence-v1"].includes(value.schema_version)) process.exitCode = 1;
}
`);
  await chmod(child, 0o700);

  try {
    // Act
    const evidence = await captureAdcObservationEvidence(
      value.root,
      value.options,
      createLocalProcessPort({ cwd: value.root, timeoutMs: 5_000 }),
      child,
      child,
      child,
      child,
      child,
      websocketFactory(snapshot(value.contract, 8, 10, 12, 1_000, 1_201)),
      value.admittedPlanSha256,
    );

    // Assert
    assert.equal(evidence.schema_version, "bitaxe-adc-observation-evidence-v1");
  } finally {
    restore();
    await rm(value.root, { recursive: true });
  }
});
