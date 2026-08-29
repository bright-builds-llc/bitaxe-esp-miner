import assert from "node:assert/strict";
import { mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import test from "node:test";

import {
  parseNoiseDiagnosticArgs,
  runNoiseDiagnosticProcess,
} from "./stratum-v2-noise-diagnostic.js";
import { noiseDiagnosticValidatorArgs } from "./stratum-v2-noise-diagnostic-process.js";
import { validateNoiseDiagnosticProjection } from "./stratum-v2-noise-diagnostic-validator.js";
import {
  noiseSendFromMonitor,
  noiseSocketErrorsFromMonitor,
  noiseStagesFromMonitor,
  noiseTimingsFromMonitor,
} from "./stratum-v2-noise-diagnostic-markers.js";
import { projectNoiseAuthConnection } from "./stratum-v2-noise-connection.js";

const source = "a".repeat(40);

function exactArgs(): string[] {
  return [
    "--board", "205",
    "--port", "/dev/cu.test",
    "--package-manifest", "bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json",
    "--wifi-credentials", "wifi-credentials.json",
    "--restore-bundle", "scratch/str005-installed-package-recovery/recovery-006/restore-bundle.private.json",
    "--private-root", "scratch/str005-noise-auth/diagnostic-001",
    "--projection", "docs/parity/evidence/str005-noise-auth/noise-auth-projection-001.json",
    "--plan", "docs/parity/work-plans/20260829T143226Z-STR-005-NOISE-AUTH/PLAN.md",
    "--diagnostic-ordinal", "1",
    "--redact-evidence",
  ];
}

function acceptedProjection(): Record<string, unknown> {
  return {
    schema_version: "bitaxe-stratum-v2-noise-auth-projection-v1",
    status: "accepted",
    board: 205,
    diagnostic_ordinal: 1,
    source_commit: source,
    reference_commit: "b".repeat(40),
    app_elf_sha256: "c".repeat(64),
    plan_sha256: "d".repeat(64),
    package_manifest_sha256: "e".repeat(64),
    evaluator_sha256: "f".repeat(64),
    terminal_category: "accepted",
    stages: {
      monitor_armed: true,
      noise_prepared: true,
      tcp_connected: true,
      act_one_created: true,
      act_one_sent: true,
      act_two_received: true,
      time_sampled: true,
      authenticated: true,
      encrypted_proof_sent: true,
    },
    timings: {
      keypair_preparation_ms: 120,
      act_one_construction_ms: 30,
      connect_ms: 5,
      act_one_write_ms: 1,
      act_two_read_ms: 4,
      proof_write_ms: 1,
    },
    connection: {
      tuple_match: true,
      local_marker_consistent: true,
      exact_peer_connection_count: 1,
      other_exact_peer_connection_count: 0,
      candidate_overflow: false,
      correlated_candidate_found: true,
    },
    send: {
      adapter: "std",
      authority_required: true,
      act_one_reported_bytes: 64,
      proof_reported_bytes: 22,
      pre_act_one_error: "none",
      post_act_one_error: "none",
      post_act_two_error: "none",
      post_proof_error: "none",
    },
    fixture: {
      listener_ready: true,
      connection_accepted: true,
      peer_matched: true,
      unexpected_peer_count: 0,
      act_one_bytes_received: 64,
      act_one_read_category: "complete",
      accept_to_first_byte_millis: 2,
      act_one_read_millis: 3,
      act_one_received: true,
      responder_created: true,
      act_two_created: true,
      act_two_sent: true,
      client_authenticated: true,
      noise_authenticated: true,
      encrypted_proof_exact: true,
    },
    campaign_started: false,
    mining_started: false,
    asic_touched: false,
    fan_touched: false,
    voltage_touched: false,
    restoration: {
      identity_exact: true,
      settings_exact: true,
      mineonboot_disabled: true,
      mining_inactive: true,
      zero_work: true,
      usb_cleanup_complete: true,
      owned_processes_remaining: 0,
    },
    redaction_complete: true,
    redaction_status: "passed",
  };
}

test("Noise-auth parser admits only the first exact no-mining contract", () => {
  // Arrange / Act
  const parsed = parseNoiseDiagnosticArgs("start", exactArgs());

  // Assert
  assert.equal(parsed.diagnosticOrdinal, 1);
  assert.equal(parsed.privateRoot, "scratch/str005-noise-auth/diagnostic-001");
  assert.equal(parsed.redactEvidence, true);
  assert.throws(() => parseNoiseDiagnosticArgs("start", exactArgs().map(value =>
    value === "1" ? "2" : value)));
});

test("Noise-auth parser keeps recovery and finalization roots action-specific", () => {
  // Arrange
  const recovery = exactArgs().map(value =>
    value === "scratch/str005-noise-auth/diagnostic-001"
      ? "scratch/str005-noise-auth/recovery-001"
      : value);

  // Act
  const parsedRecovery = parseNoiseDiagnosticArgs("recover", recovery);
  const parsedFinalize = parseNoiseDiagnosticArgs("finalize", exactArgs());

  // Assert
  assert.equal(parsedRecovery.privateRoot, "scratch/str005-noise-auth/recovery-001");
  assert.equal(parsedFinalize.privateRoot, "scratch/str005-noise-auth/diagnostic-001");
});

test("projection validator requires the complete authenticated and restored chain", async () => {
  // Arrange
  const root = await mkdtemp(path.join(tmpdir(), "str005-noise-validator-"));
  const candidate = path.join(root, "projection.json");
  const projection = acceptedProjection();
  await writeFile(candidate, JSON.stringify(projection));

  try {
    // Act / Assert
    await validateNoiseDiagnosticProjection(candidate, source, 1);
    (projection["stages"] as Record<string, unknown>)["authenticated"] = false;
    await writeFile(candidate, JSON.stringify(projection));
    await assert.rejects(validateNoiseDiagnosticProjection(candidate, source, 1));
    (projection["stages"] as Record<string, unknown>)["authenticated"] = true;
    (projection["fixture"] as Record<string, unknown>)["remote_port"] = 49_152;
    await writeFile(candidate, JSON.stringify(projection));
    await assert.rejects(validateNoiseDiagnosticProjection(candidate, source, 1));
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});

test("diagnostic process owner terminates a timed-out real child", async () => {
  // Arrange
  const child = ["-e", "setInterval(() => {}, 1000)"];

  // Act / Assert
  await assert.rejects(
    runNoiseDiagnosticProcess(process.cwd(), process.execPath, child, 50, "real_child"),
    (error: unknown) => error instanceof Error && error.message === "timeout:real_child",
  );
});

test("diagnostic marker parser retains only bounded stages and timings", () => {
  // Arrange
  const monitor = [
    'stratum_v2_noise_diagnostic={"stage":"noise_prepared"}',
    'stratum_v2_noise_diagnostic={"stage":"monitor_armed"}',
    'stratum_v2_noise_diagnostic={"stage":"tcp_connected"}',
    'stratum_v2_noise_timing={"phase":"keypair_preparation","duration_ms":120}',
    'stratum_v2_noise_timing={"phase":"act_one_construction","duration_ms":30}',
    'stratum_v2_noise_timing={"phase":"connect","duration_ms":5}',
    'stratum_v2_noise_timing={"phase":"act_one_write","duration_ms":1}',
    'stratum_v2_noise_timing={"phase":"act_two_read","duration_ms":4}',
    'stratum_v2_noise_timing={"phase":"proof_write","duration_ms":1}',
    'stratum_v2_noise_diagnostic={"stage":"encrypted_proof_sent"}',
  ].join("\n");

  // Act
  const stages = noiseStagesFromMonitor(monitor);
  const timings = noiseTimingsFromMonitor(monitor);

  // Assert
  assert.equal(stages["noise_prepared"], true);
  assert.equal(stages["monitor_armed"], true);
  assert.equal(stages["encrypted_proof_sent"], true);
  assert.equal(stages["tcp_connected"], true);
  assert.equal(timings["keypair_preparation_ms"], 120);
  assert.equal(timings["act_one_construction_ms"], 30);
  assert.equal(timings["act_two_read_ms"], 4);
  assert.equal(timings["proof_write_ms"], 1);
});

test("Noise-auth connection join publishes only the correlated closed candidate", () => {
  // Arrange
  const monitor = [
    'stratum_v2_noise_connection_private={"local_port":49152}',
    'stratum_v2_noise_connection_private={"local_port":49152}',
  ].join("\n");
  const fixture = {
    listener_ready: true,
    connection_accepted: true,
    peer_matched: true,
    unexpected_peer_count: 0,
    exact_peer_connection_count: 1,
    candidate_overflow: false,
    noise_candidates: [{
      remote_port: 49152,
      act_one_bytes_received: 64,
      act_one_read_category: "complete",
    }],
    responder_created: true,
    act_two_created: true,
    act_two_sent: true,
    client_authenticated: true,
    noise_authenticated: true,
    encrypted_proof_exact: true,
  };

  // Act
  const projected = projectNoiseAuthConnection(monitor, fixture);

  // Assert
  assert.equal(projected.connection["tuple_match"], true);
  assert.equal(projected.fixture["act_one_bytes_received"], 64);
  assert.equal(JSON.stringify(projected).includes("49152"), false);
  assert.equal(JSON.stringify(projected).includes("remote_port"), false);
});

test("Noise-auth marker parser retains exact send counts and closed socket families", () => {
  // Arrange
  const monitor = [
    'stratum_v2_noise_send={"kind":"act_one","bytes_written":64}',
    'stratum_v2_noise_send={"kind":"proof","bytes_written":22}',
    'stratum_v2_noise_socket_error={"phase":"pre_act_one","category":"none"}',
    'stratum_v2_noise_socket_error={"phase":"post_act_one","category":"none"}',
    'stratum_v2_noise_socket_error={"phase":"post_act_two","category":"none"}',
    'stratum_v2_noise_socket_error={"phase":"post_proof","category":"none"}',
  ].join("\n");

  // Act
  const send = noiseSendFromMonitor(monitor);
  const sockets = noiseSocketErrorsFromMonitor(monitor);

  // Assert
  assert.equal(send["act_one_reported_bytes"], 64);
  assert.equal(send["proof_reported_bytes"], 22);
  assert.equal(sockets["pre_act_one_error"], "none");
  assert.equal(sockets["post_proof_error"], "none");
});

test("diagnostic projection routes independent validation through Bazel", () => {
  // Arrange / Act
  const args = noiseDiagnosticValidatorArgs("/private/candidate.json", source, 4);

  // Assert
  assert.deepEqual(args, [
    "run",
    "//tools/automation:stratum_v2_noise_auth_validator",
    "--",
    "/private/candidate.json",
    source,
    "4",
  ]);
});
