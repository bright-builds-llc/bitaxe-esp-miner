// Closed table of the detached owners these helpers may stop and finish, and the paths each one uses.
// Breadcrumbs: server-owner.json and finish inputs in scripts/str005-heartbeat-probe/finish.mjs,
// scripts/str005-share-probe/finish.mjs, scripts/str005-share-recovery/main.mjs and
// scripts/str005-step5-diagnostic/restart-main.mjs.
import { dirname, resolve } from "node:path";
import { refuse } from "./errors.mjs";

/**
 * A recovery proof and a restart claim must follow the recovery's collection-begin by at most 120 s:
 * FRESH_MS in scripts/str005-startup-preparation/model.mjs, recoveryProof in
 * scripts/str005-share-recovery/model.mjs and tools/flash/src/core_dump/proof.rs.
 */
export const RECOVERY_FRESH_MS = 120_000;

const STAGES = ["recovery", "restart"];
const family = (name, kind) => ({ name, kind, staged: kind === "restart", writesRecoveryProof: kind === "recovery" });

export const OWNERS = Object.freeze(Object.fromEntries([
  family("str005-heartbeat-shutdown", "heartbeat"),
  family("str005-heartbeat-probe", "heartbeat"),
  family("str005-accepted-share", "share"),
  family("str005-share-probe", "share"),
  family("str005-share-recovery", "recovery"),
  family("str005-control-diagnostic-recovery", "recovery"),
  family("str005-panic-recovery", "recovery"),
  family("str005-step5-restart", "restart"),
  family("str005-startup-preparation", "restart"),
  family("str005-heartbeat-preparation", "restart"),
  family("ultra205-soak", "soak"),
].map((owner) => [owner.name, Object.freeze(owner)])));

/** Resolve an owner name and stage into the exact files its serve and finish use. */
export function ownerLayout(name, root, maybeStage) {
  const owner = OWNERS[name];
  refuse(owner !== undefined, "owner_unknown");
  if (owner.staged) refuse(STAGES.includes(maybeStage), "owner_stage_required");
  else refuse(maybeStage === undefined, "owner_stage_unexpected");
  const parent = dirname(root), prefix = owner.staged ? `${maybeStage}-` : "";
  const stageRoot = owner.staged ? resolve(root, maybeStage) : root;
  return {
    owner, root, parent, stage: maybeStage, stageRoot,
    serverOwner: resolve(stageRoot, "server-owner.json"),
    // The restart recovery stage and the share-recovery collector both record their page collection here.
    collectionBegin: resolve(stageRoot, "collection-begin.json"),
    collectionFinished: resolve(stageRoot, "finished.json"),
    finalDetector: resolve(parent, `${prefix}final-detector.stdout.log`),
    finalDetectorStderr: resolve(parent, `${prefix}final-detector.stderr.log`),
    finishStdout: resolve(parent, `${prefix}finish.stdout.log`),
    finishStderr: resolve(parent, `${prefix}finish.stderr.log`),
    finishArguments: [name, "finish", "--private-root", root, ...(owner.staged ? ["--stage", maybeStage] : [])],
  };
}

/** Milliseconds left in the freshness window that starts at a recovery's collection-begin. */
export function windowRemainingMs(collectionStartedAtUnixMs, nowUnixMs) {
  refuse(Number.isSafeInteger(collectionStartedAtUnixMs) && collectionStartedAtUnixMs <= nowUnixMs, "collection_begin_invalid");
  return collectionStartedAtUnixMs + RECOVERY_FRESH_MS - nowUnixMs;
}
