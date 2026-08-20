import { createHash } from "node:crypto";
import {
  chmod,
  lstat,
  mkdir,
  readFile,
  readdir,
  rename,
  unlink,
  writeFile,
} from "node:fs/promises";
import path from "node:path";

import type { ScoreboardEvidence } from "./contracts.generated.js";
import { internalCommandSpec } from "./contracts.generated.js";
import {
  bootMiningDisabled,
  expectedProjection,
  failure,
  object,
  requiredBoolean,
  requiredInteger,
  requiredString,
  scoreboardRestartPersists,
  scoreboardView,
  sha256,
  type JsonObject,
  type ScoreboardView,
} from "./scoreboard-evidence-contract.js";
import {
  campaignQuorum,
  requirePrivateTreeModes,
  ScoreboardEvidenceError,
} from "./scoreboard-evidence.js";
import { scoreboardSourceInventory } from "./scoreboard-source-inventory.js";
import type { ProcessPort } from "./process.js";
import { assertWithinWorkspace } from "./workspace.js";

const expectedPrivateRoot = "scratch/stat003-scoreboard/attempt-005";
const expectedWrapperRoot = "scratch/stat003-scoreboard/wrapper-005";
const expectedCapturePlan = "docs/parity/work-plans/20260820T150151Z-STAT-003/PLAN.md";
const expectedCaptureClosure = "docs/parity/work-plans/20260820T150151Z-STAT-003/CLOSURE.md";
const expectedEvaluationPlan = "docs/parity/work-plans/20260820T220854Z-STAT-003/PLAN.md";
const expectedCapturePlanSha256 = "43d13ec599e9f46988f0ebb44607dc000eff95db78c37fdc340fe52e14365684";
const expectedCaptureClosureSha256 = "65ea96446527414d7a91af203531f7284f59daaca157b68ddba9598e9de335f5";
const expectedEvaluationPlanSha256 = "ec0df4b780dd6c9dd1fc6453dfd318feb16d917badd9c0e177db199e9fe1b8ee";
const expectedCaptureSourceCommit = "a31af2873e6b2d41fe47aa18a57626f33aaf099b";
const expectedReferenceCommit = "c1915b0a63bfabebdb95a515cedfee05146c1d50";
const expectedAppElfSha256 = "6bf80011336101f4820cf84a6b338724b91a86e2547ce91f1a32c3dcfe14549c";
const expectedPackageManifestSha256 = "e8a14fc2f269fd4472ef160d2fcb994ca300879bbcbfde041363ae6c2784b4bd";
const activeTask = "task-parity-stat003-scoreboard";
const portPattern = /^(?:\/dev\/(?:cu\.usbmodem|cu\.usbserial|ttyUSB|ttyACM)[A-Za-z0-9._-]*|COM[0-9]+)$/u;

const campaignFiles = [
  "campaign-diagnostics.private.json",
  "campaign-flash.private.json",
  "campaign-mining-diagnostics.private.json",
  "campaign-network.private.json",
  "campaign-observations.private.json",
  "campaign-result.json",
  "campaign-result.sha256",
] as const;
const fixedAttemptFiles = [
  "post-campaign-monitor.private.log",
  "post-campaign-system.private.json",
  "restart-response.private.txt",
  "scoreboard-after-restart-a.private.json",
  "scoreboard-after-restart-b.private.json",
  "scoreboard-before-restart-a.private.json",
  "scoreboard-before-restart-b.private.json",
  "scoreboard-route.private.html",
] as const;
const wrapperFiles = [
  "capture.stderr",
  "capture.stdout",
  "detector.stderr",
  "detector.stdout",
  "recheck.stderr",
  "recheck.stdout",
] as const;

export type ScoreboardRecheckOptions = Readonly<{
  privateRoot: string;
  wrapperRoot: string;
  capturePlan: string;
  captureClosure: string;
  evaluationPlan: string;
  projection: string;
}>;

export type ScoreboardRecheckIdentity = Readonly<{
  capturePlanSha256: string;
  captureClosureSha256: string;
  evaluationPlanSha256: string;
  captureSourceCommit: string;
  referenceCommit: string;
  appElfSha256: string;
  packageManifestSha256: string;
}>;

export const productionScoreboardRecheckIdentity: ScoreboardRecheckIdentity = {
  capturePlanSha256: expectedCapturePlanSha256,
  captureClosureSha256: expectedCaptureClosureSha256,
  evaluationPlanSha256: expectedEvaluationPlanSha256,
  captureSourceCommit: expectedCaptureSourceCommit,
  referenceCommit: expectedReferenceCommit,
  appElfSha256: expectedAppElfSha256,
  packageManifestSha256: expectedPackageManifestSha256,
};

type CapturedSystemIdentity = Readonly<{
  bootSession: string;
  bootOrdinal: number;
  resetReason: string;
  miningActivity: string;
  startMiningOnBoot: boolean;
}>;

async function requiredChildText(
  processPort: ProcessPort,
  program: string,
  args: readonly string[],
): Promise<string> {
  const outcome = await processPort.run(internalCommandSpec(program, [...args], (value) => value));
  if (outcome.timedOut || outcome.exitCode !== 0) {
    throw failure("evidence_invalid", "scoreboard recheck child validation failed");
  }
  return outcome.stdout.trim();
}

async function readObject(candidate: string, context: string): Promise<{
  readonly document: string;
  readonly value: JsonObject;
}> {
  const document = await readFile(candidate, "utf8");
  try {
    return { document, value: object(JSON.parse(document), context) };
  } catch (error) {
    if (error instanceof ScoreboardEvidenceError) throw error;
    throw failure("evidence_invalid", `${context} is malformed`);
  }
}

async function readArray(candidate: string, context: string): Promise<ScoreboardView> {
  const document = await readFile(candidate, "utf8");
  try {
    const value: unknown = JSON.parse(document);
    if (!Array.isArray(value)) throw new Error("not an array");
    return scoreboardView(value, context);
  } catch (error) {
    if (error instanceof ScoreboardEvidenceError) throw error;
    throw failure("evidence_invalid", `${context} is malformed`);
  }
}

function capturedSystemIdentity(
  value: JsonObject,
  identity: ScoreboardRecheckIdentity,
): CapturedSystemIdentity {
  if (requiredString(value, "sourceCommit", "captured system info") !== identity.captureSourceCommit
    || requiredString(value, "referenceCommit", "captured system info") !== identity.referenceCommit
    || requiredString(value, "appElfSha256", "captured system info") !== identity.appElfSha256) {
    throw failure("evidence_invalid", "captured package identity is invalid");
  }
  return {
    bootSession: requiredString(value, "bootSession", "captured system info"),
    bootOrdinal: requiredInteger(value, "bootOrdinal", "captured system info"),
    resetReason: requiredString(value, "resetReasonCategory", "captured system info"),
    miningActivity: requiredString(value, "miningActivity", "captured system info"),
    startMiningOnBoot: requiredBoolean(value, "startMiningOnBoot", "captured system info"),
  };
}

async function requirePlanBindings(
  workspaceRoot: string,
  options: ScoreboardRecheckOptions,
  identity: ScoreboardRecheckIdentity,
): Promise<void> {
  const [capturePlan, captureClosure, evaluationPlan, tasks] = await Promise.all([
    readFile(path.join(workspaceRoot, options.capturePlan), "utf8"),
    readFile(path.join(workspaceRoot, options.captureClosure), "utf8"),
    readFile(path.join(workspaceRoot, options.evaluationPlan), "utf8"),
    readFile(path.join(workspaceRoot, "TASKS.md"), "utf8"),
  ]);
  const taskHeading = `### ${activeTask} |`;
  const taskStart = tasks.indexOf(taskHeading);
  const maybeTaskEnd = tasks.indexOf("\n### ", taskStart + taskHeading.length);
  const taskBlock = tasks.slice(taskStart, maybeTaskEnd === -1 ? tasks.length : maybeTaskEnd);
  if (sha256(capturePlan) !== identity.capturePlanSha256
    || sha256(captureClosure) !== identity.captureClosureSha256
    || sha256(evaluationPlan) !== identity.evaluationPlanSha256
    || !capturePlan.includes("- Parity row: `STAT-003`")
    || !captureClosure.includes("scoreboard restart persistence is invalid")
    || !evaluationPlan.includes("- Parity row: `STAT-003`")
    || !evaluationPlan.includes(`- Active task: \`${activeTask}\``)
    || taskStart === -1
    || tasks.indexOf(taskHeading, taskStart + taskHeading.length) !== -1
    || !taskBlock.includes(options.evaluationPlan)) {
    throw failure("evidence_invalid", "scoreboard recheck plan binding is invalid");
  }
}

async function requireExactTree(root: string, expected: readonly string[]): Promise<void> {
  const entries = (await readdir(root)).sort();
  const sortedExpected = [...expected].sort();
  if (entries.length !== sortedExpected.length
    || entries.some((entry, index) => entry !== sortedExpected[index])) {
    throw failure("evidence_invalid", "protected evidence inventory is invalid");
  }
}

async function requireDetectorHandoff(detectorOutput: string): Promise<void> {
  const metadata = await lstat(detectorOutput);
  if (metadata.isSymbolicLink() || !metadata.isFile() || (metadata.mode & 0o777) !== 0o600) {
    throw failure("evidence_invalid", "scoreboard detector handoff is invalid");
  }
  const document = await readFile(detectorOutput, "utf8");
  const ports = document
    .split(/\r?\n/u)
    .flatMap((line) => line.startsWith("port: ") ? [line.slice("port: ".length)] : []);
  if (ports.length !== 1 || !portPattern.test(ports[0] ?? "")) {
    throw failure("evidence_invalid", "scoreboard detector handoff is invalid");
  }
}

async function attemptFiles(privateRoot: string): Promise<readonly string[]> {
  const entries = await readdir(privateRoot);
  const restartFiles = entries
    .filter((entry) => /^post-restart-system-(?:[1-9]|[1-5][0-9]|60)\.private\.json$/u.test(entry))
    .sort((left, right) => restartOrdinal(left) - restartOrdinal(right));
  if (restartFiles.length === 0) {
    throw failure("evidence_invalid", "post-restart evidence is missing");
  }
  await requireExactTree(privateRoot, ["campaign", ...fixedAttemptFiles, ...restartFiles]);
  await requireExactTree(path.join(privateRoot, "campaign"), campaignFiles);
  return restartFiles;
}

function restartOrdinal(name: string): number {
  const match = /^post-restart-system-([0-9]+)\.private\.json$/u.exec(name);
  return Number(match?.[1] ?? 0);
}

function requireExpectedCaptureFailure(stdout: string, stderr: string): void {
  if (/(?:password|pool(?:url|user|password)|credential|wifi|ssid)\s*[:=]/iu.test(`${stdout}\n${stderr}`)) {
    throw failure("evidence_invalid", "capture terminal output privacy is invalid");
  }
  const lines = stdout.split(/\r?\n/u).filter((line) => line.length > 0);
  const jsonLines = lines.filter((line) => line.startsWith("{"));
  const commandLines = lines.filter((line) => !line.startsWith("{"));
  if (jsonLines.length !== 1 || commandLines.length > 1
    || commandLines.some((line) => !line.startsWith(
      "bazel run //tools/automation:capture_scoreboard_evidence -- ",
    ))) {
    throw failure("evidence_invalid", "capture terminal output is invalid");
  }
  let result: JsonObject;
  try {
    result = object(JSON.parse(jsonLines[0] ?? ""), "capture terminal result");
  } catch (error) {
    if (error instanceof ScoreboardEvidenceError) throw error;
    throw failure("evidence_invalid", "capture terminal output is invalid");
  }
  const publicValue = object(result["public"], "capture terminal public value");
  const failureLine = "bitaxe-automation: capture-scoreboard-evidence blocked: scoreboard restart persistence is invalid";
  if (requiredString(result, "schema_version", "capture terminal result") !== "bitaxe-automation-result-v1"
    || requiredString(result, "command", "capture terminal result") !== "capture-scoreboard-evidence"
    || requiredString(result, "status", "capture terminal result") !== "blocked"
    || requiredString(result, "category", "capture terminal result") !== "hardware_blocked"
    || requiredString(publicValue, "stage", "capture terminal public value") !== "scoreboard_capture"
    || publicValue["projection_published"] !== false
    || stderr.split(/\r?\n/u).filter((line) => line === failureLine).length !== 1) {
    throw failure("evidence_invalid", "capture did not stop at the expected boundary");
  }
}

async function protectedInputDigest(
  privateRoot: string,
  wrapperRoot: string,
  restartFiles: readonly string[],
): Promise<string> {
  const digest = createHash("sha256");
  const relativeFiles = [
    ...fixedAttemptFiles,
    ...restartFiles,
    ...campaignFiles.map((name) => `campaign/${name}`),
  ].sort();
  for (const relative of relativeFiles) {
    digest.update(relative).update("\0");
    digest.update(await readFile(path.join(privateRoot, relative))).update("\0");
  }
  for (const name of wrapperFiles.filter((candidate) => !candidate.startsWith("recheck."))) {
    digest.update(`wrapper/${name}`).update("\0");
    digest.update(await readFile(path.join(wrapperRoot, name))).update("\0");
  }
  return digest.digest("hex");
}

async function removeCandidate(candidate: string): Promise<void> {
  try {
    await unlink(candidate);
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code !== "ENOENT") throw error;
  }
}

async function requireAbsent(candidate: string): Promise<void> {
  try {
    await lstat(candidate);
    throw failure("evidence_invalid", "scoreboard projection must be absent before recheck");
  } catch (error) {
    if (error instanceof ScoreboardEvidenceError) throw error;
    if ((error as NodeJS.ErrnoException).code !== "ENOENT") throw error;
  }
}

export async function recheckScoreboardEvidence(
  workspaceRoot: string,
  options: ScoreboardRecheckOptions,
  processPort: ProcessPort,
  gitProgram: string,
  validatorProgram: string,
  identity: ScoreboardRecheckIdentity = productionScoreboardRecheckIdentity,
): Promise<ScoreboardEvidence> {
  const privateRoot = assertWithinWorkspace(workspaceRoot, options.privateRoot);
  const wrapperRoot = assertWithinWorkspace(workspaceRoot, options.wrapperRoot);
  const projection = assertWithinWorkspace(workspaceRoot, options.projection);
  const candidate = `${projection}.candidate`;
  if (path.relative(workspaceRoot, privateRoot) !== expectedPrivateRoot
    || path.relative(workspaceRoot, wrapperRoot) !== expectedWrapperRoot
    || path.relative(workspaceRoot, projection) !== expectedProjection
    || options.capturePlan !== expectedCapturePlan
    || options.captureClosure !== expectedCaptureClosure
    || options.evaluationPlan !== expectedEvaluationPlan) {
    throw failure("evidence_invalid", "scoreboard recheck path contract is invalid");
  }
  await requireAbsent(projection);
  await requireAbsent(candidate);

  await requirePlanBindings(workspaceRoot, options, identity);
  await requirePrivateTreeModes(privateRoot);
  await requirePrivateTreeModes(wrapperRoot);
  await requireExactTree(wrapperRoot, wrapperFiles);
  const restartFiles = await attemptFiles(privateRoot);
  await requireDetectorHandoff(path.join(wrapperRoot, "detector.stdout"));
  const [captureStdout, captureStderr] = await Promise.all([
    readFile(path.join(wrapperRoot, "capture.stdout"), "utf8"),
    readFile(path.join(wrapperRoot, "capture.stderr"), "utf8"),
  ]);
  requireExpectedCaptureFailure(captureStdout, captureStderr);

  const [currentSource, pushedSource, reference, dirty, referenceDirty] = await Promise.all([
    requiredChildText(processPort, gitProgram, ["rev-parse", "HEAD"]),
    requiredChildText(processPort, gitProgram, ["rev-parse", "origin/main"]),
    requiredChildText(processPort, gitProgram, ["-C", path.join(workspaceRoot, "reference/esp-miner"), "rev-parse", "HEAD"]),
    requiredChildText(processPort, gitProgram, ["status", "--porcelain", "--untracked-files=no"]),
    requiredChildText(processPort, gitProgram, ["-C", path.join(workspaceRoot, "reference/esp-miner"), "status", "--porcelain"]),
  ]);
  if (currentSource !== pushedSource || dirty !== "" || referenceDirty !== ""
    || reference !== identity.referenceCommit) {
    throw failure("evidence_invalid", "scoreboard recheck source identity is invalid");
  }

  const inventory = await scoreboardSourceInventory(workspaceRoot);
  if (inventory.pathCount !== 32) {
    throw failure("evidence_invalid", "scoreboard recheck source inventory is invalid");
  }
  const campaign = await campaignQuorum(path.join(privateRoot, "campaign"));
  if (!campaign.candidateObserved || !campaign.submitObserved) {
    throw failure("evidence_invalid", "scoreboard campaign outcome is incomplete");
  }

  const beforeValue = (await readObject(
    path.join(privateRoot, "post-campaign-system.private.json"),
    "pre-restart system info",
  )).value;
  const before = capturedSystemIdentity(beforeValue, identity);
  if (before.startMiningOnBoot || before.miningActivity === "active") {
    throw failure("evidence_invalid", "pre-restart safe stop is invalid");
  }
  let after: CapturedSystemIdentity | undefined;
  for (const [index, name] of restartFiles.entries()) {
    const value = (await readObject(path.join(privateRoot, name), "post-restart system info")).value;
    const candidateIdentity = capturedSystemIdentity(value, identity);
    const final = index === restartFiles.length - 1;
    if (!final && candidateIdentity.bootSession !== before.bootSession) {
      throw failure("evidence_invalid", "post-restart identity sequence is invalid");
    }
    if (final) after = candidateIdentity;
  }
  if (after === undefined || after.bootSession === before.bootSession
    || after.bootOrdinal !== before.bootOrdinal + 1
    || after.resetReason !== "software_cpu"
    || !bootMiningDisabled(after.startMiningOnBoot, after.miningActivity)) {
    throw failure("evidence_invalid", "post-restart identity is invalid");
  }

  const [beforeA, beforeB, afterA, afterB, spa, restartResponse] = await Promise.all([
    readArray(path.join(privateRoot, "scoreboard-before-restart-a.private.json"), "pre-restart scoreboard"),
    readArray(path.join(privateRoot, "scoreboard-before-restart-b.private.json"), "pre-restart scoreboard repeat"),
    readArray(path.join(privateRoot, "scoreboard-after-restart-a.private.json"), "post-restart scoreboard"),
    readArray(path.join(privateRoot, "scoreboard-after-restart-b.private.json"), "post-restart scoreboard repeat"),
    readFile(path.join(privateRoot, "scoreboard-route.private.html"), "utf8"),
    readFile(path.join(privateRoot, "restart-response.private.txt"), "utf8"),
  ]);
  if (beforeA.digest !== beforeB.digest || afterA.digest !== afterB.digest
    || !scoreboardRestartPersists(beforeA, afterA)
    || !spa.includes('data-page="scoreboard"') || !spa.includes('/assets/api-client.js')
    || Buffer.byteLength(restartResponse, "utf8") > 1_024) {
    throw failure("evidence_invalid", "scoreboard retained observation quorum is invalid");
  }

  const protectedDigest = await protectedInputDigest(privateRoot, wrapperRoot, restartFiles);
  const evidence: ScoreboardEvidence = {
    schema_version: "bitaxe-scoreboard-evidence-v1",
    board: 205,
    attempt_ordinal: 5,
    source_commit: identity.captureSourceCommit,
    reference_commit: identity.referenceCommit,
    package_manifest_sha256: identity.packageManifestSha256,
    workflow: {
      schema_version: "bitaxe-workflow-identity-v1",
      command: "capture-scoreboard-evidence",
      request_sha256: sha256(JSON.stringify({
        evaluator_source_commit: currentSource,
        capture_plan_sha256: identity.capturePlanSha256,
        capture_closure_sha256: identity.captureClosureSha256,
        evaluation_plan_sha256: identity.evaluationPlanSha256,
        protected_input_sha256: protectedDigest,
        source_inventory_sha256: inventory.digest,
      })),
    },
    source: {
      plan_sha256: identity.evaluationPlanSha256,
      campaign_result_sha256: campaign.resultDigest,
      campaign_network_sha256: campaign.networkDigest,
      campaign_diagnostics_sha256: campaign.diagnosticsDigest,
      source_inventory_sha256: inventory.digest,
      source_semantics_current: true,
      reference_semantics_current: true,
      source_path_count: 32,
    },
    scoreboard: {
      fresh_nvs_seed_without_scoreboard_keys: true,
      live_qualified_nonce_observed: true,
      submit_outcome_observed: true,
      entry_count: beforeA.count,
      exact_wire_shape: true,
      finite_positive_difficulty: true,
      bounded_text_fields: true,
      uppercase_fixed_width_hex: true,
      stable_descending_order: true,
      immediate_repeat_unchanged: true,
      live_spa_route_served: true,
      normal_restart_observed: true,
      boot_session_changed: true,
      boot_ordinal_incremented_once: true,
      software_cpu_reset_observed: true,
      exact_package_after_restart: true,
      boot_mining_disabled: true,
      post_restart_persistence: true,
      post_restart_repeat_unchanged: true,
    },
    detector_admitted: true,
    runtime_identity: "trusted",
    campaign_profile: "conservative",
    campaign_duration_seconds: 600,
    campaign_status: "accepted",
    safe_stop_confirmed: true,
    cleanup_complete: true,
    hardware_rerun_used: false,
    private_modes_valid: true,
    redaction_status: "passed",
  };

  try {
    await mkdir(path.dirname(projection), { recursive: true });
    await writeFile(candidate, `${JSON.stringify(evidence, null, 2)}\n`, {
      encoding: "utf8",
      mode: 0o600,
      flag: "wx",
    });
    await chmod(candidate, 0o600);
    await requiredChildText(processPort, validatorProgram, [candidate]);
    await chmod(candidate, 0o644);
    await rename(candidate, projection);
    return evidence;
  } catch (error) {
    await removeCandidate(candidate);
    if (error instanceof ScoreboardEvidenceError) throw error;
    throw failure("evidence_invalid", "scoreboard recheck publication failed");
  }
}
