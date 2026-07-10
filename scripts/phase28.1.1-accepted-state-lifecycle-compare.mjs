#!/usr/bin/env node

import fs from "node:fs";

import { parseAcceptedStateLog } from "./phase28.1.1-accepted-state-compare.mjs";

export const ACCEPTED_STATE_LIFECYCLE_STAGES = [
  "post_enumerate",
  "post_mining_ready",
  "post_max_baud",
  "post_mask_reload",
  "post_first_work",
];

function field(line, name) {
  return line.match(new RegExp(`(?:^|\\s)${name}=([^\\s]+)`))?.[1];
}

function markerFromLine(line) {
  const match = line.match(/(?:^|\s)(accepted_state_snapshot(?:\s|$).*)$/u);
  return match?.[1];
}

function snapshotsEqual(left, right) {
  return JSON.stringify(left) === JSON.stringify(right);
}

export function parseAcceptedStateLifecycleMember(text, memberName) {
  const snapshots = new Map();
  let markerCount = 0;

  for (const line of text.split(/\r?\n/u)) {
    const marker = markerFromLine(line);
    if (marker === undefined) continue;

    markerCount += 1;
    if (field(marker, "redacted") !== "true") {
      throw new Error(
        `accepted_state_lifecycle_error: ${memberName} marker is not redacted`,
      );
    }
    if (field(marker, "observation") === "unavailable") {
      throw new Error(
        `accepted_state_lifecycle_error: ${memberName} contains unavailable observation`,
      );
    }

    const parsed = parseAcceptedStateLog(marker);
    if (parsed.size !== 1) {
      throw new Error(
        `accepted_state_lifecycle_error: ${memberName} marker did not parse`,
      );
    }
    const [stage, snapshot] = parsed.entries().next().value;
    const maybeExisting = snapshots.get(stage);
    if (maybeExisting !== undefined && !snapshotsEqual(maybeExisting, snapshot)) {
      throw new Error(
        `accepted_state_lifecycle_error: ${memberName} has conflicting duplicate for ${stage}`,
      );
    }
    snapshots.set(stage, snapshot);
  }

  const actualStages = [...snapshots.keys()].sort();
  const expectedStages = [...ACCEPTED_STATE_LIFECYCLE_STAGES].sort();
  if (!snapshotsEqual(actualStages, expectedStages)) {
    throw new Error(
      `accepted_state_lifecycle_error: ${memberName} stage set is incomplete`,
    );
  }

  return {
    snapshots,
    markerCount,
    equivalentDuplicates: markerCount >= snapshots.size,
  };
}

export function compareAcceptedStateLifecycle(reinitText, coldStartText) {
  const reinit = parseAcceptedStateLifecycleMember(reinitText, "reinit");
  const coldStart = parseAcceptedStateLifecycleMember(
    coldStartText,
    "cold-start",
  );
  const stageStatus = Object.fromEntries(
    ACCEPTED_STATE_LIFECYCLE_STAGES.map((stage) => [
      `stage_${stage}`,
      snapshotsEqual(reinit.snapshots.get(stage), coldStart.snapshots.get(stage))
        ? "match"
        : "mismatch",
    ]),
  );
  const lifecycleStatus = Object.values(stageStatus).every(
    (status) => status === "match",
  )
    ? "match"
    : "mismatch";

  return {
    lifecycle_status: lifecycleStatus,
    reinit_stage_count: reinit.snapshots.size,
    cold_start_stage_count: coldStart.snapshots.size,
    reinit_marker_count: reinit.markerCount,
    cold_start_marker_count: coldStart.markerCount,
    reinit_equivalent_duplicates: reinit.equivalentDuplicates,
    cold_start_equivalent_duplicates: coldStart.equivalentDuplicates,
    ...stageStatus,
    redacted: true,
  };
}

export function unavailableAcceptedStateLifecycle() {
  return {
    lifecycle_status: "unavailable",
    reinit_stage_count: 0,
    cold_start_stage_count: 0,
    reinit_marker_count: 0,
    cold_start_marker_count: 0,
    reinit_equivalent_duplicates: false,
    cold_start_equivalent_duplicates: false,
    ...Object.fromEntries(
      ACCEPTED_STATE_LIFECYCLE_STAGES.map((stage) => [
        `stage_${stage}`,
        "unavailable",
      ]),
    ),
    redacted: true,
  };
}

export function renderAcceptedStateLifecycle(report) {
  return `${Object.entries(report)
    .map(([name, value]) => `${name}: ${String(value)}`)
    .join("\n")}\n`;
}

function parseArgs(argv) {
  const args = new Map();
  for (let index = 0; index < argv.length; index += 1) {
    const name = argv[index];
    if (name === "--unavailable") {
      args.set(name, true);
      continue;
    }
    const value = argv[index + 1];
    if (!name?.startsWith("--") || value === undefined) {
      throw new Error("accepted_state_lifecycle_error: invalid arguments");
    }
    args.set(name, value);
    index += 1;
  }
  if (!args.has("--out")) {
    throw new Error("accepted_state_lifecycle_error: missing --out");
  }
  if (args.has("--unavailable")) return args;
  for (const required of ["--reinit-log", "--cold-start-log"]) {
    if (!args.has(required)) {
      throw new Error(`accepted_state_lifecycle_error: missing ${required}`);
    }
  }
  return args;
}

if (import.meta.url === `file://${process.argv[1]}`) {
  try {
    const args = parseArgs(process.argv.slice(2));
    const report = args.has("--unavailable")
      ? unavailableAcceptedStateLifecycle()
      : compareAcceptedStateLifecycle(
          fs.readFileSync(args.get("--reinit-log"), "utf8"),
          fs.readFileSync(args.get("--cold-start-log"), "utf8"),
        );
    fs.writeFileSync(args.get("--out"), renderAcceptedStateLifecycle(report));
  } catch (error) {
    process.stderr.write(`${error.message}\n`);
    process.exitCode = 1;
  }
}
