import assert from "node:assert/strict";
import { chmod, mkdtemp, mkdir, readFile, stat, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import {
  AsicInitializationEvidenceError,
  projectAsicInitializationEvidence,
} from "./asic-initialization-evidence.js";
import { createFakeProcessPort, createLocalProcessPort, type ProcessOutcome } from "./process.js";

const attemptCommit = "a".repeat(40);
const currentCommit = "b".repeat(40);
const referenceCommit = "c".repeat(40);
const ok = (stdout = ""): ProcessOutcome => ({ exitCode: 0, stdout, stderr: "", timedOut: false });

function result(diagnosticsSha256: string, observationsSha256: string) {
  return {
    schema: "mining-campaign-result-v2",
    evidence_class: "protected-operational",
    stage: "live-share",
    profile: "conservative",
    duration_seconds: 600,
    status: "accepted",
    terminal_category: "submit_response_observed",
    package_admitted: true,
    runtime_identity: "trusted",
    runtime_attestation_status: "trusted",
    serial_outcome_detail: "clean",
    pool_config: "local-owner-supplied",
    marker_count: 10,
    submit_outcome: "accepted",
    qualified_candidate_count: 1,
    below_pool_target_count: 3,
    duplicate_candidate_count: 0,
    terminal_reason: "campaign_lease_consumed",
    active_ms: 2_000,
    safety: "fresh",
    fresh_observation_count: 5,
    mineonboot: false,
    campaign_failure: { phase: "none", step: "none", detail: "none", rollback_step: "none", rollback_detail: "none" },
    safe_stop: "confirmed",
    usb_cleanup: "ready",
    observations_sha256: observationsSha256,
    diagnostics_sha256: diagnosticsSha256,
    redacted: true,
    parity_promotion: false,
  };
}

function diagnostics(accepted = 18) {
  return {
    schema: "mining-campaign-serial-diagnostics-v1",
    observation_started: true,
    preparation_candidate_count: 18,
    accepted_preparation_event_count: accepted,
    preparation_invalid_encoding_count: 0,
    preparation_invalid_json_count: 0,
    preparation_invalid_schema_count: 0,
    latest_preparation_event: {
      schema: "mining-campaign-preparation-v1",
      step: "retain_production_uart",
      outcome: "completed",
    },
  };
}

async function fixture(name: string, accepted = 18) {
  const root = await mkdtemp(path.join(os.tmpdir(), `bitaxe-asic-init-${name}-`));
  await writeFile(path.join(root, "MODULE.bazel"), "module(name = \"fixture\")\n");
  const attemptRoot = path.join(root, "scratch", "attempt-007");
  await mkdir(attemptRoot, { recursive: true, mode: 0o700 });
  await chmod(attemptRoot, 0o700);
  const diagnosticsDocument = `${JSON.stringify(diagnostics(accepted), null, 2)}\n`;
  const observationsDocument = `${JSON.stringify({ schema: "private-observations", markers: [] })}\n`;
  const { createHash } = await import("node:crypto");
  const sha256 = (value: string) => createHash("sha256").update(value).digest("hex");
  const resultDocument = `${JSON.stringify(result(sha256(diagnosticsDocument), sha256(observationsDocument)), null, 2)}\n`;
  for (const [file, document] of [
    ["campaign-diagnostics.private.json", diagnosticsDocument],
    ["campaign-observations.private.json", observationsDocument],
    ["campaign-result.json", resultDocument],
    ["campaign-result.sha256", `${sha256(resultDocument)}\n`],
  ] as const) {
    await writeFile(path.join(attemptRoot, file), document, { mode: 0o600 });
    await chmod(path.join(attemptRoot, file), 0o600);
  }
  await writeFile(path.join(root, "TASKS.archive.md"), `### task-ultra205-accepted-pool-share | accepted\nClean commit \`${attemptCommit.slice(0, 8)}\` \`attempt-007\` admitted the exact package and one Ultra\n205, completed every preparation boundary, confirmed safe stop, USB cleanup ready, and parity promotion is false.\n`);
  return {
    root,
    attemptRoot,
    projection: path.join(root, "docs", "asic-init.json"),
    options: {
      attemptRoot,
      attemptSourceCommit: attemptCommit,
      projection: path.join(root, "docs", "asic-init.json"),
    },
  };
}

function fakePort(diffExitCode = 0) {
  return createFakeProcessPort(async (spec) => {
    if (spec.args[0] === "rev-parse") return ok(`${currentCommit}\n`);
    if (spec.args[0] === "-C") return ok(`${referenceCommit}\n`);
    if (spec.args[0] === "diff") return { ...ok(), exitCode: diffExitCode };
    return ok();
  });
}

async function captureError(promise: Promise<unknown>): Promise<AsicInitializationEvidenceError> {
  try {
    await promise;
    assert.fail("expected projection failure");
  } catch (error) {
    assert.ok(error instanceof AsicInitializationEvidenceError);
    return error;
  }
}

test("sealed complete preparation emits only closed initialization evidence", async () => {
  // Arrange
  const value = await fixture("ready");

  // Act
  const evidence = await projectAsicInitializationEvidence(
    value.root,
    value.options,
    fakePort(),
    "git",
    "validator",
  );

  // Assert
  assert.equal(evidence.initialization.accepted_preparation_event_count, 18);
  assert.equal(evidence.initialization.production_uart_retained, true);
  assert.equal(evidence.hardware_rerun_used, false);
  assert.equal((await stat(value.projection)).mode & 0o777, 0o644);
  assert.doesNotMatch(
    await readFile(value.projection, "utf8"),
    /pool_config|pool-owner|wifi|device_url|endpoint|nonce|difficulty|credential|private-observations/iu,
  );
});

test("incomplete preparation withholds public evidence", async () => {
  // Arrange
  const value = await fixture("incomplete", 17);

  // Act
  const error = await captureError(projectAsicInitializationEvidence(
    value.root,
    value.options,
    fakePort(),
    "git",
    "validator",
  ));

  // Assert
  assert.equal(error.category, "evidence_invalid");
  await assert.rejects(readFile(value.projection, "utf8"), { code: "ENOENT" });
});

test("source drift withholds public evidence", async () => {
  // Arrange
  const value = await fixture("drift");

  // Act
  const error = await captureError(projectAsicInitializationEvidence(
    value.root,
    value.options,
    fakePort(1),
    "git",
    "validator",
  ));

  // Assert
  assert.equal(error.category, "evidence_invalid");
  await assert.rejects(readFile(value.projection, "utf8"), { code: "ENOENT" });
});

test("a real child validator must accept the candidate before publication", async () => {
  // Arrange
  const value = await fixture("real-child");
  const git = path.join(value.root, "git-child.sh");
  const validator = path.join(value.root, "validator-child.sh");
  await writeFile(git, `#!/bin/sh
if [ "$1" = "rev-parse" ]; then printf '%s\\n' '${currentCommit}'; exit 0; fi
if [ "$1" = "-C" ]; then printf '%s\\n' '${referenceCommit}'; exit 0; fi
exit 0
`);
  await writeFile(validator, "#!/bin/sh\ntest -s \"$1\"\n");
  await chmod(git, 0o700);
  await chmod(validator, 0o700);

  // Act
  const evidence = await projectAsicInitializationEvidence(
    value.root,
    value.options,
    createLocalProcessPort({ cwd: value.root, timeoutMs: 5_000 }),
    git,
    validator,
  );

  // Assert
  assert.equal(evidence.schema_version, "bitaxe-asic-initialization-evidence-v1");
  assert.equal((await stat(value.projection)).mode & 0o777, 0o644);
});
