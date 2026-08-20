import { createHash } from "node:crypto";
import { chmod, mkdir, writeFile } from "node:fs/promises";
import path from "node:path";

import {
  scoreboardFixture,
  sourceCommit,
  referenceCommit,
  type ScoreboardFixture,
} from "./scoreboard-evidence.test-support.js";
import type {
  ScoreboardRecheckIdentity,
  ScoreboardRecheckOptions,
} from "./scoreboard-recheck.js";

const appElfSha256 = "e".repeat(64);

export type ScoreboardRecheckFixture = Readonly<{
  root: string;
  base: ScoreboardFixture;
  options: ScoreboardRecheckOptions;
  identity: ScoreboardRecheckIdentity;
}>;

function sha256(value: string): string {
  return createHash("sha256").update(value).digest("hex");
}

async function writePrivate(candidate: string, document: string): Promise<void> {
  await writeFile(candidate, document, { mode: 0o600 });
  await chmod(candidate, 0o600);
}

export async function scoreboardRecheckFixture(name: string): Promise<ScoreboardRecheckFixture> {
  const base = await scoreboardFixture(`recheck-${name}`);
  const capturePlan = "- Parity row: `STAT-003`\n- Active task: `task-parity-stat003-scoreboard`\n";
  const captureClosure = [
    "- Parity row: `STAT-003`",
    "- Active task: `task-parity-stat003-scoreboard`",
    "scoreboard restart persistence is invalid",
    "",
  ].join("\n");
  const evaluationPlan = "- Parity row: `STAT-003`\n- Active task: `task-parity-stat003-scoreboard`\n";
  const capturePlanPath = "docs/parity/work-plans/20260820T150151Z-STAT-003/PLAN.md";
  const captureClosurePath = "docs/parity/work-plans/20260820T150151Z-STAT-003/CLOSURE.md";
  const evaluationPlanPath = "docs/parity/work-plans/20260820T224453Z-STAT-003/PLAN.md";
  await mkdir(path.join(base.root, path.dirname(captureClosurePath)), { recursive: true });
  await mkdir(path.join(base.root, path.dirname(evaluationPlanPath)), { recursive: true });
  await writeFile(path.join(base.root, capturePlanPath), capturePlan);
  await writeFile(path.join(base.root, captureClosurePath), captureClosure);
  await writeFile(path.join(base.root, evaluationPlanPath), evaluationPlan);
  await writeFile(path.join(base.root, "TASKS.md"), [
    "### task-parity-stat003-scoreboard | fixture",
    `Plan: \`${evaluationPlanPath}\`.`,
  ].join("\n"));

  const wrapperRoot = path.join(base.root, "scratch/stat003-scoreboard/wrapper-005");
  await writePrivate(path.join(wrapperRoot, "detector.stdout"), "port: /dev/cu.usbmodem-fixture\n");
  await writePrivate(path.join(wrapperRoot, "detector.stderr"), "");
  await writePrivate(path.join(wrapperRoot, "capture.stdout"), [
    "bazel run //tools/automation:capture_scoreboard_evidence -- --private-root scratch/stat003-scoreboard/attempt-005",
    JSON.stringify({
      schema_version: "bitaxe-automation-result-v1",
      command: "capture-scoreboard-evidence",
      status: "blocked",
      category: "hardware_blocked",
      public: { stage: "scoreboard_capture", projection_published: false },
    }),
    "",
  ].join("\n"));
  await writePrivate(
    path.join(wrapperRoot, "capture.stderr"),
    "bitaxe-automation: capture-scoreboard-evidence blocked: scoreboard restart persistence is invalid\n",
  );
  await writePrivate(path.join(wrapperRoot, "recheck.stdout"), "");
  await writePrivate(path.join(wrapperRoot, "recheck.stderr"), "");
  await writePrivate(path.join(wrapperRoot, "recheck-v2.stdout"), "");
  await writePrivate(path.join(wrapperRoot, "recheck-v2.stderr"), "");

  const privateRoot = path.join(base.root, "scratch/stat003-scoreboard/attempt-005");
  const campaignRoot = path.join(privateRoot, "campaign");
  await mkdir(campaignRoot, { recursive: true, mode: 0o700 });
  await chmod(privateRoot, 0o700);
  await chmod(campaignRoot, 0o700);
  const diagnostics = `${JSON.stringify({
    schema: "mining-campaign-serial-diagnostics-v4",
    runtime_attestation_mixed_reset_reason: "none",
    panic_signature: "none",
    panic_signature_count: 0,
  })}\n`;
  const network = `${JSON.stringify({
    schema: "mining-campaign-network-continuity-v12",
    status: "accepted",
    correlation_failure: "none",
    required_window_count: 20,
    covered_window_count: 20,
    work_renewal_valid: true,
    terminal_http_valid: true,
    terminal_websocket_valid: true,
    terminal_pool_persisted: true,
    terminal_settlement: "accepted_after_serial_close",
    terminal_close_requested: true,
    terminal_consumed_observed: true,
    final_terminal_consumed: true,
    serial_finished_observed: true,
  })}\n`;
  const flash = `${JSON.stringify({
    schema: "mining-campaign-flash-diagnostics-v1",
    factory: {},
    nvs: {},
    raw_output_included: false,
  })}\n`;
  const result = `${JSON.stringify({
    schema: "mining-campaign-result-v16",
    status: "accepted",
    stage: "live-share",
    profile: "conservative",
    duration_seconds: 600,
    runtime_identity: "trusted",
    pool_config: "local_owner_supplied",
    safe_stop: "confirmed",
    usb_cleanup: "ready",
    redacted: true,
    qualified_candidate_count: 1,
    below_pool_target_count: 0,
    duplicate_candidate_count: 0,
    submit_outcome: "accepted",
    diagnostics_sha256: sha256(diagnostics),
    network_continuity_sha256: sha256(network),
    flash_diagnostics_sha256: sha256(flash),
  })}\n`;
  for (const [file, document] of [
    ["campaign-diagnostics.private.json", diagnostics],
    ["campaign-flash.private.json", flash],
    ["campaign-mining-diagnostics.private.json", "{}\n"],
    ["campaign-network.private.json", network],
    ["campaign-observations.private.json", "{}\n"],
    ["campaign-result.json", result],
    ["campaign-result.sha256", `${sha256(result)}\n`],
  ] as const) {
    await writePrivate(path.join(campaignRoot, file), document);
  }

  const system = (bootSession: string, bootOrdinal: number, resetReasonCategory: string) => ({
    sourceCommit,
    referenceCommit,
    appElfSha256,
    bootSession,
    bootOrdinal,
    resetReasonCategory,
    buildTimestampUtc: "2026-08-20T16:00:00Z",
    miningActivity: "safe_blocked",
    startMiningOnBoot: false,
  });
  const beforeEntries = [
    { difficulty: 42.54, job_id: "job-a", extranonce2: "0001", ntime: 1, nonce: "1234ABCD", version_bits: "20000000" },
    { difficulty: 10.06, job_id: "job-b", extranonce2: "0002", ntime: 2, nonce: "00000001", version_bits: "00000000" },
  ];
  const afterEntries = [
    { ...beforeEntries[0], difficulty: 42.5 },
    { ...beforeEntries[1], difficulty: 10.1 },
  ];
  for (const [file, document] of [
    ["post-campaign-monitor.private.log", "runtime origin retained privately\n"],
    ["post-campaign-system.private.json", `${JSON.stringify(system("00112233445566778899aabbccddeeff", 7, "power_on"))}\n`],
    ["restart-response.private.txt", "{}"],
    ["scoreboard-after-restart-a.private.json", `${JSON.stringify(afterEntries)}\n`],
    ["scoreboard-after-restart-b.private.json", `${JSON.stringify(afterEntries)}\n`],
    ["scoreboard-before-restart-a.private.json", `${JSON.stringify(beforeEntries)}\n`],
    ["scoreboard-before-restart-b.private.json", `${JSON.stringify(beforeEntries)}\n`],
    ["scoreboard-route.private.html", '<section data-page="scoreboard"></section><script src="/assets/api-client.js"></script>'],
    ["post-restart-system-1.private.json", `${JSON.stringify(system("ffeeddccbbaa99887766554433221100", 8, "software_cpu"))}\n`],
  ] as const) {
    await writePrivate(path.join(privateRoot, file), document);
  }

  return {
    root: base.root,
    base,
    options: {
      privateRoot: "scratch/stat003-scoreboard/attempt-005",
      wrapperRoot: "scratch/stat003-scoreboard/wrapper-005",
      capturePlan: capturePlanPath,
      captureClosure: captureClosurePath,
      evaluationPlan: evaluationPlanPath,
      projection: "docs/parity/evidence/stat003-scoreboard/scoreboard-projection.json",
    },
    identity: {
      capturePlanSha256: sha256(capturePlan),
      captureClosureSha256: sha256(captureClosure),
      evaluationPlanSha256: sha256(evaluationPlan),
      captureSourceCommit: sourceCommit,
      referenceCommit,
    },
  };
}
