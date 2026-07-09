#!/usr/bin/env node

import fs from "node:fs";

const STAGES = [
  "post_enumerate",
  "post_mining_ready",
  "post_max_baud",
  "post_mask_reload",
  "post_first_work",
];

const STATUSES = new Set(["match", "mismatch", "unavailable"]);
const POWER_CLASSES = new Set([
  "falling",
  "flat",
  "rising_hashing",
  "unavailable",
]);
const RECOMMENDATIONS = new Set([
  "accepted_state_transition_divergence",
  "cold_boot_recovery_lifecycle_parity",
  "upstream_init_transcript_prefix_bisection",
  "none",
]);
const BANNED_RECOMMENDATIONS = new Set([
  "post_max_baud_delay_2000",
  "match_upstream_register_read_poll",
  "upstream_like_long_block_receive",
  "ticket_mask_asic_difficulty",
  "count_asic_chips_rx_loop_parity",
  "negotiated_version_mask_work_field_parity",
  "pool_negotiated_mask_asic_reload",
]);

function field(line, name) {
  return line.match(new RegExp(`(?:^|\\s)${name}=([^\\s]+)`))?.[1];
}

function requiredField(line, name) {
  const value = field(line, name);
  if (value === undefined) {
    throw new Error(`accepted_state_compare_error: missing ${name}`);
  }
  return value;
}

function parseBoolean(value, name) {
  if (value === "true") return true;
  if (value === "false") return false;
  throw new Error(`accepted_state_compare_error: invalid ${name}`);
}

function parseCount(value) {
  if (!/^\d+$/.test(value)) {
    throw new Error("accepted_state_compare_error: invalid readable_responses");
  }
  return Number.parseInt(value, 10);
}

export function parseAcceptedStateLog(text) {
  const snapshots = new Map();

  for (const line of text.split(/\r?\n/u)) {
    if (!/(?:^|\s)accepted_state_snapshot(?:\s|$)/u.test(line)) continue;

    const stage = requiredField(line, "stage");
    if (!STAGES.includes(stage)) {
      throw new Error("accepted_state_compare_error: unknown stage");
    }

    const observation = requiredField(line, "observation");
    if (observation !== "available" && observation !== "unavailable") {
      throw new Error("accepted_state_compare_error: unknown observation");
    }

    const chipCountClass = requiredField(line, "chip_count_class");
    if (!STATUSES.has(chipCountClass)) {
      throw new Error("accepted_state_compare_error: unknown chip_count_class");
    }

    const powerDeltaClass = requiredField(line, "power_delta_class");
    if (!POWER_CLASSES.has(powerDeltaClass)) {
      throw new Error("accepted_state_compare_error: unknown power_delta_class");
    }

    snapshots.set(stage, {
      stage,
      observation,
      chipCountClass,
      readableResponses: parseCount(requiredField(line, "readable_responses")),
      errorCounterActive: parseBoolean(
        requiredField(line, "error_counter_active"),
        "error_counter_active",
      ),
      domainCounterActive: parseBoolean(
        requiredField(line, "domain_counter_active"),
        "domain_counter_active",
      ),
      totalCounterActive: parseBoolean(
        requiredField(line, "total_counter_active"),
        "total_counter_active",
      ),
      powerDeltaClass,
      resultCorrelated: parseBoolean(
        requiredField(line, "result_correlated"),
        "result_correlated",
      ),
      submitObserved: parseBoolean(
        requiredField(line, "submit_observed"),
        "submit_observed",
      ),
    });
  }

  return snapshots;
}

function counterActive(snapshot) {
  return (
    snapshot?.errorCounterActive === true ||
    snapshot?.domainCounterActive === true ||
    snapshot?.totalCounterActive === true
  );
}

function observationAvailable(snapshot) {
  return (
    snapshot?.observation === "available" &&
    snapshot.readableResponses > 0 &&
    snapshot.chipCountClass !== "unavailable"
  );
}

function valuesDiffer(upstream, rust) {
  return (
    upstream.chipCountClass !== rust.chipCountClass ||
    upstream.readableResponses !== rust.readableResponses ||
    upstream.errorCounterActive !== rust.errorCounterActive ||
    upstream.domainCounterActive !== rust.domainCounterActive ||
    upstream.totalCounterActive !== rust.totalCounterActive ||
    upstream.powerDeltaClass !== rust.powerDeltaClass
  );
}

export function compareAcceptedState(upstreamText, rustText) {
  const upstream = parseAcceptedStateLog(upstreamText);
  const rust = parseAcceptedStateLog(rustText);
  const resultProgress = [...rust.values()].some(
    (snapshot) => snapshot.resultCorrelated || snapshot.submitObserved,
  );

  let firstDivergentStage = "none";
  let missingObservation = false;
  let counterDivergence = false;
  let otherMismatch = false;

  for (const stage of STAGES) {
    const upstreamSnapshot = upstream.get(stage);
    const rustSnapshot = rust.get(stage);
    if (upstreamSnapshot === undefined && rustSnapshot === undefined) continue;

    if (
      !observationAvailable(upstreamSnapshot) ||
      !observationAvailable(rustSnapshot)
    ) {
      missingObservation = true;
      if (firstDivergentStage === "none") firstDivergentStage = stage;
      continue;
    }

    if (counterActive(upstreamSnapshot) && !counterActive(rustSnapshot)) {
      counterDivergence = true;
      if (firstDivergentStage === "none") firstDivergentStage = stage;
    } else if (valuesDiffer(upstreamSnapshot, rustSnapshot)) {
      otherMismatch = true;
      if (firstDivergentStage === "none") firstDivergentStage = stage;
    }
  }

  const acceptedStateStatus =
    counterDivergence || otherMismatch
      ? "mismatch"
      : missingObservation
        ? "unavailable"
        : "match";
  const recommendedInvestigation = resultProgress
    ? "none"
    : counterDivergence
      ? "accepted_state_transition_divergence"
      : missingObservation
        ? "cold_boot_recovery_lifecycle_parity"
        : otherMismatch
          ? "upstream_init_transcript_prefix_bisection"
          : "cold_boot_recovery_lifecycle_parity";

  if (
    !RECOMMENDATIONS.has(recommendedInvestigation) ||
    BANNED_RECOMMENDATIONS.has(recommendedInvestigation)
  ) {
    throw new Error("accepted_state_compare_error: invalid recommendation");
  }

  return {
    accepted_state_status: acceptedStateStatus,
    first_divergent_stage: firstDivergentStage,
    chip_count_class:
      rust.get(firstDivergentStage)?.chipCountClass ??
      [...rust.values()][0]?.chipCountClass ??
      "unavailable",
    hash_counter_activity_upstream: [...upstream.values()].some(counterActive)
      ? "active"
      : upstream.size > 0
        ? "inactive"
        : "unavailable",
    hash_counter_activity_rust: [...rust.values()].some(counterActive)
      ? "active"
      : rust.size > 0
        ? "inactive"
        : "unavailable",
    power_delta_class:
      [...rust.values()].at(-1)?.powerDeltaClass ?? "unavailable",
    result_correlated: [...rust.values()].some(
      (snapshot) => snapshot.resultCorrelated,
    ),
    fake_pool_submit_observed: [...rust.values()].some(
      (snapshot) => snapshot.submitObserved,
    ),
    recommended_investigation: recommendedInvestigation,
    redacted: true,
  };
}

export function renderAcceptedStateReport(report) {
  return `${Object.entries(report)
    .map(([name, value]) => `${name}: ${String(value)}`)
    .join("\n")}\n`;
}

function parseArgs(argv) {
  const args = new Map();
  for (let index = 0; index < argv.length; index += 2) {
    const name = argv[index];
    const value = argv[index + 1];
    if (!name?.startsWith("--") || value === undefined) {
      throw new Error("accepted_state_compare_error: invalid arguments");
    }
    args.set(name, value);
  }
  for (const required of ["--upstream-log", "--rust-log", "--out"]) {
    if (!args.has(required)) {
      throw new Error(`accepted_state_compare_error: missing ${required}`);
    }
  }
  return args;
}

if (import.meta.url === `file://${process.argv[1]}`) {
  try {
    const args = parseArgs(process.argv.slice(2));
    const report = compareAcceptedState(
      fs.readFileSync(args.get("--upstream-log"), "utf8"),
      fs.readFileSync(args.get("--rust-log"), "utf8"),
    );
    fs.writeFileSync(args.get("--out"), renderAcceptedStateReport(report));
  } catch (error) {
    process.stderr.write(`${error.message}\n`);
    process.exitCode = 1;
  }
}
