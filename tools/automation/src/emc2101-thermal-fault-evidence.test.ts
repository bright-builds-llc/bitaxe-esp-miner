import assert from "node:assert/strict";
import { chmod, mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import {
  captureEmc2101ThermalFaultEvidence,
  Emc2101ThermalFaultEvidenceError,
} from "./emc2101-thermal-fault-evidence.js";
import {
  createFakeProcessPort,
  createLocalProcessPort,
  type ProcessOutcome,
  type ProcessPort,
} from "./process.js";
import { internalCommandSpec } from "./contracts.generated.js";
import type { WebSocketClient, WebSocketFactory } from "./websocket.js";

const sourceCommit = "a".repeat(40);
const referenceCommit = "c1915b0a63bfabebdb95a515cedfee05146c1d50";
const appElfSha256 = "b".repeat(64);
const session = "1".repeat(32);
const nodeProgram = process.env["JS_BINARY__NODE_BINARY"] ?? process.execPath;
const planRelative = "docs/parity/work-plans/20260815T201754Z-THR-001/PLAN.md";
const priorRelative = "docs/parity/evidence/thr001-emc2101-thermal/thermal-projection.json";

type Contract = {
  readonly fields: Readonly<Record<string, {
    readonly type: "array" | "boolean" | "number" | "object" | "string";
    readonly presence: "always" | "block_found";
  }>>;
};

const ok = (stdout = ""): ProcessOutcome => ({
  exitCode: 0,
  stdout,
  stderr: "",
  timedOut: false,
});

async function sourceInput(relative: string): Promise<string> {
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
  throw new Error(`test source input is missing: ${relative}`);
}

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

function sampleFor(type: Contract["fields"][string]["type"]): unknown {
  if (type === "array") return [];
  if (type === "boolean") return false;
  if (type === "number") return 0;
  if (type === "object") return {};
  return "";
}

function snapshot(contract: Contract, revision: number, sequence: number): Record<string, unknown> {
  const fields: Record<string, unknown> = {};
  for (const [name, rule] of Object.entries(contract.fields)) {
    if (rule.presence === "always") fields[name] = sampleFor(rule.type);
  }
  return {
    ...fields,
    blockFound: 0,
    bootSession: session,
    operatorSnapshotRevision: revision,
    sourceCommit,
    referenceCommit,
    appElfSha256,
    runtimeHealth: runtimeHealth(sequence),
    temp: 50,
    chipTempStatus: {
      state: "fresh",
      stamp: { bootSession: 1, sequence, acquiredAtMs: sequence * 1_000 },
    },
  };
}

function retained(revision: number, sequence: number): string {
  return [
    `runtime_health boot_session=${session}`,
    `operator_snapshot_revision=${String(revision)}`,
    "self_test=unavailable supervisor=available checkpoint_category=telemetry",
    `checkpoint_sequence=${String(sequence)}`,
    "checkpoint_age_millis=100 checkpoint_health=healthy",
    "task_watchdog_participation=participating task_watchdog_reason=feed_fresh",
    `task_watchdog_feed_sequence=${String(sequence + 2)}`,
    "task_watchdog_feed_age_millis=50",
    "task_watchdog_read_outcome=stable task_watchdog_owner_phase=waiting_inbox task_watchdog_owner_subphase=unavailable task_watchdog_wait_state=within_deadline redacted=true",
  ].join(" ");
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

async function fixture(name: string) {
  const root = await mkdtemp(path.join(os.tmpdir(), `bitaxe-thermal-fault-${name}-`));
  await writeFile(path.join(root, "MODULE.bazel"), "module(name = \"fixture\")\n");
  const contractRelative = "crates/bitaxe-api/fixtures/api/system-info-contract-v1.json";
  const contractDocument = await sourceInput(contractRelative);
  const contract = JSON.parse(contractDocument) as Contract;
  await mkdir(path.dirname(path.join(root, contractRelative)), { recursive: true });
  await writeFile(path.join(root, contractRelative), contractDocument);
  const plan = await sourceInput(planRelative);
  await mkdir(path.dirname(path.join(root, planRelative)), { recursive: true });
  await writeFile(path.join(root, planRelative), plan);
  await writeFile(path.join(root, "TASKS.md"), [
    "### task-parity-thr001-emc2101-live-thermal | fixture",
    `Plan: \`${planRelative}\`.`,
    "Schema: `bitaxe-emc2101-thermal-fault-evidence-v1`.",
    "Attempt: `attempt-007`.",
    "",
  ].join("\n"));
  await mkdir(path.dirname(path.join(root, priorRelative)), { recursive: true });
  await writeFile(path.join(root, priorRelative), "{}\n");
  await mkdir(path.join(root, "inputs"));
  const manifest = path.join(root, "inputs/package.json");
  const credentials = path.join(root, "inputs/wifi.json");
  await writeFile(manifest, JSON.stringify({
    source_commit: sourceCommit,
    reference_commit: referenceCommit,
    app_elf_sha256: appElfSha256,
  }));
  await writeFile(credentials, "{}\n", { mode: 0o600 });
  const wrapper = path.join(root, "scratch/thr001-emc2101-fault/wrapper-007");
  await mkdir(wrapper, { recursive: true, mode: 0o700 });
  await chmod(wrapper, 0o700);
  for (const output of ["detector.stdout", "detector.stderr", "capture.stdout", "capture.stderr"]) {
    await writeFile(path.join(wrapper, output), "", { mode: 0o600 });
    await chmod(path.join(wrapper, output), 0o600);
  }
  return {
    root,
    contract,
    manifest,
    credentials,
    projection: path.join(
      root,
      "docs/parity/evidence/thr001-emc2101-thermal/thermal-fault-projection-attempt-007.json",
    ),
    options: {
      privateRoot: "scratch/thr001-emc2101-fault/attempt-007",
      packageManifest: manifest,
      wifiCredentials: credentials,
      detectorOutput: "scratch/thr001-emc2101-fault/wrapper-007/detector.stdout",
      port: "/dev/private-port",
      projection:
        "docs/parity/evidence/thr001-emc2101-thermal/thermal-fault-projection-attempt-007.json",
      captureTimeoutSeconds: 120,
    },
  };
}

function espLogLine(
  payload: string,
  uptimeMs: number,
  tag = "bitaxe_firmware",
): string {
  return `I (${String(uptimeMs)}) ${tag}: ${payload}`;
}

type MarkerEnvelope =
  | "bare"
  | "malformed_timestamp"
  | "nested_replay_tag"
  | "warning"
  | "wrong_module"
  | "wrong_tag";

async function writeFlashEvidence(
  args: readonly string[],
  malformedMarkers = false,
  lateAttachment = false,
  maybeMarkerEnvelope?: MarkerEnvelope,
  wrongMarkerOrder = false,
  extraMarkerPayload = false,
): Promise<void> {
  const evidenceRoot = String(args[args.indexOf("--evidence-dir") + 1]);
  await mkdir(evidenceRoot, { recursive: true, mode: 0o700 });
  await chmod(evidenceRoot, 0o700);
  const stimulus = args.includes("--thermal-fault-stimulus-intent");
  const markerLine = (payload: string, uptimeMs: number, replay = false) => {
    if (maybeMarkerEnvelope === "bare") return payload;
    if (maybeMarkerEnvelope === "malformed_timestamp") {
      return `I (invalid) bitaxe_firmware: ${payload}`;
    }
    if (maybeMarkerEnvelope === "warning") {
      return `W (${String(uptimeMs)}) bitaxe_firmware: ${payload}`;
    }
    if (maybeMarkerEnvelope === "wrong_tag") {
      return `I (${String(uptimeMs)}) other_firmware: ${payload}`;
    }
    if (maybeMarkerEnvelope === "wrong_module") {
      return `I (${String(uptimeMs)}) bitaxe_firmware::other_module: ${payload}`;
    }
    if (maybeMarkerEnvelope === "nested_replay_tag") {
      return `I (${String(uptimeMs)}) bitaxe_firmware::boot_evidence::nested: ${payload}`;
    }
    return espLogLine(
      payload,
      uptimeMs,
      replay ? "bitaxe_firmware::boot_evidence" : "bitaxe_firmware",
    );
  };
  const log = stimulus
    ? [
      espLogLine(
        "safe_state: mining=disabled asic_work_submission=disabled hardware_control=disabled",
        1_000,
      ),
      ...(lateAttachment
        ? [
          markerLine("thermal_fault_stimulus state=fault_observed redacted=true", 2_000),
          markerLine("thermal_fault_stimulus state=recovered redacted=true", 3_000),
        ]
        : []),
      ...(wrongMarkerOrder
        ? [
          markerLine("thermal_fault_stimulus state=baseline_ready redacted=true", 10_000, lateAttachment),
          markerLine("thermal_fault_stimulus state=recovered redacted=true", 10_001, lateAttachment),
          markerLine("thermal_fault_stimulus state=fault_observed redacted=true", 10_002, lateAttachment),
        ]
        : [
          markerLine("thermal_fault_stimulus state=baseline_ready redacted=true", 10_000, lateAttachment),
          markerLine("thermal_fault_stimulus state=fault_observed redacted=true", 10_001, lateAttachment),
          ...(malformedMarkers
            ? []
            : [markerLine(
              `thermal_fault_stimulus state=recovered redacted=true${extraMarkerPayload ? " extra=true" : ""}`,
              10_002,
              lateAttachment,
            )]),
        ]),
      "",
    ].join("\n")
    : [
      "safe_state: mining=disabled asic_work_submission=disabled hardware_control=disabled",
      `runtime_origin session=${session} device_url=http://private-device.test redacted=true`,
      "",
    ].join("\n");
  await writeFile(path.join(evidenceRoot, "flash-monitor.classifier-input.log"), log, {
    mode: 0o600,
  });
  await chmod(path.join(evidenceRoot, "flash-monitor.classifier-input.log"), 0o600);
  await writeFile(path.join(evidenceRoot, "flash-command-evidence.private.json"), "{}\n", {
    mode: 0o600,
  });
  await chmod(path.join(evidenceRoot, "flash-command-evidence.private.json"), 0o600);
}

function fakePort(configuration: {
  readonly stimulusOutcome?: ProcessOutcome;
  readonly malformedMarkers?: boolean;
  readonly lateAttachment?: boolean;
  readonly maybeMarkerEnvelope?: MarkerEnvelope;
  readonly wrongMarkerOrder?: boolean;
  readonly extraMarkerPayload?: boolean;
  readonly validatorOutcome?: ProcessOutcome;
} = {}): ProcessPort {
  return createFakeProcessPort(async (spec) => {
    if (spec.args[0] === "flash-monitor") {
      const stimulus = spec.args.includes("--thermal-fault-stimulus-intent");
      await writeFlashEvidence(
        spec.args,
        configuration.malformedMarkers,
        configuration.lateAttachment,
        configuration.maybeMarkerEnvelope,
        configuration.wrongMarkerOrder,
        configuration.extraMarkerPayload,
      );
      return stimulus ? configuration.stimulusOutcome ?? ok() : ok();
    }
    if (spec.program === "validator") return configuration.validatorOutcome ?? ok();
    if (spec.program === "git") {
      if (spec.args[0] === "status") return ok();
      if (spec.args[0] === "-C") return ok(`${referenceCommit}\n`);
      return ok(`${sourceCommit}\n`);
    }
    throw new Error("unexpected child process");
  });
}

function installHttp(contract: Contract) {
  const original = globalThis.fetch;
  globalThis.fetch = async (input) => {
    const target = new URL(String(input));
    if (target.pathname.endsWith("/logs")) {
      return new Response(`${retained(7, 9)}\n${retained(8, 10)}\n`, { status: 200 });
    }
    return new Response(JSON.stringify(snapshot(contract, 7, 9)), {
      status: 200,
      headers: { "content-type": "application/json" },
    });
  };
  return () => {
    globalThis.fetch = original;
  };
}

async function capture(value: Awaited<ReturnType<typeof fixture>>, port: ProcessPort) {
  return captureEmc2101ThermalFaultEvidence(
    value.root,
    value.options,
    port,
    "flash",
    "git",
    "validator",
    "validator",
    websocketFactory(snapshot(value.contract, 8, 10)),
  );
}

test("ready stimulus restores ordinary package and publishes v1 evidence", async () => {
  const value = await fixture("ready");
  const restore = installHttp(value.contract);
  try {
    const evidence = await capture(value, fakePort());
    assert.equal(evidence.stimulus.injected_sample_count, 5);
    assert.equal(evidence.restoration.stimulus_not_replayed, true);
    assert.equal(evidence.recovery_used, true);
    assert.equal(JSON.parse(await readFile(value.projection, "utf8")).schema_version,
      "bitaxe-emc2101-thermal-fault-evidence-v1");
  } finally {
    restore();
    await rm(value.root, { recursive: true, force: true });
  }
});

test("malformed marker sequence withholds evidence after confirmed restoration", async () => {
  const value = await fixture("malformed");
  const restore = installHttp(value.contract);
  try {
    await assert.rejects(
      capture(value, fakePort({ malformedMarkers: true })),
      (error: unknown) => {
        assert.ok(error instanceof Emc2101ThermalFaultEvidenceError);
        assert.equal(error.category, "evidence_invalid");
        assert.equal(error.publicValue["recovery_complete"], true);
        return true;
      },
    );
    await assert.rejects(readFile(value.projection), { code: "ENOENT" });
  } finally {
    restore();
    await rm(value.root, { recursive: true, force: true });
  }
});

test("out-of-order marker sequence withholds evidence after confirmed restoration", async () => {
  const value = await fixture("out-of-order");
  const restore = installHttp(value.contract);
  try {
    await assert.rejects(
      capture(value, fakePort({ wrongMarkerOrder: true })),
      (error: unknown) => {
        assert.ok(error instanceof Emc2101ThermalFaultEvidenceError);
        assert.equal(error.category, "evidence_invalid");
        assert.equal(error.publicValue["recovery_complete"], true);
        return true;
      },
    );
    await assert.rejects(readFile(value.projection), { code: "ENOENT" });
  } finally {
    restore();
    await rm(value.root, { recursive: true, force: true });
  }
});

test("marker payload suffix withholds evidence after confirmed restoration", async () => {
  const value = await fixture("extra-payload");
  const restore = installHttp(value.contract);
  try {
    await assert.rejects(
      capture(value, fakePort({ extraMarkerPayload: true })),
      (error: unknown) => {
        assert.ok(error instanceof Emc2101ThermalFaultEvidenceError);
        assert.equal(error.category, "evidence_invalid");
        assert.equal(error.publicValue["recovery_complete"], true);
        return true;
      },
    );
    await assert.rejects(readFile(value.projection), { code: "ENOENT" });
  } finally {
    restore();
    await rm(value.root, { recursive: true, force: true });
  }
});

test("noncanonical marker envelopes cannot satisfy the production witness", async (context) => {
  for (const markerEnvelope of [
    "bare",
    "malformed_timestamp",
    "nested_replay_tag",
    "warning",
    "wrong_module",
    "wrong_tag",
  ] as const) {
    await context.test(markerEnvelope, async () => {
      const value = await fixture(`invalid-envelope-${markerEnvelope}`);
      const restore = installHttp(value.contract);
      try {
        await assert.rejects(
          capture(value, fakePort({ maybeMarkerEnvelope: markerEnvelope })),
          (error: unknown) => {
            assert.ok(error instanceof Emc2101ThermalFaultEvidenceError);
            assert.equal(error.category, "evidence_invalid");
            assert.equal(error.publicValue["recovery_complete"], true);
            return true;
          },
        );
        await assert.rejects(readFile(value.projection), { code: "ENOENT" });
      } finally {
        restore();
        await rm(value.root, { recursive: true, force: true });
      }
    });
  }
});

test("retained marker replay closes a late monitor attachment", async () => {
  const value = await fixture("late-attachment");
  const restore = installHttp(value.contract);
  try {
    const evidence = await capture(value, fakePort({ lateAttachment: true }));
    assert.equal(evidence.stimulus.injected_sample_count, 5);
    assert.equal(evidence.cleanup_complete, true);
  } finally {
    restore();
    await rm(value.root, { recursive: true, force: true });
  }
});

test("non-ready stimulus preserves hardware category when restoration succeeds", async () => {
  const value = await fixture("blocked");
  const restore = installHttp(value.contract);
  try {
    await assert.rejects(
      capture(value, fakePort({ stimulusOutcome: { ...ok(), exitCode: 3 } })),
      (error: unknown) => {
        assert.ok(error instanceof Emc2101ThermalFaultEvidenceError);
        assert.equal(error.category, "hardware_blocked");
        assert.equal(error.publicValue["secondary_recovery_failure"], false);
        return true;
      },
    );
  } finally {
    restore();
    await rm(value.root, { recursive: true, force: true });
  }
});

test("stimulus timeout preserves its primary category through restoration", async () => {
  const value = await fixture("timeout");
  const restore = installHttp(value.contract);
  try {
    await assert.rejects(
      capture(value, fakePort({ stimulusOutcome: { ...ok(), timedOut: true } })),
      (error: unknown) => {
        assert.ok(error instanceof Emc2101ThermalFaultEvidenceError);
        assert.equal(error.category, "timeout");
        assert.equal(error.publicValue["recovery_complete"], true);
        return true;
      },
    );
    await assert.rejects(readFile(value.projection), { code: "ENOENT" });
  } finally {
    restore();
    await rm(value.root, { recursive: true, force: true });
  }
});

test("real child process owns stimulus and restoration evidence files", async () => {
  const value = await fixture("real-child");
  const restore = installHttp(value.contract);
  const child = path.join(value.root, "child.mjs");
  await writeFile(child, `#!${nodeProgram}
import { chmod, mkdir, writeFile } from "node:fs/promises";
import path from "node:path";
const args = process.argv.slice(1);
if (args.includes("flash-monitor")) {
  const root = args[args.indexOf("--evidence-dir") + 1];
  await mkdir(root, { recursive: true, mode: 0o700 }); await chmod(root, 0o700);
  const stimulus = args.includes("--thermal-fault-stimulus-intent");
  const lines = stimulus ? [
    "I (1000) bitaxe_firmware: safe_state: mining=disabled asic_work_submission=disabled hardware_control=disabled",
    "I (1001) bitaxe_firmware: thermal_fault_stimulus state=fault_observed redacted=true",
    "I (1002) bitaxe_firmware: thermal_fault_stimulus state=recovered redacted=true",
    "I (1003) bitaxe_firmware::boot_evidence: thermal_fault_stimulus state=baseline_ready redacted=true",
    "I (1004) bitaxe_firmware::boot_evidence: thermal_fault_stimulus state=fault_observed redacted=true",
    "I (1005) bitaxe_firmware::boot_evidence: thermal_fault_stimulus state=recovered redacted=true",
  ] : [
    "safe_state: mining=disabled asic_work_submission=disabled hardware_control=disabled",
    "runtime_origin session=${session} device_url=http://private-device.test redacted=true",
  ];
  await writeFile(path.join(root, "flash-monitor.classifier-input.log"), lines.join("\\n") + "\\n", { mode: 0o600 });
  await writeFile(path.join(root, "flash-command-evidence.private.json"), "{}\\n", { mode: 0o600 });
} else if (args.includes("-C")) process.stdout.write("${referenceCommit}\\n");
else if (args.includes("rev-parse")) process.stdout.write("${sourceCommit}\\n");
`, { mode: 0o700 });
  await chmod(child, 0o700);
  const processPort = createLocalProcessPort({ cwd: value.root, timeoutMs: 5_000 });
  const childSpec = (args: readonly string[]) =>
    internalCommandSpec(child, [...args], (input: unknown) => input);
  try {
    const current = await processPort.run(childSpec(["rev-parse", "HEAD"]));
    const pushed = await processPort.run(childSpec(["rev-parse", "origin/main"]));
    const reference = await processPort.run(childSpec([
      "-C",
      path.join(value.root, "reference/esp-miner"),
      "rev-parse",
      "HEAD",
    ]));
    const dirty = await processPort.run(childSpec(["status", "--porcelain", "--untracked-files=no"]));
    assert.equal(current.exitCode, 0, current.stderr);
    assert.equal(pushed.exitCode, 0, pushed.stderr);
    assert.equal(reference.exitCode, 0, reference.stderr);
    assert.equal(dirty.exitCode, 0, dirty.stderr);
    assert.equal(current.stdout.trim(), sourceCommit);
    assert.equal(pushed.stdout.trim(), sourceCommit);
    assert.equal(reference.stdout.trim(), referenceCommit);
    assert.equal(dirty.stdout.trim(), "");
    const evidence = await captureEmc2101ThermalFaultEvidence(
      value.root,
      value.options,
      processPort,
      child,
      child,
      child,
      child,
      websocketFactory(snapshot(value.contract, 8, 10)),
    );
    assert.equal(evidence.cleanup_complete, true);
  } finally {
    restore();
    await rm(value.root, { recursive: true, force: true });
  }
});
