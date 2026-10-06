import { access, readFile, stat, writeFile, mkdir } from "node:fs/promises";
import path from "node:path";

import {
  flashCommand,
  flashMonitorCommand,
  internalCommandSpec,
  type OtawwwEvidence,
} from "./contracts.generated.js";
import { flashChildFailureFacts, flashEffectEnvironment, inspectFlashEffect } from "./flash-child-diagnostics.js";
import {
  absent, createPrivateRoot, delay, failure, object, OtawwwEvidenceError, privateJson, privateModesValid, text,
  type JsonObject, type RecoveryFacts,
} from "./otawww-evidence-io.js";
import {
  isRecoveryPage, OtawwwInputError, parseOtawwwInputs, parseStationEndpoint, privateIpv4, settingsDigest, sha256,
  unavailableAssetVersion, type OtawwwInputs,
} from "./otawww-evidence-model.js";
import {
  interruptedPrefixBytes, interruptionObserved, provenRestart, sameBuild, servedDigest, systemInfo, uploadComplete,
  type StageContext,
} from "./otawww-evidence-stages.js";
import type { ProcessOutcome, ProcessPort } from "./process.js";
import { verifySemanticEvidenceRedaction } from "./redaction.js";
import { assertWithinWorkspace } from "./workspace.js";

export { OtawwwEvidenceError, postBinaryOnce } from "./otawww-evidence-io.js";

export type OtawwwPhase = "install" | "run";

export type OtawwwEvidenceOptions = {
  readonly phase: OtawwwPhase;
  readonly privateRoot: string;
  readonly packageManifest: string;
  readonly wwwProbeManifest: string;
  readonly port: string;
  readonly maybeEndpointInput?: string;
  readonly projection: string;
  readonly captureTimeoutSeconds: number;
};

/** Builds options from parsed flags; the invocation schema already enforced required values. */
export function otawwwOptions(flag: (name: string) => string | undefined, port: string): OtawwwEvidenceOptions {
  const required = (name: string): string => {
    const value = flag(name);
    if (value === undefined) throw new Error(`missing required option ${name}`);
    return value;
  };
  const maybeEndpointInput = flag("--endpoint-input");
  return {
    phase: required("--phase") === "run" ? "run" : "install",
    ...maybeEndpointInput === undefined ? {} : { maybeEndpointInput },
    privateRoot: required("--private-root"),
    packageManifest: required("--package-manifest"),
    wwwProbeManifest: required("--www-probe-manifest"),
    port,
    projection: required("--projection"),
    captureTimeoutSeconds: Number(required("--capture-timeout-seconds")),
  };
}

export type OtawwwInstallResult = { readonly phase: "install"; readonly installed: true };

/** Polling cadence; the handler erases all 3 MiB before its first read fails and serves nothing meanwhile. */
export type OtawwwTiming = {
  readonly interruptionPollCount: number;
  readonly interruptionPollDelayMs: number;
  readonly baselineRetryDelayMs: number;
  readonly now: () => number;
  /** Which station addresses may be admitted; private LAN addresses only, outside host tests. */
  readonly admitAddress: (ipv4: unknown) => boolean;
};

const defaultTiming: OtawwwTiming = {
  interruptionPollCount: 60,
  interruptionPollDelayMs: 3_000,
  baselineRetryDelayMs: 1_000,
  now: Date.now,
  admitAddress: privateIpv4,
};
/** The install capture only proves a completed boot; the run reads identity over HTTP. */
const installCaptureSeconds = 60;
const baselineAttemptCount = 6;
const installRecordSchema = "bitaxe-otawww-install-v1";

type Paths = {
  readonly privateRoot: string;
  readonly manifest: string;
  readonly probeManifest: string;
  readonly projection: string;
  readonly maybeEndpoint: string | undefined;
};

type Loaded = {
  readonly inputs: OtawwwInputs;
  readonly manifestDigest: string;
  readonly probeDigest: string;
  readonly packageWww: Buffer;
  readonly probeWww: Buffer;
};

async function loadInputs(paths: Paths): Promise<Loaded> {
  const packageWwwPath = path.join(path.dirname(paths.manifest), "www.bin");
  const probeWwwPath = path.join(path.dirname(paths.probeManifest), "www-probe.bin");
  try {
    await Promise.all([access(paths.manifest), access(paths.probeManifest), access(packageWwwPath), access(probeWwwPath)]);
  } catch {
    throw failure("package_invalid", "OTAWWW package inputs are unavailable");
  }
  const [manifestDocument, probeDocument, packageWww, probeWww] = await Promise.all([
    readFile(paths.manifest, "utf8"), readFile(paths.probeManifest, "utf8"), readFile(packageWwwPath), readFile(probeWwwPath),
  ]);
  try {
    return {
      inputs: parseOtawwwInputs(manifestDocument, probeDocument, packageWww, probeWww),
      manifestDigest: sha256(manifestDocument),
      probeDigest: sha256(probeDocument),
      packageWww,
      probeWww,
    };
  } catch (error) {
    throw failure("package_invalid", error instanceof OtawwwInputError ? error.message : "OTAWWW package inputs are malformed");
  }
}

async function recover(processPort: ProcessPort, flashProgram: string, port: string, manifest: string, privateRoot: string): Promise<RecoveryFacts> {
  try {
    const outcome = await processPort.run(flashCommand(flashProgram, {
      board: 205, port, manifest, evidenceDir: path.join(privateRoot, "recovery"),
    }));
    const complete = !outcome.timedOut && outcome.exitCode === 0;
    return { recovery_complete: complete, recovery_flash_used: true, secondary_recovery_failure: !complete };
  } catch {
    return { recovery_complete: false, recovery_flash_used: true, secondary_recovery_failure: true };
  }
}

/** One state-preserving exact-package install; the endpoint handoff and the run follow it. */
async function install(paths: Paths, options: OtawwwEvidenceOptions, processPort: ProcessPort, flashProgram: string): Promise<OtawwwInstallResult> {
  await createPrivateRoot(paths.privateRoot);
  const loaded = await loadInputs(paths);
  const effectPath = path.join(paths.privateRoot, "flash-effect.private.json");
  const expectedEffect = { packageIdentityDigest: loaded.manifestDigest, factoryImageDigest: loaded.inputs.factoryDigest };
  const base = flashMonitorCommand(flashProgram, {
    board: 205, port: options.port, manifest: paths.manifest, captureTimeoutSeconds: installCaptureSeconds,
    evidenceMode: "dual", evidenceDir: paths.privateRoot,
  });
  const spec = internalCommandSpec(base.program, [...base.args], base.result, flashEffectEnvironment(effectPath, expectedEffect));
  let deviceEffectStarted = false;
  let flashCompleted = false;
  try {
    let outcome: ProcessOutcome;
    try {
      outcome = await processPort.run(spec);
    } catch {
      const effect = await inspectFlashEffect(effectPath, expectedEffect);
      deviceEffectStarted = effect.flash_effect_status !== "unavailable" && effect.flash_effect_status !== "failed_no_device_effect";
      throw failure("process_failed", "exact-package flash-monitor launch failed", flashChildFailureFacts(undefined, effect));
    }
    const effect = await inspectFlashEffect(effectPath, expectedEffect);
    deviceEffectStarted = effect.flash_effect_status !== "unavailable" && effect.flash_effect_status !== "failed_no_device_effect";
    const facts = flashChildFailureFacts(outcome, effect);
    if (outcome.timedOut) throw failure("timeout", "exact-package flash-monitor timed out", facts);
    if (outcome.exitCode !== 0) throw failure("hardware_blocked", "exact-package flash-monitor failed", facts);
    if (effect.flash_effect_result_status !== "valid" || effect.flash_effect_status !== "completed") {
      throw failure("evidence_invalid", "exact-package flash effect result is invalid", facts);
    }
    flashCompleted = true;
    await privateJson(path.join(paths.privateRoot, "install.private.json"), {
      schema: installRecordSchema, package_manifest_sha256: loaded.manifestDigest, www_probe_metadata_sha256: loaded.probeDigest,
    });
    return { phase: "install", installed: true };
  } catch (error) {
    const primary = error instanceof OtawwwEvidenceError ? error : failure("evidence_invalid", "OTAWWW install evidence is invalid");
    if (!deviceEffectStarted || flashCompleted) throw primary;
    throw primary.withRecovery(await recover(processPort, flashProgram, options.port, paths.manifest, paths.privateRoot));
  }
}

async function workerLeaseInactive(endpointInput: string): Promise<boolean> {
  try {
    const closed = object(JSON.parse(await readFile(path.join(path.dirname(endpointInput), "closed.json"), "utf8")), "closed record");
    return closed["schema"] === "otawww-endpoint-closed-v1" && closed["endpoint_saved"] === true && closed["worker_lease_inactive"] === true;
  } catch {
    return false;
  }
}

async function readBaseline(context: Pick<StageContext, "origin" | "privateRoot">, retryDelayMs: number): Promise<JsonObject> {
  for (let attempt = 1; attempt <= baselineAttemptCount; attempt += 1) {
    try {
      return await systemInfo(context, `baseline-${String(attempt)}`);
    } catch (error) {
      if (error instanceof OtawwwEvidenceError) throw error;
      await delay(retryDelayMs);
    }
  }
  throw failure("origin_unavailable", "baseline HTTP readiness was not established at the handed-off origin");
}

type StagesResult = {
  readonly probe: Awaited<ReturnType<typeof provenRestart>>;
  readonly interrupted: Awaited<ReturnType<typeof provenRestart>>;
  readonly recovery: Awaited<ReturnType<typeof provenRestart>>;
};

async function runStages(context: StageContext, baseline: JsonObject, loaded: Loaded, restored: (value: boolean) => void): Promise<StagesResult> {
  const { inputs } = loaded;
  restored(false);
  if (!await uploadComplete(context, loaded.probeWww, "probe")) throw failure("update_not_observed", "probe OTAWWW did not complete");
  const probe = await provenRestart(context, baseline, "probe");
  const probeVersion = await servedDigest(context, "/version.txt", "probe-version");
  const probeIndex = await servedDigest(context, "/index.html", "probe-index");
  if (probe.info["axeOSVersion"] !== inputs.probe.probe_label || probeVersion.digest !== inputs.probe.probe_version_txt_sha256
    || probeIndex.digest !== inputs.probe.index_html_sha256) {
    throw failure("asset_identity_mismatch", "served assets do not match the probe image");
  }

  if (!await interruptionObserved(context, probe.info, loaded.packageWww)) {
    throw failure("interruption_not_observed", "interrupted OTAWWW protocol error was not retained");
  }
  const interrupted = await provenRestart(context, await systemInfo(context, "interrupted"), "interrupted");
  const fallbackVersion = await servedDigest(context, "/version.txt", "interrupted-version");
  const recoveryPage = await servedDigest(context, "/recovery", "interrupted-recovery");
  if (interrupted.info["axeOSVersion"] !== unavailableAssetVersion || !isRecoveryPage(fallbackVersion.body) || !isRecoveryPage(recoveryPage.body)) {
    throw failure("interruption_not_observed", "interrupted partition still served web assets after restart");
  }

  if (!await uploadComplete(context, loaded.packageWww, "recovery")) throw failure("recovery_not_observed", "recovery OTAWWW did not complete");
  const recovery = await provenRestart(context, interrupted.info, "recovery");
  const recoveryVersion = await servedDigest(context, "/version.txt", "recovery-version");
  const recoveryIndex = await servedDigest(context, "/index.html", "recovery-index");
  if (recovery.info["axeOSVersion"] !== inputs.buildLabel || recoveryVersion.digest !== inputs.probe.package_version_txt_sha256
    || recoveryIndex.digest !== inputs.probe.index_html_sha256) {
    throw failure("recovery_not_observed", "recovered assets do not match the package image");
  }
  restored(true);
  return { probe, interrupted, recovery };
}

function evidenceFor(loaded: Loaded, stages: StagesResult, captureTimeoutSeconds: number): OtawwwEvidence {
  return {
    schema_version: "bitaxe-otawww-evidence-v1",
    board: 205,
    source_commit: loaded.inputs.source,
    reference_commit: loaded.inputs.reference,
    package_manifest_sha256: loaded.manifestDigest,
    www_probe_metadata_sha256: loaded.probeDigest,
    package_www_sha256: loaded.inputs.probe.package_www_sha256,
    probe_www_sha256: loaded.inputs.probe.probe_www_sha256,
    workflow: {
      schema_version: "bitaxe-workflow-identity-v1",
      command: "capture-otawww-evidence",
      request_sha256: sha256(JSON.stringify({ manifest: loaded.manifestDigest, probe: loaded.probeDigest, timeout: captureTimeoutSeconds })),
    },
    detector_admitted: true,
    otawww: {
      update_request_count: 3,
      probe_update_response_complete: true,
      probe_finished_status_retained: true,
      probe_version_reported: true,
      probe_version_txt_digest_matches: true,
      probe_index_html_digest_matches: true,
      interrupted_upload_attempt_count: 1,
      interrupted_upload_prefix_bytes: interruptedPrefixBytes,
      interruption_protocol_error_retained: true,
      interruption_boot_session_unchanged: true,
      interrupted_assets_unavailable_after_restart: true,
      recovery_page_served_after_interruption: true,
      recovery_update_response_complete: true,
      recovery_version_reported: true,
      recovery_version_txt_digest_matches: true,
      recovery_index_html_digest_matches: true,
      build_identity_unchanged: true,
      hostname_unchanged: true,
      settings_digest_unchanged: true,
    },
    probe_restart_session: stages.probe.session,
    interrupted_restart_session: stages.interrupted.session,
    recovery_restart_session: stages.recovery.session,
    mining_state: "disabled",
    hardware_control_state: "disabled",
    cleanup_complete: true,
    recovery_flash_used: false,
    private_modes_valid: true,
    redaction_status: "passed",
  };
}

/** The update, interruption and recovery stages at the Gate-handed-off origin of the installed boot. */
async function run(
  paths: Paths, options: OtawwwEvidenceOptions, processPort: ProcessPort, programs: { flash: string; deviceSession: string; validator: string },
  timing: OtawwwTiming,
): Promise<OtawwwEvidence> {
  const loaded = await loadInputs(paths);
  const record = object(JSON.parse(await readFile(path.join(paths.privateRoot, "install.private.json"), "utf8")), "install record");
  if (record["schema"] !== installRecordSchema || record["package_manifest_sha256"] !== loaded.manifestDigest
    || record["www_probe_metadata_sha256"] !== loaded.probeDigest) {
    throw failure("package_invalid", "run inputs differ from the installed package");
  }
  await absent(path.join(paths.privateRoot, "final-evidence.private.json"), "final evidence");
  if (paths.maybeEndpoint === undefined) throw failure("origin_unavailable", "the run requires the Gate endpoint handoff");
  let endpoint;
  try {
    endpoint = parseStationEndpoint(await readFile(paths.maybeEndpoint, "utf8"), timing.now(), timing.admitAddress);
  } catch {
    throw failure("origin_unavailable", "the Gate endpoint handoff is missing, invalid or stale");
  }
  const io = { origin: endpoint.origin, privateRoot: paths.privateRoot };
  const baseline = await readBaseline(io, timing.baselineRetryDelayMs);
  if (!sameBuild(baseline, loaded.inputs, "baseline info") || baseline["bootOrdinal"] !== endpoint.bootOrdinal) {
    throw failure("evidence_invalid", "the handed-off origin does not serve the installed boot");
  }
  if (baseline["axeOSVersion"] !== loaded.inputs.buildLabel) throw failure("asset_identity_mismatch", "installed web assets do not report the package label");
  // The boot's one-time safe-state log line rotates out of the 512 KiB ring within minutes; the Gate's closed
  // Worker state for this same boot proves no lease, and therefore no mining, instead.
  if (!await workerLeaseInactive(paths.maybeEndpoint)) throw failure("hardware_blocked", "the handoff did not prove an inactive Worker lease");
  const hostnameDigest = sha256(text(baseline, "hostname", "baseline info"));
  const baselineSettings = settingsDigest(baseline);
  const context: StageContext = {
    ...io, inputs: loaded.inputs, processPort, deviceSessionProgram: programs.deviceSession, port: options.port,
    captureTimeoutSeconds: options.captureTimeoutSeconds, hostnameDigest,
    interruptionPollCount: timing.interruptionPollCount, interruptionPollDelayMs: timing.interruptionPollDelayMs,
  };
  let packageAssetsInstalled = true;
  try {
    const stages = await runStages(context, baseline, loaded, (value) => { packageAssetsInstalled = value; });
    const finalInfo = stages.recovery.info;
    if (sha256(text(finalInfo, "hostname", "final info")) !== hostnameDigest || settingsDigest(finalInfo) !== baselineSettings) {
      throw failure("nvs_not_preserved", "stored settings changed across the OTAWWW run");
    }
    if (!await privateModesValid(paths.privateRoot)) throw failure("evidence_invalid", "private artifact modes are invalid");
    const evidence = evidenceFor(loaded, stages, options.captureTimeoutSeconds);
    const candidate = path.join(paths.privateRoot, "final-evidence.private.json");
    await privateJson(candidate, evidence);
    await verifySemanticEvidenceRedaction(paths.privateRoot);
    let validation: ProcessOutcome;
    try {
      validation = await processPort.run(internalCommandSpec(programs.validator, [candidate], (value) => value));
    } catch {
      throw failure("process_failed", "OTAWWW validator launch failed");
    }
    if (validation.timedOut || validation.exitCode !== 0) throw failure("evidence_invalid", "OTAWWW validation failed");
    await mkdir(path.dirname(paths.projection), { recursive: true });
    await writeFile(paths.projection, `${JSON.stringify(evidence, null, 2)}\n`, { encoding: "utf8", flag: "wx" });
    return evidence;
  } catch (error) {
    const primary = error instanceof OtawwwEvidenceError ? error : failure("evidence_invalid", "OTAWWW orchestration evidence is invalid");
    if (packageAssetsInstalled) throw primary;
    throw primary.withRecovery(await recover(processPort, programs.flash, options.port, paths.manifest, paths.privateRoot));
  }
}

export async function captureOtawwwEvidence(
  workspaceRoot: string,
  options: OtawwwEvidenceOptions,
  processPort: ProcessPort,
  flashProgram: string,
  deviceSessionProgram: string,
  validatorProgram: string,
  timing: OtawwwTiming = defaultTiming,
): Promise<OtawwwEvidence | OtawwwInstallResult> {
  const paths: Paths = {
    privateRoot: assertWithinWorkspace(workspaceRoot, options.privateRoot),
    manifest: assertWithinWorkspace(workspaceRoot, options.packageManifest),
    probeManifest: assertWithinWorkspace(workspaceRoot, options.wwwProbeManifest),
    projection: assertWithinWorkspace(workspaceRoot, options.projection),
    maybeEndpoint: options.maybeEndpointInput === undefined ? undefined : assertWithinWorkspace(workspaceRoot, options.maybeEndpointInput),
  };
  try {
    await stat(paths.projection);
    throw failure("evidence_invalid", "projection must be absent before launch");
  } catch (error) {
    if (error instanceof OtawwwEvidenceError) throw error;
  }
  if (options.phase === "install") return install(paths, options, processPort, flashProgram);
  return run(paths, options, processPort, { flash: flashProgram, deviceSession: deviceSessionProgram, validator: validatorProgram }, timing);
}
