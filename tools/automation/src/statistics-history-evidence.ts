import { access, chmod, mkdir, readFile, readdir, rename, stat, unlink, writeFile } from "node:fs/promises";
import path from "node:path";

import {
  flashMonitorCommand,
  internalCommandSpec,
  type StatisticsHistoryEvidence,
} from "./contracts.generated.js";
import {
  factoryImageDigest,
  flashChildFailureFacts,
  flashEffectEnvironment,
  inspectFlashEffect,
} from "./flash-child-diagnostics.js";
import { fetchJsonFromSameOrigin, sendSameOriginRequest, uniqueRuntimeOrigin } from "./http.js";
import type { ProcessOutcome, ProcessPort } from "./process.js";
import {
  expectedLabels,
  expectedPlanSha256,
  expectedPrivateRoot,
  expectedProjection,
  expectedReferenceCommit,
  expectedWrapperRoot,
  failure,
  historyDigest,
  historyView,
  noRecovery,
  object,
  requiredFrequency,
  requiredString,
  sha256,
  StatisticsHistoryEvidenceError,
  type HistoryView,
  type JsonObject,
  type RecoveryFacts,
  validateIdentity,
  validateStatisticsHistoryTaskAndSources,
} from "./statistics-history-contract.js";
import { hasPassiveSafeState } from "./version-evidence.js";
import { assertWithinWorkspace } from "./workspace.js";

export type StatisticsHistoryEvidenceOptions = {
  readonly privateRoot: string;
  readonly packageManifest: string;
  readonly wifiCredentials: string;
  readonly detectorOutput: string;
  readonly port: string;
  readonly projection: string;
  readonly captureTimeoutSeconds: number;
};

type ExpectedEffectIdentity = {
  readonly packageIdentityDigest: string;
  readonly factoryImageDigest: string;
};

export { StatisticsHistoryEvidenceError } from "./statistics-history-contract.js";

export const flashMonitorCleanupGraceMilliseconds = 60_000;

export function flashMonitorSupervisorLifetimeMilliseconds(
  childTimeoutMilliseconds: number,
  cleanupGraceMilliseconds = flashMonitorCleanupGraceMilliseconds,
): number {
  if (!Number.isSafeInteger(childTimeoutMilliseconds) || childTimeoutMilliseconds <= 0
    || !Number.isSafeInteger(cleanupGraceMilliseconds) || cleanupGraceMilliseconds <= 0) {
    throw failure("evidence_invalid", "flash-monitor supervisor lifetime is invalid");
  }
  const supervisorLifetimeMilliseconds = childTimeoutMilliseconds + cleanupGraceMilliseconds;
  if (!Number.isSafeInteger(supervisorLifetimeMilliseconds)
    || supervisorLifetimeMilliseconds <= childTimeoutMilliseconds) {
    throw failure("evidence_invalid", "flash-monitor supervisor lifetime is not strictly later");
  }
  return supervisorLifetimeMilliseconds;
}

function sleep(milliseconds: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, milliseconds));
}

async function requireAbsent(candidate: string, context: string): Promise<void> {
  try {
    await stat(candidate);
    throw failure("evidence_invalid", `${context} must be absent before capture`);
  } catch (error) {
    if (error instanceof StatisticsHistoryEvidenceError) throw error;
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

async function createPrivateRoot(root: string): Promise<void> {
  await mkdir(root, { recursive: true, mode: 0o700 });
  await chmod(root, 0o700);
}

async function privateJson(output: string, value: unknown): Promise<void> {
  await writeFile(output, `${JSON.stringify(value, null, 2)}\n`, {
    encoding: "utf8",
    flag: "wx",
    mode: 0o600,
  });
  await chmod(output, 0o600);
}

async function readFrequency(
  origin: URL,
  output: string,
  manifest: JsonObject,
): Promise<number> {
  const value = object(await fetchJsonFromSameOrigin(origin, "/api/system/info", output), "system info");
  validateIdentity(value, manifest);
  return requiredFrequency(value, "system info");
}

async function readHistory(origin: URL, output: string): Promise<HistoryView> {
  return historyView(
    await fetchJsonFromSameOrigin(origin, "/api/system/statistics", output),
    "statistics response",
  );
}

async function patchFrequency(origin: URL, output: string, frequency: number): Promise<void> {
  await sendSameOriginRequest(origin, "/api/system", "PATCH", output, { statsFrequency: frequency });
}

async function restoreAndConfirm(
  origin: URL,
  privateRoot: string,
  original: number,
  manifest: JsonObject,
  prefix: string,
): Promise<boolean> {
  await patchFrequency(origin, path.join(privateRoot, `${prefix}-restore.private.txt`), original);
  return await readFrequency(
    origin,
    path.join(privateRoot, `${prefix}-restored.private.json`),
    manifest,
  ) === original;
}

async function completedFlashMonitor(
  processPort: ProcessPort,
  flashProgram: string,
  evidenceRoot: string,
  effectPath: string,
  options: StatisticsHistoryEvidenceOptions,
  manifestPath: string,
  credentialsPath: string,
  expectedEffectIdentity: ExpectedEffectIdentity,
): Promise<ProcessOutcome> {
  const base = flashMonitorCommand(flashProgram, {
    board: 205,
    port: options.port,
    manifest: manifestPath,
    wifiCredentials: credentialsPath,
    captureTimeoutSeconds: options.captureTimeoutSeconds,
    evidenceMode: "dual",
    evidenceDir: evidenceRoot,
  });
  const spec = internalCommandSpec(
    base.program,
    [...base.args],
    base.result,
    flashEffectEnvironment(effectPath, expectedEffectIdentity),
  );
  let outcome: ProcessOutcome;
  try {
    outcome = await processPort.run(
      spec,
      flashMonitorSupervisorLifetimeMilliseconds(options.captureTimeoutSeconds * 1_000),
    );
  } catch {
    const effect = await inspectFlashEffect(effectPath, expectedEffectIdentity);
    throw failure("process_failed", "exact-package flash-monitor launch failed",
      flashChildFailureFacts(undefined, effect));
  }
  const effect = await inspectFlashEffect(effectPath, expectedEffectIdentity);
  const facts = flashChildFailureFacts(outcome, effect);
  if (outcome.timedOut) throw failure("timeout", "exact-package flash-monitor timed out", facts);
  if (outcome.exitCode !== 0) throw failure("process_failed", "exact-package flash-monitor failed", facts);
  if (effect.flash_effect_result_status !== "valid" || effect.flash_effect_status !== "completed") {
    throw failure("evidence_invalid", "exact-package flash effect result is invalid", facts);
  }
  return outcome;
}

async function recover(
  processPort: ProcessPort,
  flashProgram: string,
  origin: URL,
  privateRoot: string,
  original: number,
  manifest: JsonObject,
  options: StatisticsHistoryEvidenceOptions,
  manifestPath: string,
  credentialsPath: string,
  expectedEffectIdentity: ExpectedEffectIdentity,
): Promise<RecoveryFacts> {
  try {
    if (await restoreAndConfirm(origin, privateRoot, original, manifest, "recovery-primary")) {
      return { ...noRecovery, restoration_complete: true };
    }
  } catch {
    // The bounded exact-package flash-monitor below is the sole fallback.
  }
  const recoveryRoot = path.join(privateRoot, "recovery-flash");
  try {
    await createPrivateRoot(recoveryRoot);
    await completedFlashMonitor(
      processPort,
      flashProgram,
      recoveryRoot,
      path.join(recoveryRoot, "flash-effect.private.json"),
      options,
      manifestPath,
      credentialsPath,
      expectedEffectIdentity,
    );
    const monitor = await readFile(path.join(recoveryRoot, "flash-monitor.classifier-input.log"), "utf8");
    const recoveredOrigin = uniqueRuntimeOrigin(monitor);
    const restored = await restoreAndConfirm(
      recoveredOrigin,
      recoveryRoot,
      original,
      manifest,
      "recovery-flash",
    );
    return {
      restoration_complete: restored,
      recovery_flash_used: true,
      recovery_origin_readmitted: true,
      secondary_recovery_failure: !restored,
    };
  } catch {
    return {
      restoration_complete: false,
      recovery_flash_used: true,
      recovery_origin_readmitted: false,
      secondary_recovery_failure: true,
    };
  }
}

async function observeHistory(
  origin: URL,
  privateRoot: string,
  wait: (milliseconds: number) => Promise<void>,
): Promise<{
  readonly stable: HistoryView;
  readonly later: HistoryView;
}> {
  let stable: HistoryView | undefined;
  for (let attempt = 1; attempt <= 8; attempt += 1) {
    await wait(1_050);
    const first = await readHistory(origin, path.join(privateRoot, `history-${attempt}-a.private.json`));
    const second = await readHistory(origin, path.join(privateRoot, `history-${attempt}-b.private.json`));
    if (first.rows.length >= 3 && historyDigest(first) === historyDigest(second)) {
      stable = second;
      break;
    }
  }
  if (stable === undefined) {
    throw failure("hardware_blocked", "stable producer-owned statistics history was not observed");
  }
  const stableLast = stable.timestamps.at(-1);
  for (let attempt = 1; attempt <= 3; attempt += 1) {
    await wait(1_050);
    const later = await readHistory(origin, path.join(privateRoot, `history-later-${attempt}.private.json`));
    if (stableLast !== undefined && (later.timestamps.at(-1) ?? -1) > stableLast) {
      return { stable, later };
    }
  }
  throw failure("hardware_blocked", "statistics history did not grow on the producer cadence");
}

export async function captureStatisticsHistoryEvidence(
  workspaceRoot: string,
  options: StatisticsHistoryEvidenceOptions,
  processPort: ProcessPort,
  flashProgram: string,
  gitProgram: string,
  validatorProgram: string,
  admittedPlanSha256 = expectedPlanSha256,
  wait: (milliseconds: number) => Promise<void> = sleep,
): Promise<StatisticsHistoryEvidence> {
  const privateRoot = assertWithinWorkspace(workspaceRoot, options.privateRoot);
  const manifestPath = assertWithinWorkspace(workspaceRoot, options.packageManifest);
  const credentialsPath = assertWithinWorkspace(workspaceRoot, options.wifiCredentials);
  const detectorOutput = assertWithinWorkspace(workspaceRoot, options.detectorOutput);
  const wrapperRoot = path.dirname(detectorOutput);
  const projection = assertWithinWorkspace(workspaceRoot, options.projection);
  const candidate = `${projection}.candidate`;
  if (path.relative(workspaceRoot, privateRoot) !== expectedPrivateRoot
    || path.relative(workspaceRoot, wrapperRoot) !== expectedWrapperRoot
    || path.relative(workspaceRoot, projection) !== expectedProjection
    || options.captureTimeoutSeconds !== 360) {
    throw failure("evidence_invalid", "STAT-002 protected path contract is invalid");
  }
  await requireAbsent(privateRoot, "protected attempt root");
  await requireAbsent(projection, "statistics history projection");
  await requireAbsent(candidate, "statistics history projection candidate");
  await access(manifestPath);
  await access(credentialsPath);
  await validateStatisticsHistoryTaskAndSources(workspaceRoot, admittedPlanSha256);

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
  const dirty = await childText(
    processPort,
    gitProgram,
    ["status", "--porcelain", "--untracked-files=no"],
    "source cleanliness",
  );
  const referenceDirty = await childText(
    processPort,
    gitProgram,
    ["-C", path.join(workspaceRoot, "reference/esp-miner"), "status", "--porcelain"],
    "reference cleanliness",
  );
  if (currentSourceCommit !== pushedSourceCommit || dirty !== ""
    || referenceDirty !== ""
    || requiredString(manifest, "source_commit", "package manifest") !== currentSourceCommit
    || requiredString(manifest, "reference_commit", "package manifest") !== expectedReferenceCommit
    || referenceCommit !== expectedReferenceCommit) {
    throw failure("evidence_invalid", "exact clean pushed package identity is invalid");
  }
  let factoryDigest: string;
  try {
    factoryDigest = factoryImageDigest(manifest);
  } catch {
    throw failure("evidence_invalid", "package manifest factory image is invalid");
  }
  const expectedEffectIdentity = {
    packageIdentityDigest: sha256(manifestDocument),
    factoryImageDigest: factoryDigest,
  };

  await createPrivateRoot(privateRoot);
  await completedFlashMonitor(
    processPort,
    flashProgram,
    privateRoot,
    path.join(privateRoot, "flash-effect.private.json"),
    options,
    manifestPath,
    credentialsPath,
    expectedEffectIdentity,
  );
  const monitor = await readFile(path.join(privateRoot, "flash-monitor.classifier-input.log"), "utf8");
  if (!hasPassiveSafeState(monitor)) {
    throw failure("evidence_invalid", "boot lacks passive safe-state evidence");
  }
  let origin: URL;
  try {
    origin = uniqueRuntimeOrigin(monitor);
  } catch {
    throw failure("evidence_invalid", "runtime origin admission is invalid");
  }
  const original = await readFrequency(
    origin,
    path.join(privateRoot, "original-system-info.private.json"),
    manifest,
  );
  const enabled = original === 1 ? 2 : 1;
  let mutated = false;
  try {
    await patchFrequency(origin, path.join(privateRoot, "enable.private.txt"), enabled);
    mutated = true;
    const enabledReadback = await readFrequency(
      origin,
      path.join(privateRoot, "enabled-system-info.private.json"),
      manifest,
    );
    if (enabledReadback !== enabled) {
      throw failure("hardware_blocked", "statistics enable readback mismatch");
    }
    const observed = await observeHistory(origin, privateRoot, wait);
    const intervals = observed.later.timestamps.slice(1).map(
      (timestamp, index) => timestamp - (observed.later.timestamps[index] ?? timestamp),
    );
    if (intervals.length < 2 || intervals.some((interval) => interval < 750 || interval > 1_500)) {
      throw failure("hardware_blocked", "statistics producer cadence is outside tolerance");
    }
    if (!await restoreAndConfirm(origin, privateRoot, original, manifest, "final")) {
      throw failure("hardware_blocked", "statistics setting restoration readback mismatch");
    }
    let clearStatus: "observed" | "not_applicable" = "not_applicable";
    if (original === 0) {
      let cleared = false;
      for (let attempt = 1; attempt <= 3; attempt += 1) {
        await wait(1_050);
        const view = await readHistory(
          origin,
          path.join(privateRoot, `cleared-history-${attempt}.private.json`),
        );
        if (view.rows.length === 0) {
          cleared = true;
          break;
        }
      }
      if (!cleared) {
        throw failure("hardware_blocked", "zero statistics setting did not clear history");
      }
      clearStatus = "observed";
    }
    mutated = false;
    const evidence: StatisticsHistoryEvidence = {
      schema_version: "bitaxe-statistics-history-evidence-v1",
      board: 205,
      source_commit: currentSourceCommit,
      reference_commit: referenceCommit,
      package_manifest_sha256: sha256(manifestDocument),
      plan_sha256: admittedPlanSha256,
      workflow: {
        schema_version: "bitaxe-workflow-identity-v1",
        command: "capture-statistics-history-evidence",
        request_sha256: sha256(JSON.stringify({
          manifest: sha256(manifestDocument),
          plan: admittedPlanSha256,
          field: "statsFrequency",
          timeout: options.captureTimeoutSeconds,
        })),
      },
      detector_admitted: true,
      boot_observed: true,
      same_origin_observed: true,
      statistics_history: {
        original_setting_sha256: sha256(String(original)),
        enabled_setting_sha256: sha256(String(enabled)),
        mutation_request_field_count: 1,
        enabled_readback_confirmed: true,
        label_count: expectedLabels.length,
        row_width: expectedLabels.length,
        sample_count: observed.later.rows.length,
        interval_count: intervals.length,
        minimum_interval_ms: Math.min(...intervals),
        maximum_interval_ms: Math.max(...intervals),
        timestamps_strictly_increasing: true,
        finite_numeric_rows: true,
        immediate_repeat_unchanged: true,
        later_producer_growth: true,
        restoration_complete: true,
        zero_setting_clear_status: clearStatus,
      },
      mining_state: "disabled",
      hardware_control_state: "disabled",
      recovery_flash_used: false,
      recovery_origin_readmitted: false,
      private_modes_valid: true,
      cleanup_complete: true,
      redaction_status: "passed",
    };
    const privateCandidate = path.join(privateRoot, "final-evidence.private.json");
    await privateJson(privateCandidate, evidence);
    await requireMode(wrapperRoot, 0o700, true);
    for (const name of ["detector.stdout", "detector.stderr", "capture.stdout", "capture.stderr"]) {
      await requireMode(path.join(wrapperRoot, name), 0o600, false);
    }
    await requirePrivateTreeModes(privateRoot);
    await mkdir(path.dirname(projection), { recursive: true });
    await writeFile(candidate, `${JSON.stringify(evidence, null, 2)}\n`, {
      encoding: "utf8",
      flag: "wx",
      mode: 0o600,
    });
    await childText(processPort, validatorProgram, [candidate], "statistics history evidence validator");
    await chmod(candidate, 0o644);
    await rename(candidate, projection);
    return evidence;
  } catch (error) {
    try {
      await unlink(candidate);
    } catch (cleanupError) {
      if ((cleanupError as NodeJS.ErrnoException).code !== "ENOENT") throw cleanupError;
    }
    const primary = error instanceof StatisticsHistoryEvidenceError
      ? error
      : failure("evidence_invalid", "statistics history orchestration evidence is invalid");
    if (!mutated) throw primary;
    throw primary.withRecovery(await recover(
      processPort,
      flashProgram,
      origin,
      privateRoot,
      original,
      manifest,
      options,
      manifestPath,
      credentialsPath,
      expectedEffectIdentity,
    ));
  }
}
