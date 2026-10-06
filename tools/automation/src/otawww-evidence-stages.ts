import path from "node:path";

import { internalCommandSpec } from "./contracts.generated.js";
import { isDeviceSessionProjectionFailure, readClosedDeviceSession } from "./device-session-projection.js";
import { fetchJsonFromSameOrigin, fetchTextFromSameOrigin } from "./http.js";
import { sendInterruptedUpload } from "./interrupted-upload.js";
import {
  delay, failure, object, ordinal, postBinaryOnce, privateDir, privateJson, text, type JsonObject,
} from "./otawww-evidence-io.js";
import {
  retainedLine, sha256, wwwFinishedLine, wwwProtocolErrorLine, wwwRoute, wwwSuccessBody, type OtawwwInputs,
} from "./otawww-evidence-model.js";
import type { ProcessOutcome, ProcessPort } from "./process.js";
import { passiveSafeStateLine } from "./sdkconfig-rollback-retained-log.js";

export const interruptedPrefixBytes = 4_096;
/** Whole-partition erase plus a 3 MiB body over Wi-Fi; the handler blocks the HTTP task throughout. */
const uploadTimeoutMs = 600_000;
/** Session facts the OTAWWW validator requires; checked per restart so a gap stops the run early. */
const requiredSessionFacts = [
  "same_physical_device", "stable_enumeration", "reader_armed", "pre_restart_serial_delivery",
  "post_restart_serial_delivery", "service_loss_observed", "trusted_origin_preserved", "application_recovered",
  "build_identity_matches", "boot_session_changed", "boot_ordinal_advanced_by_one", "software_reset_observed",
  "postcondition_matches", "cleanup_complete",
] as const;

export type StageContext = {
  readonly origin: URL;
  readonly privateRoot: string;
  readonly inputs: OtawwwInputs;
  readonly processPort: ProcessPort;
  readonly deviceSessionProgram: string;
  readonly port: string;
  readonly captureTimeoutSeconds: number;
  readonly hostnameDigest: string;
  readonly interruptionPollCount: number;
  readonly interruptionPollDelayMs: number;
};

export function sameBuild(info: JsonObject, inputs: OtawwwInputs, context: string): boolean {
  return text(info, "sourceCommit", context) === inputs.source
    && text(info, "referenceCommit", context) === inputs.reference
    && text(info, "appElfSha256", context) === inputs.app;
}

export async function systemInfo(context: Pick<StageContext, "origin" | "privateRoot">, name: string): Promise<JsonObject> {
  return object(await fetchJsonFromSameOrigin(context.origin, "/api/system/info",
    path.join(context.privateRoot, `${name}-system-info.private.json`)), `${name} system info`);
}

export async function logs(context: Pick<StageContext, "origin" | "privateRoot">, name: string): Promise<string> {
  return fetchTextFromSameOrigin(context.origin, "/api/system/logs", path.join(context.privateRoot, `${name}-logs.private.txt`));
}

export async function servedDigest(context: StageContext, route: string, name: string): Promise<{ digest: string; body: string }> {
  const body = await fetchTextFromSameOrigin(context.origin, route, path.join(context.privateRoot, `${name}.private.txt`));
  return { digest: sha256(body), body };
}

export async function uploadComplete(context: StageContext, image: Buffer, name: string): Promise<boolean> {
  let response;
  try {
    response = await postBinaryOnce(context.origin, wwwRoute, image, uploadTimeoutMs);
  } catch {
    return false;
  }
  await privateJson(path.join(context.privateRoot, `${name}-upload-response.private.json`), response);
  if (response.status !== 200 || response.body !== wwwSuccessBody) return false;
  return retainedLine(await logs(context, `${name}-upload`), wwwFinishedLine);
}

/** One HTTP restart proven by `device-session reboot-live`; returns the closed session and new info. */
export async function provenRestart(
  context: StageContext,
  before: JsonObject,
  name: string,
): Promise<{ session: JsonObject; info: JsonObject }> {
  const stage = { stage: `${name}_restart` };
  const intentPath = path.join(context.privateRoot, `${name}-reboot-intent.private.json`);
  const sessionRoot = path.join(context.privateRoot, `${name}-session`);
  const projectionPath = path.join(context.privateRoot, `${name}-session-projection.private.json`);
  const runningPartition = text(before, "runningPartition", `${name} pre-restart info`);
  await privateJson(intentPath, {
    schema_version: "esp-device-session-reboot-intent-v1",
    board_category: "205",
    trusted_origin: context.origin.origin,
    baseline: {
      boot_session: text(before, "bootSession", `${name} pre-restart info`),
      boot_ordinal: ordinal(before, `${name} pre-restart info`),
      source_commit: context.inputs.source,
      reference_commit: context.inputs.reference,
      app_elf_sha256: context.inputs.app,
      running_partition: runningPartition,
    },
    expected_postcondition: {
      hostname_sha256: context.hostnameDigest,
      app_elf_sha256: context.inputs.app,
      running_partition: runningPartition,
    },
  });
  await privateDir(sessionRoot);
  let outcome: ProcessOutcome;
  try {
    outcome = await context.processPort.run(internalCommandSpec(context.deviceSessionProgram, [
      "reboot-live", "--port", context.port,
      "--intent-input", intentPath,
      "--private-root", sessionRoot,
      "--projection-output", projectionPath,
      "--timeout-seconds", String(context.captureTimeoutSeconds),
    ], (value) => value));
  } catch {
    throw failure("process_failed", `${name} restart launch failed`, stage);
  }
  if (outcome.timedOut) throw failure("timeout", `${name} restart timed out`, stage);
  let session: JsonObject;
  try {
    session = await readClosedDeviceSession(projectionPath);
  } catch (error) {
    if (isDeviceSessionProjectionFailure(error)) throw failure(error.category, error.message, { ...error.facts, ...stage });
    throw failure("evidence_invalid", `${name} restart projection is invalid`, stage);
  }
  if (outcome.exitCode !== 0) throw failure("hardware_blocked", `${name} restart was not proven`, stage);
  const missing = requiredSessionFacts.filter((field) => session[field] !== true);
  if (missing.length > 0) throw failure("hardware_blocked", `${name} restart session lacks required facts`, { ...stage, missing_facts: missing });
  const info = await systemInfo(context, `${name}-restart`);
  if (
    !sameBuild(info, context.inputs, `${name} restart info`)
    || ordinal(info, `${name} restart info`) !== ordinal(before, `${name} pre-restart info`) + 1
    || text(info, "bootSession", `${name} restart info`) === text(before, "bootSession", `${name} pre-restart info`)
  ) {
    throw failure("hardware_blocked", `${name} restart identity is invalid`, stage);
  }
  if (!retainedLine(await logs(context, `${name}-restart`), passiveSafeStateLine)) {
    throw failure("hardware_blocked", `${name} boot lacks the passive safe state`, stage);
  }
  return { session, info };
}

export async function interruptionObserved(context: StageContext, before: JsonObject, image: Buffer): Promise<boolean> {
  await sendInterruptedUpload(context.origin, wwwRoute, image, interruptedPrefixBytes);
  for (let attempt = 1; attempt <= context.interruptionPollCount; attempt += 1) {
    await delay(context.interruptionPollDelayMs);
    let current: JsonObject;
    let retained: string;
    try {
      current = await systemInfo(context, `interruption-${String(attempt)}`);
      retained = await logs(context, `interruption-${String(attempt)}`);
    } catch {
      continue; // The handler holds the only HTTP task while it erases.
    }
    if (
      !sameBuild(current, context.inputs, "post-interruption info")
      || text(current, "bootSession", "post-interruption info") !== text(before, "bootSession", "pre-interruption info")
    ) {
      throw failure("interruption_not_observed", "interrupted upload changed the running application");
    }
    if (retainedLine(retained, wwwProtocolErrorLine)) return true;
  }
  return false;
}
