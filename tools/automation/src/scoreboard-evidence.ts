import { access, chmod, mkdir, readFile, readdir, rename, stat, unlink, writeFile } from "node:fs/promises";
import path from "node:path";

import {
  internalCommandSpec,
  monitorCommand,
  type ScoreboardEvidence,
} from "./contracts.generated.js";
import {
  fetchJsonArrayFromSameOrigin,
  fetchJsonFromSameOrigin,
  fetchTextResponseFromSameOrigin,
  sendSameOriginRequest,
  uniqueRuntimeOrigin,
} from "./http.js";
import type { ProcessOutcome, ProcessPort } from "./process.js";
import {
  expectedPlanSha256,
  expectedPrivateRoot,
  expectedProjection,
  expectedReferenceCommit,
  expectedWrapperRoot,
  failure,
  object,
  requiredBoolean,
  requiredInteger,
  requiredString,
  scoreboardView,
  ScoreboardEvidenceError,
  sha256,
  validateScoreboardTaskAndSources,
  type JsonObject,
  type ScoreboardView,
} from "./scoreboard-evidence-contract.js";
import { assertWithinWorkspace } from "./workspace.js";

export { ScoreboardEvidenceError } from "./scoreboard-evidence-contract.js";

export type ScoreboardEvidenceOptions = {
  readonly privateRoot: string;
  readonly packageManifest: string;
  readonly wifiCredentials: string;
  readonly poolCredentials: string;
  readonly detectorOutput: string;
  readonly port: string;
  readonly projection: string;
  readonly durationSeconds: number;
  readonly captureTimeoutSeconds: number;
};

type SystemIdentity = Readonly<{
  bootSession: string;
  bootOrdinal: number;
  resetReason: string;
  miningActivity: string;
  startMiningOnBoot: boolean;
}>;

type CampaignQuorum = Readonly<{
  resultDigest: string;
  networkDigest: string;
  diagnosticsDigest: string;
  candidateObserved: boolean;
  submitObserved: boolean;
}>;

function sleep(milliseconds: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, milliseconds));
}

async function requireAbsent(candidate: string, context: string): Promise<void> {
  try {
    await stat(candidate);
    throw failure("evidence_invalid", `${context} must be absent before capture`);
  } catch (error) {
    if (error instanceof ScoreboardEvidenceError) throw error;
    if ((error as NodeJS.ErrnoException).code !== "ENOENT") throw error;
  }
}

async function requireMode(candidate: string, mode: number, directory: boolean): Promise<void> {
  const metadata = await stat(candidate);
  if ((directory ? !metadata.isDirectory() : !metadata.isFile())
    || (metadata.mode & 0o777) !== mode) {
    throw failure("evidence_invalid", "protected evidence mode is invalid");
  }
}

async function requirePrivateTreeModes(root: string): Promise<void> {
  await requireMode(root, 0o700, true);
  for (const entry of await readdir(root, { withFileTypes: true })) {
    const candidate = path.join(root, entry.name);
    if (entry.isDirectory()) await requirePrivateTreeModes(candidate);
    else await requireMode(candidate, 0o600, false);
  }
}

async function childText(
  processPort: ProcessPort,
  program: string,
  args: readonly string[],
  context: string,
): Promise<string> {
  let outcome: ProcessOutcome;
  try {
    outcome = await processPort.run(internalCommandSpec(program, [...args], (value) => value));
  } catch {
    throw failure("process_failed", `${context} launch failed`);
  }
  if (outcome.timedOut) throw failure("timeout", `${context} timed out`);
  if (outcome.exitCode !== 0) throw failure("evidence_invalid", `${context} did not pass`);
  return outcome.stdout.trim();
}

async function readJson(candidate: string, context: string): Promise<{
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

function systemIdentity(value: JsonObject, manifest: JsonObject): SystemIdentity {
  for (const [wire, source] of [
    ["sourceCommit", "source_commit"],
    ["referenceCommit", "reference_commit"],
    ["appElfSha256", "app_elf_sha256"],
  ] as const) {
    if (requiredString(value, wire, "system info")
      !== requiredString(manifest, source, "package manifest")) {
      throw failure("evidence_invalid", "system info does not match exact package");
    }
  }
  return {
    bootSession: requiredString(value, "bootSession", "system info"),
    bootOrdinal: requiredInteger(value, "bootOrdinal", "system info"),
    resetReason: requiredString(value, "resetReasonCategory", "system info"),
    miningActivity: requiredString(value, "miningActivity", "system info"),
    startMiningOnBoot: requiredBoolean(value, "startMiningOnBoot", "system info"),
  };
}

async function runCampaign(
  processPort: ProcessPort,
  flashProgram: string,
  options: ScoreboardEvidenceOptions,
  campaignRoot: string,
): Promise<void> {
  const spec = internalCommandSpec(flashProgram, [
    "mining-campaign", "--stage", "live-share", "--profile", "conservative",
    "--board", "205", "--port", options.port,
    "--manifest", options.packageManifest,
    "--wifi-credentials", options.wifiCredentials,
    "--pool-credentials", options.poolCredentials,
    "--evidence-dir", campaignRoot,
    "--duration-seconds", String(options.durationSeconds), "--redact-evidence",
  ], (value) => value);
  let outcome: ProcessOutcome;
  try {
    outcome = await processPort.run(spec, options.captureTimeoutSeconds * 1_000);
  } catch {
    throw failure("process_failed", "scoreboard mining campaign launch failed");
  }
  if (outcome.timedOut) throw failure("timeout", "scoreboard mining campaign timed out");
  if (outcome.exitCode !== 0) {
    throw failure("hardware_blocked", "scoreboard mining campaign did not complete", {
      campaign_evidence_created: await stat(campaignRoot).then(() => true, () => false),
    });
  }
}

async function campaignQuorum(campaignRoot: string): Promise<CampaignQuorum> {
  const result = await readJson(path.join(campaignRoot, "campaign-result.json"), "campaign result");
  const network = await readJson(path.join(campaignRoot, "campaign-network.private.json"), "campaign network");
  const diagnostics = await readJson(path.join(campaignRoot, "campaign-diagnostics.private.json"), "campaign diagnostics");
  const flash = await readJson(path.join(campaignRoot, "campaign-flash.private.json"), "campaign flash diagnostics");
  const seal = (await readFile(path.join(campaignRoot, "campaign-result.sha256"), "utf8")).trim();
  if (seal !== sha256(result.document)
    || requiredString(result.value, "diagnostics_sha256", "campaign result") !== sha256(diagnostics.document)
    || requiredString(result.value, "network_continuity_sha256", "campaign result") !== sha256(network.document)
    || requiredString(result.value, "flash_diagnostics_sha256", "campaign result") !== sha256(flash.document)
    || requiredString(result.value, "schema", "campaign result") !== "mining-campaign-result-v16"
    || requiredString(result.value, "status", "campaign result") !== "accepted"
    || requiredString(result.value, "stage", "campaign result") !== "live-share"
    || requiredString(result.value, "profile", "campaign result") !== "conservative"
    || requiredInteger(result.value, "duration_seconds", "campaign result") !== 600
    || requiredString(result.value, "runtime_identity", "campaign result") !== "trusted"
    || requiredString(result.value, "pool_config", "campaign result") !== "local_owner_supplied"
    || requiredString(result.value, "safe_stop", "campaign result") !== "confirmed"
    || requiredString(result.value, "usb_cleanup", "campaign result") !== "ready"
    || !requiredBoolean(result.value, "redacted", "campaign result")
    || requiredString(network.value, "schema", "campaign network")
      !== "mining-campaign-network-continuity-v12"
    || requiredString(network.value, "status", "campaign network") !== "accepted"
    || requiredString(network.value, "correlation_failure", "campaign network") !== "none"
    || requiredInteger(network.value, "required_window_count", "campaign network") !== 20
    || requiredInteger(network.value, "covered_window_count", "campaign network") !== 20
    || !requiredBoolean(network.value, "work_renewal_valid", "campaign network")
    || !requiredBoolean(network.value, "terminal_http_valid", "campaign network")
    || !requiredBoolean(network.value, "terminal_websocket_valid", "campaign network")
    || !requiredBoolean(network.value, "terminal_pool_persisted", "campaign network")
    || requiredString(network.value, "terminal_settlement", "campaign network")
      !== "accepted_after_serial_close"
    || !requiredBoolean(network.value, "terminal_close_requested", "campaign network")
    || !requiredBoolean(network.value, "terminal_consumed_observed", "campaign network")
    || !requiredBoolean(network.value, "final_terminal_consumed", "campaign network")
    || !requiredBoolean(network.value, "serial_finished_observed", "campaign network")
    || requiredString(diagnostics.value, "schema", "campaign diagnostics")
      !== "mining-campaign-serial-diagnostics-v4"
    || requiredString(diagnostics.value, "runtime_attestation_mixed_reset_reason", "campaign diagnostics") !== "none"
    || requiredString(diagnostics.value, "panic_signature", "campaign diagnostics") !== "none"
    || requiredInteger(diagnostics.value, "panic_signature_count", "campaign diagnostics") !== 0
    || requiredString(flash.value, "schema", "campaign flash diagnostics")
      !== "mining-campaign-flash-diagnostics-v1"
    || flash.value["nvs"] === null || flash.value["nvs"] === undefined
    || requiredBoolean(flash.value, "raw_output_included", "campaign flash diagnostics")) {
    throw failure("evidence_invalid", "scoreboard campaign quorum is incomplete");
  }
  const candidateCount = [
    "qualified_candidate_count", "below_pool_target_count", "duplicate_candidate_count",
  ].map((field) => requiredInteger(result.value, field, "campaign result"))
    .reduce((sum, count) => sum + count, 0);
  const submitOutcome = requiredString(result.value, "submit_outcome", "campaign result");
  return {
    resultDigest: sha256(result.document),
    networkDigest: sha256(network.document),
    diagnosticsDigest: sha256(diagnostics.document),
    candidateObserved: candidateCount > 0,
    submitObserved: submitOutcome === "accepted" || submitOutcome === "rejected",
  };
}

async function passiveOrigin(
  processPort: ProcessPort,
  flashProgram: string,
  options: ScoreboardEvidenceOptions,
  privateRoot: string,
): Promise<URL> {
  const outcome = await processPort.run(monitorCommand(flashProgram, {
    board: 205,
    port: options.port,
    captureTimeoutSeconds: 25,
  }), 40_000);
  if (outcome.timedOut) throw failure("timeout", "passive origin capture timed out");
  if (outcome.exitCode !== 0) throw failure("hardware_blocked", "passive origin capture failed");
  const output = path.join(privateRoot, "post-campaign-monitor.private.log");
  await writeFile(output, outcome.stdout, { encoding: "utf8", mode: 0o600, flag: "wx" });
  await chmod(output, 0o600);
  try {
    return uniqueRuntimeOrigin(outcome.stdout);
  } catch {
    throw failure("evidence_invalid", "passive runtime origin admission is invalid");
  }
}

async function readScoreboard(
  origin: URL,
  privateRoot: string,
  label: string,
): Promise<ScoreboardView> {
  const response = await fetchJsonArrayFromSameOrigin(
    origin,
    "/api/system/scoreboard",
    path.join(privateRoot, `${label}.private.json`),
  );
  return scoreboardView(response, label);
}

async function awaitRestart(
  origin: URL,
  privateRoot: string,
  manifest: JsonObject,
  before: SystemIdentity,
  wait: (milliseconds: number) => Promise<void>,
): Promise<SystemIdentity> {
  await sendSameOriginRequest(
    origin,
    "/api/system/restart",
    "POST",
    path.join(privateRoot, "restart-response.private.txt"),
  );
  for (let attempt = 1; attempt <= 60; attempt += 1) {
    await wait(2_000);
    try {
      const value = object(await fetchJsonFromSameOrigin(
        origin,
        "/api/system/info",
        path.join(privateRoot, `post-restart-system-${attempt}.private.json`),
      ), "post-restart system info");
      const after = systemIdentity(value, manifest);
      if (after.bootSession === before.bootSession) continue;
      if (after.bootOrdinal !== before.bootOrdinal + 1
        || after.resetReason !== "software_cpu"
        || after.startMiningOnBoot
        || after.miningActivity !== "safe_blocked") {
        throw failure("hardware_blocked", "post-restart system state is invalid");
      }
      return after;
    } catch (error) {
      if (error instanceof ScoreboardEvidenceError) throw error;
    }
  }
  throw failure("hardware_blocked", "post-restart system state was not observed");
}

export async function captureScoreboardEvidence(
  workspaceRoot: string,
  options: ScoreboardEvidenceOptions,
  processPort: ProcessPort,
  flashProgram: string,
  gitProgram: string,
  validatorProgram: string,
  admittedPlanSha256 = expectedPlanSha256,
  wait: (milliseconds: number) => Promise<void> = sleep,
): Promise<ScoreboardEvidence> {
  const privateRoot = assertWithinWorkspace(workspaceRoot, options.privateRoot);
  const manifestPath = assertWithinWorkspace(workspaceRoot, options.packageManifest);
  const wifiPath = assertWithinWorkspace(workspaceRoot, options.wifiCredentials);
  const poolPath = assertWithinWorkspace(workspaceRoot, options.poolCredentials);
  const detectorOutput = assertWithinWorkspace(workspaceRoot, options.detectorOutput);
  const wrapperRoot = path.dirname(detectorOutput);
  const projection = assertWithinWorkspace(workspaceRoot, options.projection);
  const candidate = `${projection}.candidate`;
  if (path.relative(workspaceRoot, privateRoot) !== expectedPrivateRoot
    || path.relative(workspaceRoot, wrapperRoot) !== expectedWrapperRoot
    || path.relative(workspaceRoot, projection) !== expectedProjection
    || options.durationSeconds !== 600 || options.captureTimeoutSeconds !== 1_800) {
    throw failure("evidence_invalid", "STAT-003 protected path or duration contract is invalid");
  }
  await requireAbsent(privateRoot, "protected attempt root");
  await requireAbsent(projection, "scoreboard projection");
  await requireAbsent(candidate, "scoreboard projection candidate");
  const inventory = await validateScoreboardTaskAndSources(workspaceRoot, admittedPlanSha256);
  await access(manifestPath);
  await access(wifiPath);
  await access(poolPath);

  const manifestDocument = await readFile(manifestPath, "utf8");
  const manifest = object(JSON.parse(manifestDocument), "package manifest");
  const currentSourceCommit = await childText(processPort, gitProgram, ["rev-parse", "HEAD"], "source identity");
  const pushedSourceCommit = await childText(processPort, gitProgram, ["rev-parse", "origin/main"], "pushed source identity");
  const referenceCommit = await childText(
    processPort,
    gitProgram,
    ["-C", path.join(workspaceRoot, "reference/esp-miner"), "rev-parse", "HEAD"],
    "reference identity",
  );
  const dirty = await childText(processPort, gitProgram, ["status", "--porcelain", "--untracked-files=no"], "source cleanliness");
  const referenceDirty = await childText(
    processPort,
    gitProgram,
    ["-C", path.join(workspaceRoot, "reference/esp-miner"), "status", "--porcelain"],
    "reference cleanliness",
  );
  if (currentSourceCommit !== pushedSourceCommit || dirty !== "" || referenceDirty !== ""
    || requiredString(manifest, "source_commit", "package manifest") !== currentSourceCommit
    || requiredString(manifest, "reference_commit", "package manifest") !== expectedReferenceCommit
    || referenceCommit !== expectedReferenceCommit) {
    throw failure("evidence_invalid", "exact clean pushed package identity is invalid");
  }

  await mkdir(privateRoot, { recursive: true, mode: 0o700 });
  await chmod(privateRoot, 0o700);
  const campaignRoot = path.join(privateRoot, "campaign");
  try {
    await runCampaign(processPort, flashProgram, options, campaignRoot);
    const campaign = await campaignQuorum(campaignRoot);
    if (!campaign.candidateObserved || !campaign.submitObserved) {
      throw failure("hardware_blocked", "campaign produced no scoreboard-qualified outcome");
    }
    const origin = await passiveOrigin(processPort, flashProgram, options, privateRoot);
    const systemValue = object(await fetchJsonFromSameOrigin(
      origin,
      "/api/system/info",
      path.join(privateRoot, "post-campaign-system.private.json"),
    ), "post-campaign system info");
    const beforeRestart = systemIdentity(systemValue, manifest);
    if (beforeRestart.startMiningOnBoot || beforeRestart.miningActivity === "active") {
      throw failure("hardware_blocked", "post-campaign device is not safely stopped");
    }
    const scoreboard = await readScoreboard(origin, privateRoot, "scoreboard-before-restart-a");
    const scoreboardRepeat = await readScoreboard(origin, privateRoot, "scoreboard-before-restart-b");
    if (scoreboard.digest !== scoreboardRepeat.digest) {
      throw failure("hardware_blocked", "scoreboard immediate repeat changed");
    }
    const spa = await fetchTextResponseFromSameOrigin(
      origin,
      "/scoreboard",
      path.join(privateRoot, "scoreboard-route.private.html"),
    );
    if (!spa.contentType?.startsWith("text/html")
      || !spa.body.includes('data-page="scoreboard"')
      || !spa.body.includes('/assets/api-client.js')) {
      throw failure("hardware_blocked", "live scoreboard SPA route is invalid");
    }
    const afterRestart = await awaitRestart(origin, privateRoot, manifest, beforeRestart, wait);
    const postRestart = await readScoreboard(origin, privateRoot, "scoreboard-after-restart-a");
    const postRestartRepeat = await readScoreboard(origin, privateRoot, "scoreboard-after-restart-b");
    if (scoreboard.digest !== postRestart.digest
      || postRestart.digest !== postRestartRepeat.digest
      || postRestart.count !== scoreboard.count) {
      throw failure("hardware_blocked", "scoreboard restart persistence is invalid");
    }
    const evidence: ScoreboardEvidence = {
      schema_version: "bitaxe-scoreboard-evidence-v1",
      board: 205,
      attempt_ordinal: 2,
      source_commit: currentSourceCommit,
      reference_commit: referenceCommit,
      package_manifest_sha256: sha256(manifestDocument),
      workflow: {
        schema_version: "bitaxe-workflow-identity-v1",
        command: "capture-scoreboard-evidence",
        request_sha256: sha256(JSON.stringify({
          manifest: sha256(manifestDocument),
          plan: admittedPlanSha256,
          inventory: inventory.digest,
          campaign_result: campaign.resultDigest,
          campaign_network: campaign.networkDigest,
          campaign_diagnostics: campaign.diagnosticsDigest,
          scoreboard: scoreboard.digest,
          duration_seconds: options.durationSeconds,
        })),
      },
      source: {
        plan_sha256: admittedPlanSha256,
        campaign_result_sha256: campaign.resultDigest,
        campaign_network_sha256: campaign.networkDigest,
        campaign_diagnostics_sha256: campaign.diagnosticsDigest,
        source_inventory_sha256: inventory.digest,
        source_semantics_current: true,
        reference_semantics_current: true,
        source_path_count: inventory.pathCount,
      },
      scoreboard: {
        fresh_nvs_seed_without_scoreboard_keys: true,
        live_qualified_nonce_observed: true,
        submit_outcome_observed: true,
        entry_count: scoreboard.count,
        exact_wire_shape: true,
        finite_positive_difficulty: true,
        bounded_text_fields: true,
        uppercase_fixed_width_hex: true,
        stable_descending_order: true,
        immediate_repeat_unchanged: true,
        live_spa_route_served: true,
        normal_restart_observed: true,
        boot_session_changed: afterRestart.bootSession !== beforeRestart.bootSession,
        boot_ordinal_incremented_once: afterRestart.bootOrdinal === beforeRestart.bootOrdinal + 1,
        software_cpu_reset_observed: afterRestart.resetReason === "software_cpu",
        exact_package_after_restart: true,
        boot_mining_disabled: !afterRestart.startMiningOnBoot && afterRestart.miningActivity === "safe_blocked",
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
    const privateEvidence = path.join(privateRoot, "final-evidence.private.json");
    await writeFile(privateEvidence, `${JSON.stringify(evidence, null, 2)}\n`, {
      encoding: "utf8", mode: 0o600, flag: "wx",
    });
    await chmod(privateEvidence, 0o600);
    await requireMode(wrapperRoot, 0o700, true);
    for (const name of ["detector.stdout", "detector.stderr", "capture.stdout", "capture.stderr"]) {
      await requireMode(path.join(wrapperRoot, name), 0o600, false);
    }
    await requirePrivateTreeModes(privateRoot);
    await mkdir(path.dirname(projection), { recursive: true });
    await writeFile(candidate, `${JSON.stringify(evidence, null, 2)}\n`, {
      encoding: "utf8", mode: 0o600, flag: "wx",
    });
    await childText(processPort, validatorProgram, [candidate], "scoreboard evidence validator");
    await chmod(candidate, 0o644);
    await rename(candidate, projection);
    return evidence;
  } catch (error) {
    try {
      await unlink(candidate);
    } catch (cleanupError) {
      if ((cleanupError as NodeJS.ErrnoException).code !== "ENOENT") throw cleanupError;
    }
    if (error instanceof ScoreboardEvidenceError) throw error;
    throw failure("evidence_invalid", "scoreboard evidence processing failed");
  }
}
