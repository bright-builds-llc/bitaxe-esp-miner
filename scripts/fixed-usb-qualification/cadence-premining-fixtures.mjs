import { copyFile, mkdir, readFile, writeFile } from "node:fs/promises";
import { dirname, relative, resolve } from "node:path";
import { cadencePreflight } from "./cadence-preflight.mjs";
import { BUNDLE, PAGE, digest, fileDigest, inspectPackage, nonce, writeNew } from "./contract.mjs";
import { verifyArtifactSnapshot } from "./snapshot.mjs";
import { inventory } from "./cadence-premining-evidence.mjs";
import { LEGACY_PREMINING_AUDITOR } from "./cadence-premining.mjs";
import { unissuedFixture } from "./cadence-premining-fixture-base.mjs";

export async function artifactFixture(f, sourceCommit = "f".repeat(40)) {
  const sourceRoot = resolve(f.base, `source-artifacts-${sourceCommit}`),
    entries = [];
  async function artifact(path, bytes) {
    const output = resolve(sourceRoot, path);
    await mkdir(dirname(output), { recursive: true, mode: 0o700 });
    await writeFile(output, bytes, { mode: 0o600 });
    entries.push({ path, sha256: digest(bytes), length: Buffer.byteLength(bytes) });
  }
  const kinds = [
    "firmware_elf",
    "firmware_ota_image",
    "www_spiffs_image",
    "factory_merged_image",
    "partition_table",
    "otadata_initial",
    "bootloader",
    "partition_table_binary",
  ];
  const artifacts = [];
  for (const kind of kinds) {
    const bytes = Buffer.from(`fixture ${kind}`);
    await artifact(`firmware/${kind}.bin`, bytes);
    artifacts.push({ kind, path: `${kind}.bin`, sha256: digest(bytes) });
  }
  const geometry = [
    ["bootloader", 0],
    ["partition_table_binary", 0x8000],
    ["firmware_ota_image", 0x10000],
    ["www_spiffs_image", 0x410000],
    ["otadata_initial", 0xf10000],
  ];
  const manifest = {
    schema_version: 4,
    source_commit: sourceCommit,
    reference_commit: "e".repeat(40),
    build_identity: { source_dirty: false },
    app_elf_sha256: artifacts[0].sha256,
    artifacts,
    update_segments: geometry.map(([kind, offset]) => ({
      artifact_kind: kind,
      offset,
      length: entries.find((row) => row.path === `firmware/${kind}.bin`).length,
    })),
  };
  await artifact("firmware/bitaxe-ultra205-package.json", Buffer.from(JSON.stringify(manifest)));
  for (const name of ["license-inventory", "provenance-manifest"])
    await artifact(`firmware/docs/release/${name}.md`, Buffer.from("fixture provenance"));
  await artifact(`gate/${BUNDLE}`, Buffer.from("fixture bundle"));
  await artifact(`gate/${PAGE}`, Buffer.from("fixture page"));
  const snapshot = {
    firmware_commit: manifest.source_commit,
    gate_commit: f.oldContext.gate_commit,
    ...(await inspectPackage(resolve(sourceRoot, "firmware/bitaxe-ultra205-package.json"), manifest.source_commit, () =>
      resolve(sourceRoot, "firmware"),
    )),
    gate_bundle_sha256: digest("fixture bundle"),
    gate_page_sha256: digest("fixture page"),
    gate_page_relative_path: PAGE,
  };
  return { sourceRoot, entries, snapshot };
}

export function baselineFixture(context) {
  const state = {
    schema: "worker-serial-acceptance-v1",
    gateCommit: context.gate_commit,
    expectedFirmwareSourceCommit: context.firmware_commit,
    expectedAppElfSha256: context.app_elf_sha256,
    status: "closed",
    connected: false,
    running: false,
    heartbeatSuppressed: false,
    renewalsConfirmed: 0,
    deviceRestorationConfirmed: false,
    deviceBaselineConfirmed: true,
    deviceLeaseInactive: true,
    serialOwnershipReleased: true,
    preservation: {
      schema: "worker-preservation-continuity-v1",
      baseline_id: nonce(),
      device_identity_match: true,
      settings_match: true,
      authorization_high_water_match: true,
      mine_on_boot: false,
    },
    cadence: { schema: "worker-cadence-browser-v1", enabled: true, suppressionRequested: false },
  };
  return state;
}

export async function cycleFixture(root, context, state, {paddingBytes=65000}={}) {
  const records = [],
    probe = { paddingBytes, requestPayloadBytes: 65536, responsePayloadBytes: 65536 };
  await writeNew(resolve(root, "detector.device.private.json"), { port: "/dev/cu.fixture", usb_profile: "serial_jtag_runtime" });
  const deviceHash = await fileDigest(resolve(root, "detector.device.private.json"));
  for (let n = 0; n <= 4; n++) {
    await mkdir(resolve(root, `flash-${n}`), { mode: 0o700 });
    await writeNew(resolve(root, `flash-${n}/flash-command-evidence.json`), {
      flash_status: "completed",
      monitor_evidence_status: "trusted",
      trusted_output: true,
      commit_ready: true,
      firmware_commit: context.firmware_commit,
      observed_firmware_commit: context.firmware_commit,
      reference_commit: context.reference_commit,
      observed_reference_commit: "Unavailable",
      trust_basis: "fixed_serial",
      nvs_seed_status: "not_provided",
      redaction_mode: "commit-redacted",
      capture_timeout_seconds: 30,
      manifest_path: "[redacted-path]",
      timestamp: String(10 + n * 10),
      fixed_serial_assessment: {
        execution_present: true,
        safe_baseline_confirmed: true,
        startup_complete: true,
        startup_failed: false,
        stable_boot: true,
        issues: [],
      },
    });
    await writeNew(resolve(root, `flash-${n}.observation.json`), {
      schema: "hello-passive-command-observation-v1",
      rootObserved: true,
      observations: 2,
      remaining: [],
      failures: [],
    });
    if (!n) continue;
    const b = n * 2 - 1,
      a = n * 2,
      before = structuredClone(state),
      after = { ...structuredClone(state), status: "ready", connected: true, serialOwnershipReleased: false, probe };
    records.push({ sequence: b, state: before }, { sequence: a, state: after });
    const report = {
      schema: "fixed-usb-cycle-report-v1",
      cycle: n,
      firmware_commit: context.firmware_commit,
      app_elf_sha256: context.app_elf_sha256,
      baseline_id: state.preservation.baseline_id,
      browser_released: true,
      flash_success: true,
      runtime_identity_match: true,
      cleanup_complete: true,
      device_identity_match: true,
      settings_match: true,
      authorization_high_water_match: true,
      probe_request_bytes: 65536,
      probe_response_bytes: 65536,
      mine_on_boot: false,
    };
    await writeNew(resolve(root, `cycle-${n}.json`), report);
    await writeNew(resolve(root, `cycle-${n}.observed-input.json`), report);
    const beforePath = resolve(root, `cycle-${n}.before_flash.json`),
      afterPath = resolve(root, `cycle-${n}.after_flash.json`),
      probePath = resolve(root, `cycle-${n}.browser-observation.json`),
      flashPath = resolve(root, `flash-${n}/flash-command-evidence.json`);
    for (const phase of ["before_flash", "after_flash"]) {
      const afterFlash = phase === "after_flash",
        processPath = resolve(root, `flash-${n}.observation.json`);
      await writeNew(resolve(root, `cycle-${n}.${phase}.json`), {
        schema: "hello-cycle-host-observation-v1",
        cycle: n,
        phase,
        observed_at_unix_ms: (10 + n * 10) * 1000 + (afterFlash ? 1000 : -1000),
        observer: "parent-observed",
        browser_serial_released: true,
        usb_holder_count: 0,
        owned_command_children_remaining: 0,
        ...(afterFlash ? { flash_command_exit_code: 0 } : {}),
      });
      await writeNew(resolve(root, `cycle-${n}.${phase}-source.json`), {
        journal_sequence: b,
        process_observation: relative(context.firmware_root, processPath),
        process_observation_sha256: await fileDigest(processPath),
        device_observation_sha256: deviceHash,
      });
    }
    await writeNew(probePath, {
      schema: "hello-cycle-browser-observation-v1",
      cycle: n,
      observer: "parent-observed",
      fresh_connect_completed: true,
      probe_completed: true,
      after_ready_sequence: a,
      observed_at_unix_ms: (10 + n * 10) * 1000 + 2000,
      result: probe,
    });
    const inputPath = resolve(root, `cycle-${n}.audit-input.json`);
    await writeNew(inputPath, {
      schema: "hello-cycle-audit-input-v1",
      cycle: n,
      before_closed_sequence: b,
      after_ready_sequence: a,
      before_host_witness: beforePath,
      after_host_witness: afterPath,
      probe_witness: probePath,
      flash_record: flashPath,
    });
    const inputs = {};
    for (const [key, path] of Object.entries({
      observations: inputPath,
      before_host: beforePath,
      after_host: afterPath,
      probe: probePath,
      flash: flashPath,
    }))
      inputs[key] = { path, sha256: await fileDigest(path) };
    await writeNew(resolve(root, `cycle-${n}.observation-audit.json`), {
      schema: "hello-cycle-observation-audit-v1",
      cycle: n,
      context_sha256: digest(JSON.stringify(context)),
      before_closed_sequence: b,
      after_ready_sequence: a,
      before_state_sha256: digest(JSON.stringify(before)),
      after_state_sha256: digest(JSON.stringify(after)),
      inputs,
      report_sha256: digest(JSON.stringify(report)),
      package_reference_bound: true,
      runtime_reference_observed: false,
      receipt_manifest_path_observed: false,
      hardware_execution_claimed_by_helper: false,
    });
  }
  return records;
}

async function observerFixture(root, context) {
  await writeNew(resolve(root, "cadence-idle-arm.json"), {
    context_sha256: digest(JSON.stringify(context)),
    phase: "idle",
    arm: { schema: "worker-telemetry-cadence-arm-v1", phase: "idle", armedAtUs: 1000000, generation: 0 },
    started_sequence: 8,
    started_at_unix_ms: 100200,
  });
  const events = ["connected", "arrival", "closed"].map((event, index) => ({
    sequence: index + 1,
    observedAtUnixMs: [100100, 110100, 163100][index],
    event: {
      schema: "cpu0-cadence-observer-v1",
      event,
      elapsedMs: [0, 10000, 63000][index],
      messageCount: index ? 1 : 0,
      totalBytes: index ? 100 : 0,
      byteCount: index === 1 ? 100 : 0,
      reason: index === 2 ? "requested" : null,
    },
  }));
  const observerBytes = events.map((row) => JSON.stringify(row) + "\n").join("");
  await writeFile(resolve(root, "cadence-observer.jsonl"), observerBytes, { mode: 0o600 });
  await writeNew(resolve(root, "cadence-observer-result.json"), {
    schema: "worker-cadence-observer-result-v1",
    connected: true,
    closed: true,
    exitCode: 0,
    reason: "requested",
    cleanupComplete: true,
    startedAtUnixMs: 100000,
    connectedAtUnixMs: 100100,
    closedAtUnixMs: 163102,
    messageCount: 1,
    totalBytes: 100,
    eventCount: 3,
    journalSha256: digest(observerBytes),
  });
}

async function failureFixture(root) {
  const ledger = {
    schema: "worker-qualification-ledger-v1",
    next_ordinal: 16,
    last_completed_ordinal: 15,
    pending: false,
    total_charged_ms: 1200000,
  };
  await writeNew(resolve(root, "operator-failure.json"), {
    schema: "cpu0-cadence-operator-failure-v1",
    source: "parent-observed",
    earliest_failure: {
      stage: "idle_review",
      category: "probe_admission",
      class: "SerialFailure",
      method: "workerAcceptance.cadenceReview",
      guard: "before_possession_refresh",
    },
    diagnostic_recovery: {
      fresh_authenticated_reconnect: true,
      review_result: "probe_admission",
      guard: "after_possession_refresh",
      frozen_summary_collected: false,
    },
    qualification_pass: false,
    usb_phase_started: false,
    mining_phase_started: false,
    allowance_issued: false,
    ledger,
    final_state: {
      status: "closed",
      connected: false,
      running: false,
      deviceBaselineConfirmed: true,
      deviceLeaseInactive: true,
      serialOwnershipReleased: true,
    },
    raw_network_payloads_persisted: false,
  });
  await writeNew(resolve(root, "operator-cleanup.json"), {
    schema: "worker-cadence-host-cleanup-v1",
    source: "parent-observed",
    browser_closed: true,
    supervisor_exited: true,
    supervisor_exit_code: 0,
    listener_absent: true,
    owned_children_absent: true,
    serial_holders_absent: true,
  });
}

async function sealFixture(f, root, context) {
  const seal = {
    schema: "cpu0-cadence-failed-inventory-v1",
    outcome: "failed_preparation",
    qualification_pass: false,
    continuation_authority: false,
    auditor_sha256: LEGACY_PREMINING_AUDITOR,
    context_sha256: digest(JSON.stringify(context)),
    artifact_snapshot_sha256: await fileDigest(resolve(root, "artifact-snapshot.json")),
    unissued_closure_sha256: context.unissued_predecessor.closure_sha256,
    samples_sha256: await fileDigest(resolve(root, "iterative.samples.jsonl")),
    operator_failure_sha256: await fileDigest(resolve(root, "operator-failure.json")),
    idle_arm_sha256: await fileDigest(resolve(root, "cadence-idle-arm.json")),
    observer_result_sha256: await fileDigest(resolve(root, "cadence-observer-result.json")),
    inventory: await inventory(root),
  };
  await writeNew(resolve(root, "failed-inventory.json"), seal);
  Object.defineProperty(f.operations, "expectedPreminingInventorySha256", {
    value: await fileDigest(resolve(root, "failed-inventory.json")),
    enumerable: true,
  });
  return seal;
}

export async function preminingFixture(t) {
  const f = await unissuedFixture(t);
  const { sourceRoot, entries, snapshot } = await artifactFixture(f);
  f.operations.inspectSources = async () => snapshot;
  await cadencePreflight(f.options, f.operations);
  const root = f.options.privateRoot,
    { context } = JSON.parse(await readFile(resolve(root, "context.json"), "utf8"));
  for (const entry of entries) {
    const output = resolve(root, "qualified-artifacts", entry.path);
    await mkdir(dirname(output), { recursive: true, mode: 0o700 });
    await copyFile(resolve(sourceRoot, entry.path), output);
  }
  await writeNew(resolve(root, "artifact-snapshot.json"), {
    schema: "fixed-usb-qualified-artifacts-v1",
    context_sha256: digest(JSON.stringify(context)),
    files: entries,
  });
  f.operations.verifyArtifactSnapshot = async (path, value) => (path === root ? verifyArtifactSnapshot(path, value) : undefined);
  const state = baselineFixture(context),
    records = await cycleFixture(root, context, state);
  records.push(
    { sequence: 9, state: { ...structuredClone(state), status: "stopping", deviceLeaseInactive: false, deviceBaselineConfirmed: false } },
    { sequence: 10, state: structuredClone(state) },
  );
  await writeFile(resolve(root, "iterative.samples.jsonl"), records.map((row) => JSON.stringify(row) + "\n").join(""), { mode: 0o600 });
  await observerFixture(root, context);
  await failureFixture(root);
  const seal = await sealFixture(f, root, context);
  return { ...f, root, context, records, seal, snapshot };
}
