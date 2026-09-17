import { proof } from "../str005-noise-serial/files.mjs";
import { check, object, sha256, uint } from "./values.mjs";

/** These three values share the retained browser page's monotonic clock, never the host/device clock. */
export function validateStartTiming(timing) {
  object(timing, ["fixtureRequestAtPageMs", "startInvokedAtPageMs", "startRepliedAtPageMs"]);
  for (const value of Object.values(timing)) check(typeof value === "number" && Number.isFinite(value) && value >= 0, "v2_start_page_clock");
  check(timing.startInvokedAtPageMs >= timing.fixtureRequestAtPageMs && timing.startInvokedAtPageMs - timing.fixtureRequestAtPageMs <= 10000,
    "v2_start_fixture_deadline");
  check(timing.startRepliedAtPageMs >= timing.startInvokedAtPageMs && timing.startRepliedAtPageMs - timing.startInvokedAtPageMs <= 30000,
    "v2_start_reply_deadline");
}

/** Read-only timing/source joins; a running observation is not substituted for Start invocation. */
export async function inspectShareStart(root, context, states, devices) {
  const value = (await proof(root, "share-start-observed.json")).value;
  object(value, ["schema", "contextSha256", "clientSha256", "fixtureReadySha256", "consumedSha256", "atHostMs", "observedStateSequence",
    "bootOrdinal", "workerGeneration", "serialTransportEpoch", "networkObservedAtDeviceUs", "timing"]);
  for (const key of ["atHostMs", "observedStateSequence", "bootOrdinal", "workerGeneration", "serialTransportEpoch", "networkObservedAtDeviceUs"]) uint(value[key]);
  validateStartTiming(value.timing);
  const ready = await proof(root, "fixture-ready.json"), consumed = await proof(root, "consumed.json"), first = devices[0];
  const state = states[value.observedStateSequence - 1];
  check(context.scope === "share" && value.schema === "str005-v2-share-start-observed-v1" && value.contextSha256 === sha256(JSON.stringify(context)) &&
    value.clientSha256 === context.client_sha256 && value.fixtureReadySha256 === ready.sha256 && value.consumedSha256 === consumed.sha256 &&
    state?.phase === "candidate" && state.state.running && state.state.qualification?.generation === value.workerGeneration &&
    !state.state.heartbeatSuppressed && !state.state.failure && state.atHostMs <= value.atHostMs &&
    value.atHostMs >= consumed.value.atHostMs && value.atHostMs >= ready.value.readyAtMs && value.atHostMs <= first.atHostMs &&
    ["bootOrdinal", "workerGeneration", "serialTransportEpoch"].every(key => value[key] === first.record[key]) &&
    value.networkObservedAtDeviceUs <= first.record.admittedAtDeviceUs && first.record.admittedAtDeviceUs - value.networkObservedAtDeviceUs <= 5000000,
    "v2_share_start_source_join");
  return value;
}
