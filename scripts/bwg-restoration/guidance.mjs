// Machine-readable `GET /supervisor-state`: the current scenario and step, whether a human checkpoint is armed,
// the safe state held, the local action that starts any finite effect, what to observe and which automated
// bounds apply (AGENTS.md, Asynchronous Human Checkpoints). Human waits never carry a deadline.
import { humanWait, view } from "./checkpoint.mjs";
import { EXPIRED_START_WAIT_MS, FRESH_POSSESSION_MS, MAXIMUM_REARMS, MAXIMUM_RENEWALS, MAXIMUM_STARTS, PHYSICAL_PLANS, REMOVAL_WINDOW_MS,
  STIMULUS_OBSERVATION_MS, SUPERVISOR_STATE_SCHEMA, WATCHER_PRESENT_TIMEOUT_MS } from "./contract.mjs";

const IDLE = "No lease is active; the device holds its paused baseline and the page holds no lease.";
const LEASED = "A signed 60 s lease is active; the device ends it on its own at the lease deadline or on transport loss.";
const PAGE_STEPS = {
  completion: "connect, prepareStart, loadScenarioLease, startScenarioLease, wait at least 20 s, renewOnce, restoreChallengeSatisfied, submitCompletion",
  pause: "connect, prepareStart, loadScenarioLease, startScenarioLease, pause, submitCompletion",
  cancel: "connect, prepareStart, loadScenarioLease, startScenarioLease, cancel, submitCompletion",
  expiry: "connect, prepareStart, loadScenarioLease, startScenarioLease, then poll statusReview until the device reports lease_expired, submitCompletion",
  monotonic_uncertainty: "connect, clockDiscontinuityStimulusReview, prepareStart, loadScenarioLease, startScenarioLease, triggerClockDiscontinuity, " +
    "then poll statusReview at most every 2 s until the device reports monotonic_reset, submitCompletion",
};
const LEG_STEPS = {
  n1: "connect, replayArtifact (the reboot Start); after the device rejection the page disconnects: connect, authorizationRejectionReview",
  n1_review: "connect, authorizationRejectionReview",
  n2_sign: "prepareStart (the server signs one Start for this possession and holds it)",
  n2_wait: `keep the page connected for ${EXPIRED_START_WAIT_MS} ms after signing, then replayArtifact; connect, authorizationRejectionReview`,
  n2_review: "connect, authorizationRejectionReview",
  n3_replay: `within ${FRESH_POSSESSION_MS} ms of the last review: replayArtifact; connect, authorizationRejectionReview`,
  n3_review: "connect, authorizationRejectionReview",
  n4_sign: "prepareStart, loadScenarioLease, startScenarioLease",
  n4_renew: "renewOnce",
  n4_replay: "replayArtifact (the accepted renewal); connect, authorizationRejectionReview",
  n4_review: "connect, authorizationRejectionReview",
  complete: "statusReview, submitCompletion",
};

function physicalStep(scenario, now) {
  const checkpoint = scenario.maybeCheckpoint, plan = PHYSICAL_PLANS[scenario.name];
  const { checkpoint: name } = view(checkpoint, now);
  const base = { checkpoint: name, human_checkpoint_armed: humanWait(checkpoint) || name === plan.removeCheckpoint, rearms: checkpoint.rearms };
  const steps = {
    awaiting_operator_ready: { safe_state: IDLE, local_action: "The owner confirms readiness; then POST /checkpoint/ready with this scenario and checkpoint.",
      observe: "Nothing yet; no instruction is live.", automated_bounds: [] },
    rearm_required: { safe_state: IDLE, local_action: "End any page lease (cancel) if one is active; the owner confirms readiness; POST /checkpoint/ready with checkpoint rearm_required.",
      observe: `The previous attempt expired (${checkpoint.maybeRearmReason}); it is expired authority, not device evidence.`, automated_bounds: [] },
    ready_for_lease: { safe_state: IDLE, local_action: "prepareStart, loadScenarioLease, startScenarioLease, statusReview, beginPhysicalWindow.",
      observe: "The lease starts; begin starts the presence watcher before any instruction.", automated_bounds: ["lease_60000_ms"] },
    watcher_starting: { safe_state: LEASED, local_action: "None; the watcher must prove the admitted device present.",
      observe: "Watcher presence.", automated_bounds: [`watcher_present_${WATCHER_PRESENT_TIMEOUT_MS}_ms`] },
    [plan.removeCheckpoint]: { safe_state: LEASED, local_action: "The owner performs the removal now.", instruction: plan.removal,
      observe: "The watcher reports the admitted device absent.", automated_bounds: [`removal_window_${REMOVAL_WINDOW_MS}_ms`, "lease_60000_ms"] },
    absence_bounding: { safe_state: "The device is unpowered or disconnected and holds no lease.", local_action: "Wait, then armPhysicalWindow.",
      observe: "Absence continues.", automated_bounds: [`minimum_absence_${plan.minimumAbsenceMs}_ms`] },
    absence_satisfied: { safe_state: "The device is unpowered or disconnected and holds no lease.", local_action: "armPhysicalWindow.",
      observe: "Absence bound met.", automated_bounds: [] },
    [plan.restoreCheckpoint]: { safe_state: "The device is unpowered or disconnected and holds no lease.", local_action: "The owner restores power or USB now.",
      instruction: plan.restore, observe: "The same physical device reappears with a new enumeration and stays stable.", automated_bounds: [],
      restore_watcher: { action_token: "bwg-restoration-restore-watcher-armed-v1", response_required: false } },
    stabilizing: { safe_state: "The device is reappearing and holds no lease.", local_action: "None.", observe: "Stability gate.", automated_bounds: ["stable_3000_ms"] },
    reconnect_ready: { safe_state: "The device is present and holds no lease.", local_action: "connect (Connect click), statusReview, submitCompletion.",
      observe: "The device reports the scenario's terminal reason.", automated_bounds: [] },
    reconnected: { safe_state: "The page is connected; the device holds no lease.", local_action: "statusReview, submitCompletion.", observe: "Terminal reason.", automated_bounds: [] },
    failed: { safe_state: IDLE, local_action: "None; the attempt is stopped.", observe: checkpoint.maybeFailure, automated_bounds: [] },
  };
  return { ...base, ...steps[name] };
}

function pageStep(scenario, now) {
  if (scenario.name === "authorization_negatives") {
    const held = scenario.maybeExpiredStart;
    return { checkpoint: scenario.leg, human_checkpoint_armed: false, safe_state: IDLE, local_action: LEG_STEPS[scenario.leg],
      observe: "The device rejects each replay; its rejection review attributes the leg.", automated_bounds: scenario.leg === "n3_replay" ? [`fresh_possession_${FRESH_POSSESSION_MS}_ms`] : [],
      ...(held && scenario.leg === "n2_wait" ? { replay_ready_in_ms: Math.max(0, held.signedAt + EXPIRED_START_WAIT_MS - now) } : {}) };
  }
  return { checkpoint: "page_operations", human_checkpoint_armed: false, safe_state: IDLE, local_action: PAGE_STEPS[scenario.name],
    observe: "The device reaches baseline with the scenario's terminal reason.",
    automated_bounds: scenario.name === "monotonic_uncertainty" ? [`stimulus_observation_${STIMULUS_OBSERVATION_MS}_ms`] : [] };
}

export function supervisorState(campaign, now) {
  const scenario = campaign.scenario;
  const step = campaign.complete || campaign.maybeFailure
    ? { checkpoint: campaign.complete ? "campaign_complete" : "campaign_stopped", human_checkpoint_armed: false, safe_state: IDLE,
      local_action: "Navigate the Gate tab to about:blank, stop serve, then finish.", observe: "Nothing.", automated_bounds: [] }
    : scenario.maybeCheckpoint ? physicalStep(scenario, now) : pageStep(scenario, now);
  return { schema: SUPERVISOR_STATE_SCHEMA, scenario: scenario.name, scenario_index: campaign.index, ...step,
    waiting_for_human_has_no_deadline: true,
    caps: { starts_signed: campaign.startsSigned, starts_cap: MAXIMUM_STARTS, renewals_signed: campaign.renewalsSigned, renewals_cap: MAXIMUM_RENEWALS,
      rearms_cap: MAXIMUM_REARMS },
    results: campaign.results.map((result) => ({ scenario: result.scenario, result: result.result })), failure: campaign.maybeFailure, complete: campaign.complete };
}
