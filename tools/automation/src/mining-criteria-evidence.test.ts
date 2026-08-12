import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { chmod, mkdir, mkdtemp, readFile, stat, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import {
  MiningCriteriaEvidenceError,
  projectMiningCriteriaEvidence,
  type MiningCriteriaValidators,
} from "./mining-criteria-evidence.js";
import {
  createFakeProcessPort,
  createLocalProcessPort,
  type ProcessOutcome,
  type ProcessPort,
} from "./process.js";

const currentCommit = "a".repeat(40);
const referenceCommit = "c1915b0a63bfabebdb95a515cedfee05146c1d50";
const sourcePaths = [
  "docs/parity/evidence/phase-21-live-mining-and-soak-evidence/summary.md",
  "docs/parity/evidence/phase-21-live-mining-and-soak-evidence/live-mining-smoke.md",
  "docs/parity/evidence/phase-21-live-mining-and-soak-evidence/bounded-soak.md",
  "docs/parity/evidence/str006-protocol-coordinator/protocol-coordinator-projection.json",
] as const;
const criteriaPaths = [
  "tools/flash/src/campaign.rs",
  "tools/flash/src/campaign/admission.rs",
  "tools/flash/src/campaign/markers.rs",
  "tools/flash/src/campaign/markers/soak.rs",
  "tools/flash/src/campaign/evidence.rs",
  "tools/flash/src/campaign/network/model.rs",
  "crates/bitaxe-stratum/src/v1/production_session/tests/lifecycle.rs",
  "tools/flash/src/tests/campaign.rs",
] as const;
const validators: MiningCriteriaValidators = {
  coordinator: "coordinator-validator",
  evidence: "evidence-validator",
};

function ok(stdout = ""): ProcessOutcome {
  return { exitCode: 0, stdout, stderr: "", timedOut: false };
}

function digest(document: string): string {
  return createHash("sha256").update(document).digest("hex");
}

function historicalDocuments() {
  const summary = [
    "phase21_status: passed",
    "phase21_evidence_closure: approved_controlled_no_share_soak",
    "redaction_status: passed",
    "raw_artifacts_committed: no",
    "reference_clean: passed",
  ].join("\n") + "\n";
  const smoke = [
    "live_mining_smoke_status: controlled-no-share",
    "controlled_package_boot_status: trusted",
    "controlled_runtime_harness_status: observed",
    "pool_lifecycle_status: active",
    "subscribe_status: sent",
    "authorize_status: sent",
    "notify_job_status: accepted work_enqueued=true",
    "bm1366_work_dispatch_status: typed_action_ready",
    "result_receive_status: bounded_no_result",
    "share_submission_status: bounded_no_share",
    "api_websocket_telemetry_update_status: ready",
    "watchdog_status: bounded observations present",
    "safe_stop_status: complete mining=disabled hardware_control=disabled work_submission=disabled",
    "redaction_status: passed",
  ].join("\n") + "\n";
  const soak = [
    "bounded_soak_status: approved_controlled_no_share_soak",
    "duration_seconds: 300",
    "live_smoke_prerequisite: controlled-no-share",
    "controlled_package_boot_status: trusted",
    "controlled_runtime_harness_status: observed",
    "watchdog_responsiveness_status: passed",
    "api_snapshot_status: redacted_sample_captured",
    "websocket_frame_status: passed frames=5",
    "safe_stop_status: complete mining=disabled hardware_control=disabled work_submission=disabled",
    "redaction_status: passed",
  ].join("\n") + "\n";
  const coordinator = `${JSON.stringify({
    schema_version: "bitaxe-protocol-coordinator-evidence-v1",
    board: 205,
    coordinator: {
      single_owner_serialization: true,
      authorized_before_asic_dispatch: true,
      qualified_result_before_submit: true,
      accepted_submit_observed: true,
      ordered_terminal_safe_stop: true,
      watchdog_feed_in_owner_loop: true,
      lifecycle_spans_compatible: true,
    },
    hardware_rerun_used: false,
    redaction_status: "passed",
  }, null, 2)}\n`;
  return [summary, smoke, soak, coordinator] as const;
}

async function fixture(name: string) {
  const root = await mkdtemp(path.join(os.tmpdir(), `bitaxe-mining-criteria-${name}-`));
  await writeFile(path.join(root, "MODULE.bazel"), "module(name = \"fixture\")\n");
  const documents = historicalDocuments();
  for (const [index, sourcePath] of sourcePaths.entries()) {
    const destination = path.join(root, sourcePath);
    await mkdir(path.dirname(destination), { recursive: true });
    await writeFile(destination, documents[index] ?? "");
  }
  const projection = path.join(root,
    "docs/parity/evidence/str007-mining-criteria/mining-criteria-projection.json");
  return {
    root,
    projection,
    documents,
    admittedDigests: {
      summary: digest(documents[0]),
      smoke: digest(documents[1]),
      soak: digest(documents[2]),
      coordinator: digest(documents[3]),
    },
    options: {
      summary: path.join(root, sourcePaths[0]),
      smoke: path.join(root, sourcePaths[1]),
      soak: path.join(root, sourcePaths[2]),
      coordinatorProjection: path.join(root, sourcePaths[3]),
      projection,
    },
  };
}

function criteriaSource(sourcePath: string): string {
  if (sourcePath === criteriaPaths[0]) {
    return [
      "const MINING_DURATION_SECONDS: u64 = 600;",
      "let cleanup_result = environment.finish_usb_session();",
    ].join("\n");
  }
  if (sourcePath === criteriaPaths[1]) {
    return [
      "command.board != BoardId::Ultra205 || !command.redact_evidence",
      "MiningCampaignStage::LiveShare | MiningCampaignStage::Soak => MINING_DURATION_SECONDS,",
      "command.profile == Some(MiningCampaignProfile::UpstreamDefault)",
    ].join("\n");
  }
  if (sourcePath === criteriaPaths[2]) {
    return [
      "marker.safe_stop == SafeStopMarker::Confirmed;",
      "assess_soak_terminal(marker, admission.duration_seconds)",
    ].join("\n");
  }
  if (sourcePath === criteriaPaths[3]) {
    return [
      "marker.accepted_share_count == 0",
      "marker.active_ms < duration_seconds.saturating_mul(1_000)",
    ].join("\n");
  }
  if (sourcePath === criteriaPaths[4]) {
    return [
      "set_private_directory_mode(root)?;",
      "write_private_new_bytes(&paths.result, &result_bytes)",
      "redacted: true,",
    ].join("\n");
  }
  if (sourcePath === criteriaPaths[5]) {
    return [
      "self.close_elapsed_windows(600_000, serial);",
      "covered_window_count == REQUIRED_WINDOWS",
      "watchdog_valid: self.watchdog_valid,",
      "terminal_http_valid: self.terminal_http_valid,",
      "terminal_websocket_valid: self.terminal_websocket_valid,",
    ].join("\n");
  }
  if (sourcePath === criteriaPaths[6]) return "fn active_duration_counts_from_authorized_mining()";
  return "fn soak_requires_full_active_duration()";
}

function fakePort(options: {
  readonly dirty?: boolean;
  readonly semanticDrift?: boolean;
  readonly duplicate?: boolean;
  readonly validatorFailure?: boolean;
  readonly launchFailure?: boolean;
} = {}): ProcessPort {
  return createFakeProcessPort(async (spec) => {
    if (options.launchFailure && spec.program === validators.evidence) throw new Error("launch");
    if (spec.program === validators.coordinator || spec.program === validators.evidence) {
      return options.validatorFailure ? { ...ok(), exitCode: 1 } : ok();
    }
    if (spec.args[0] === "rev-parse" && spec.args[1] === "HEAD") return ok(currentCommit);
    if (spec.args[0] === "-C") return ok(referenceCommit);
    if (spec.args[0] === "status") return ok(options.dirty ? " M tools/flash/src/campaign.rs" : "");
    if (spec.args[0] === "show") {
      const sourcePath = String(spec.args[1]).slice(currentCommit.length + 1);
      let document = criteriaSource(sourcePath);
      if (options.semanticDrift && sourcePath === criteriaPaths[0]) {
        document = document.replace("600", "601");
      }
      if (options.duplicate && sourcePath === criteriaPaths[0]) document = `${document}\n${document}`;
      return ok(document);
    }
    return ok();
  });
}

async function captureError(promise: Promise<unknown>): Promise<MiningCriteriaEvidenceError> {
  try {
    await promise;
    assert.fail("expected projection failure");
  } catch (error) {
    assert.ok(error instanceof MiningCriteriaEvidenceError);
    return error;
  }
}

test("closed historical and current criteria publish redacted evidence", async () => {
  // Arrange
  const value = await fixture("ready");

  // Act
  const evidence = await projectMiningCriteriaEvidence(
    value.root, value.options, fakePort(), "git", validators, value.admittedDigests,
  );

  // Assert
  assert.equal(evidence.criteria.historical_soak_duration_seconds, 300);
  assert.equal(evidence.criteria.current_duration_seconds, 600);
  assert.equal(evidence.criteria.terminal_attempt_reopened, false);
  assert.equal((await stat(value.projection)).mode & 0o777, 0o644);
  assert.doesNotMatch(await readFile(value.projection, "utf8"),
    /password|credential|device_url|endpoint|pool_url|wifi|serial_port|usb_identity/iu);
});

test("digest drift and incomplete historical facts withhold evidence", async () => {
  for (const name of ["digest", "facts"] as const) {
    // Arrange
    const value = await fixture(name);
    if (name === "digest") {
      await writeFile(value.options.soak, "changed\n");
    } else {
      const incomplete = value.documents[2].replace("watchdog_responsiveness_status: passed\n", "");
      await writeFile(value.options.soak, incomplete);
      value.admittedDigests.soak = digest(incomplete);
    }

    // Act
    const error = await captureError(projectMiningCriteriaEvidence(
      value.root, value.options, fakePort(), "git", validators, value.admittedDigests,
    ));

    // Assert
    assert.equal(error.category, "evidence_invalid");
    await assert.rejects(readFile(value.projection, "utf8"), { code: "ENOENT" });
  }
});

test("semantic, duplicate, and dirty source drift withhold evidence", async () => {
  for (const [name, port] of [
    ["semantic", fakePort({ semanticDrift: true })],
    ["duplicate", fakePort({ duplicate: true })],
    ["dirty", fakePort({ dirty: true })],
  ] as const) {
    // Arrange
    const value = await fixture(name);

    // Act
    const error = await captureError(projectMiningCriteriaEvidence(
      value.root, value.options, port, "git", validators, value.admittedDigests,
    ));

    // Assert
    assert.equal(error.category, "evidence_invalid");
    await assert.rejects(readFile(value.projection, "utf8"), { code: "ENOENT" });
  }
});

test("malformed coordinator and validator failures preserve typed categories", async () => {
  for (const [name, category, port] of [
    ["coordinator", "evidence_invalid", fakePort()],
    ["validator", "evidence_invalid", fakePort({ validatorFailure: true })],
    ["launch", "process_failed", fakePort({ launchFailure: true })],
  ] as const) {
    // Arrange
    const value = await fixture(name);
    if (name === "coordinator") {
      await writeFile(value.options.coordinatorProjection, "not-json\n");
      value.admittedDigests.coordinator = digest("not-json\n");
    }

    // Act
    const error = await captureError(projectMiningCriteriaEvidence(
      value.root, value.options, port, "git", validators, value.admittedDigests,
    ));

    // Assert
    assert.equal(error.category, category);
    await assert.rejects(readFile(value.projection, "utf8"), { code: "ENOENT" });
  }
});

test("real child validators must accept the coordinator and candidate files", async () => {
  // Arrange
  const value = await fixture("real-child");
  const validator = path.join(value.root, "validator-child.sh");
  await writeFile(validator, "#!/bin/sh\ntest -s \"$1\"\n");
  await chmod(validator, 0o700);
  const localPort = createLocalProcessPort({ cwd: value.root, timeoutMs: 5_000 });
  const gitPort = fakePort();
  const processPort: ProcessPort = {
    loadEspEnvironment: () => localPort.loadEspEnvironment(),
    run: (spec, maybeTimeoutMs) => spec.program === "git-fixture"
      ? gitPort.run(spec, maybeTimeoutMs)
      : localPort.run(spec, maybeTimeoutMs),
  };

  // Act
  const evidence = await projectMiningCriteriaEvidence(
    value.root, value.options, processPort, "git-fixture",
    { coordinator: validator, evidence: validator }, value.admittedDigests,
  );

  // Assert
  assert.equal(evidence.schema_version, "bitaxe-mining-criteria-evidence-v1");
  assert.equal((await stat(value.projection)).mode & 0o777, 0o644);
});
