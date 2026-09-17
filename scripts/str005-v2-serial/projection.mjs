import { check, digest, object, sha256, uint } from "./values.mjs";

const CRITERIA = ["identity", "continuity", "authority", "standardChannel", "targetAndJob", "accounting", "preservation", "restoration", "cleanup", "privacy"];
/** Public output contains no attempt nonce, hardware/job identifiers, socket data or private paths. */
export function projection(context, accepted, sealSha256, resultSha256) {
  check(accepted.scope === context.scope && accepted.contextSha256 === sha256(JSON.stringify(context)), "v2_projection_context");
  const value = {
    schema: "str005-v2-serial-projection-v1", scope: context.scope, status: "accepted", board: 205,
    hostOrdinal: context.hostOrdinal, sourceCommit: context.firmware_commit, gateCommit: context.gate_commit,
    provenance: { contract: context.contractSha256, contracts: sha256(JSON.stringify(context.contracts)),
      appElf: context.app_elf_sha256, packageManifest: context.manifest_sha256, evaluator: sha256(JSON.stringify(context.evaluator)),
      fixture: context.fixture_sha256, observer: context.cadence_observer.sha256, privateResult: resultSha256, sealedInventory: sealSha256,
      channelResult: context.scope === "share" ? context.predecessor.resultSha256 : null,
      channelSeal: context.scope === "share" ? context.predecessor.sealSha256 : null },
    criteria: Object.fromEntries([...CRITERIA, ...(context.scope === "share" ? ["asicNonce", "acceptedShare", "heartbeatShutdown", "cooling"] : ["noAsicWork", "noReservation"])].map(key => [key, true])),
    counts: { installations: accepted.continuity.installations, continuityCycles: 4, connections: accepted.protocol.counts.connections,
      deviceRecords: accepted.protocol.counts.deviceRecords, submitted: accepted.protocol.counts.submitted,
      deviceAcknowledged: accepted.protocol.counts.deviceAcknowledged, workDispatched: accepted.safety?.workDispatched ?? 0 },
    timings: { ...accepted.protocol.timings, hostCleanupMs: accepted.cleanupMs },
    nonClaims: ["external-pool-acceptance", "private-socket-tuple-reconstruction", "complete-native-callgraph-bound",
      ...(context.scope === "channel" ? ["asic-mining", "accepted-share", "funded-work"] : [])],
    redactionStatus: "passed",
  };
  return parseProjection(value);
}

export function parseProjection(value) {
  object(value, ["schema", "scope", "status", "board", "hostOrdinal", "sourceCommit", "gateCommit", "provenance", "criteria", "counts", "timings", "nonClaims", "redactionStatus"]);
  check(value.schema === "str005-v2-serial-projection-v1" && ["channel", "share"].includes(value.scope) && value.status === "accepted" &&
    value.board === 205 && value.redactionStatus === "passed" && /^[a-f0-9]{40}$/u.test(value.sourceCommit) && /^[a-f0-9]{40}$/u.test(value.gateCommit), "v2_projection_schema");
  uint(value.hostOrdinal); check(value.hostOrdinal > 0, "v2_projection_ordinal");
  object(value.provenance, ["contract", "contracts", "appElf", "packageManifest", "evaluator", "fixture", "observer", "privateResult", "sealedInventory", "channelResult", "channelSeal"]);
  for (const [key, item] of Object.entries(value.provenance)) {
    if (["channelResult", "channelSeal"].includes(key) && value.scope === "channel") check(item === null, "v2_projection_lineage");
    else digest(item);
  }
  object(value.criteria, [...CRITERIA, ...(value.scope === "share" ? ["asicNonce", "acceptedShare", "heartbeatShutdown", "cooling"] : ["noAsicWork", "noReservation"])]);
  check(Object.values(value.criteria).every(item => item === true), "v2_projection_criteria");
  object(value.counts, ["installations", "continuityCycles", "connections", "deviceRecords", "submitted", "deviceAcknowledged", "workDispatched"]);
  for (const count of Object.values(value.counts)) uint(count);
  check(value.counts.installations === (value.scope === "channel" ? 5 : 4) && value.counts.continuityCycles === 4 && value.counts.connections === 1,
    "v2_projection_counts");
  if (value.scope === "channel") check(value.counts.submitted === 0 && value.counts.deviceAcknowledged === 0 && value.counts.workDispatched === 0, "v2_projection_channel_work");
  else check(value.counts.submitted > 0 && value.counts.deviceAcknowledged > 0 && value.counts.workDispatched > 0, "v2_projection_share_missing");
  object(value.timings, ["preparationUs", "maximumReadUs", "maximumWriteUs", "connectUs", "resourceReleaseUs", "hostCleanupMs",
    ...(value.scope === "share" ? ["heartbeatToRevocationUs", "heartbeatToShutdownUs", "observerTailMs"] : [])]);
  for (const time of Object.values(value.timings)) uint(time);
  check(value.timings.hostCleanupMs <= 5000, "v2_projection_cleanup");
  check(JSON.stringify(value.nonClaims) === JSON.stringify(["external-pool-acceptance", "private-socket-tuple-reconstruction", "complete-native-callgraph-bound",
    ...(value.scope === "channel" ? ["asic-mining", "accepted-share", "funded-work"] : [])]), "v2_projection_nonclaims");
  return structuredClone(value);
}
