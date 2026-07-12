#!/usr/bin/env node

import fs from "node:fs";
import { fileURLToPath } from "node:url";

import {
  parseHeartbeatStream,
  validateOrderedHeartbeats,
} from "./ultra205-late-attach-classifier.mjs";

const ACCEPTED_STAGES = [
  "post_enumerate",
  "post_mining_ready",
  "post_max_baud",
  "post_mask_reload",
  "post_first_work",
];

function streamStatus(text) {
  const stream = parseHeartbeatStream(text);
  const ordered = validateOrderedHeartbeats([stream]);
  return { stream, ordered };
}

export function validateUartConnectedPreflight(nativeText, uartText) {
  const native = streamStatus(nativeText);
  const uart = streamStatus(uartText);
  const sessions = new Set([
    ...native.stream.heartbeats,
    ...uart.stream.heartbeats,
  ].map(({ session }) => session));
  const passed =
    native.ordered.heartbeatCount > 0 &&
    uart.ordered.heartbeatCount > 0 &&
    !native.stream.malformed &&
    !uart.stream.malformed &&
    native.ordered.monotonic &&
    uart.ordered.monotonic &&
    native.ordered.cadenceValid &&
    uart.ordered.cadenceValid &&
    native.ordered.listenerArmed &&
    uart.ordered.listenerArmed &&
    sessions.size === 1;
  return {
    schema_version: "ultra205-uart-connected-preflight-v1",
    passed,
    native_heartbeat_count: native.ordered.heartbeatCount,
    uart_heartbeat_count: uart.ordered.heartbeatCount,
    same_session: sessions.size === 1,
    native_monotonic: native.ordered.monotonic,
    uart_monotonic: uart.ordered.monotonic,
    cadence_valid: native.ordered.cadenceValid && uart.ordered.cadenceValid,
    listener_ready: native.ordered.listenerArmed && uart.ordered.listenerArmed,
    session: sessions.size === 1 ? [...sessions][0] : null,
  };
}

function markerPayload(line, marker) {
  return line.match(new RegExp(`(?:^|\\s)(${marker}(?:\\s|$).*)$`, "u"))?.[1];
}

function parseBootEvidence(text) {
  const sessions = new Set();
  const states = new Set();
  let malformed = false;
  for (const line of text.split(/\r?\n/u)) {
    if (!line.includes("plan13_boot_evidence")) continue;
    const marker = markerPayload(line, "plan13_boot_evidence");
    const match = marker?.match(
      /^plan13_boot_evidence session=([0-9a-f]{32}) state=(booted|listener_armed) redacted=true$/u,
    );
    if (match === undefined || match === null) {
      malformed = true;
      continue;
    }
    sessions.add(match[1]);
    states.add(match[2]);
  }
  return { sessions, states, malformed };
}

function parseAcceptedStages(text) {
  const observed = new Set();
  let malformed = false;
  for (const line of text.split(/\r?\n/u)) {
    if (!line.includes("accepted_state_snapshot")) continue;
    const marker = markerPayload(line, "accepted_state_snapshot");
    const match = marker?.match(/^accepted_state_snapshot stage=([^\s]+)\s/u);
    if (match === undefined || match === null || !marker?.endsWith(" redacted=true")) {
      malformed = true;
      continue;
    }
    if (!ACCEPTED_STAGES.includes(match[1])) {
      malformed = true;
      continue;
    }
    observed.add(match[1]);
  }
  return { observed, malformed };
}

function countOriginal(text, marker) {
  return text.split(/\r?\n/u).filter((line) => line.includes(marker)).length;
}

function hasRuntimeHazard(text) {
  return /stack overflow|stack canary|guru meditation|panic(?:ed)?|abort\(\)|SW_CPU_RESET|RTC_SW_(?:SYS|CPU)_RST|software reset/iu.test(
    text,
  );
}

export function classifyColdUartStream(text, preflightSession) {
  const heartbeat = streamStatus(text);
  const evidence = parseBootEvidence(text);
  const stages = parseAcceptedStages(text);
  const bootCount = countOriginal(text, "bitaxe-rust boot");
  const listenerCount = countOriginal(text, "h4_continuous_result=listener_armed");
  const heartbeatSession = heartbeat.ordered.session;
  const evidenceSession = evidence.sessions.size === 1 ? [...evidence.sessions][0] : undefined;
  const sessionConsistent =
    heartbeatSession !== undefined &&
    evidenceSession !== undefined &&
    heartbeatSession === evidenceSession;
  const newSession = sessionConsistent && heartbeatSession !== preflightSession;
  const stagesComplete = ACCEPTED_STAGES.every((stage) => stages.observed.has(stage));
  const runtimeHazard = hasRuntimeHazard(text);
  const passed =
    bootCount === 1 &&
    listenerCount === 1 &&
    !heartbeat.stream.malformed &&
    heartbeat.ordered.heartbeatCount >= 3 &&
    heartbeat.ordered.sameSession &&
    heartbeat.ordered.monotonic &&
    heartbeat.ordered.cadenceValid &&
    heartbeat.ordered.listenerArmed &&
    !evidence.malformed &&
    evidence.sessions.size === 1 &&
    evidence.states.has("booted") &&
    evidence.states.has("listener_armed") &&
    !stages.malformed &&
    stagesComplete &&
    !runtimeHazard &&
    sessionConsistent &&
    newSession;
  return {
    schema_version: "ultra205-uart-cold-classification-v1",
    classification_category: passed ? "uart_cold_delivers" : "cold_uart_evidence_invalid",
    original_boot_count: bootCount,
    original_listener_count: listenerCount,
    heartbeat_count: heartbeat.ordered.heartbeatCount,
    heartbeat_monotonic: heartbeat.ordered.monotonic,
    heartbeat_cadence_valid: heartbeat.ordered.cadenceValid,
    listener_ready: heartbeat.ordered.listenerArmed,
    evidence_states_complete:
      evidence.states.has("booted") && evidence.states.has("listener_armed"),
    accepted_state_stage_count: stages.observed.size,
    accepted_state_stages_complete: stagesComplete,
    one_session: sessionConsistent,
    new_session: newSession,
    malformed:
      heartbeat.stream.malformed || evidence.malformed || stages.malformed,
    runtime_hazard: runtimeHazard,
    session: heartbeatSession ?? null,
  };
}

function main() {
  const command = process.argv[2];
  if (command === "preflight" && process.argv.length === 5) {
    const result = validateUartConnectedPreflight(
      fs.readFileSync(process.argv[3], "utf8"),
      fs.readFileSync(process.argv[4], "utf8"),
    );
    process.stdout.write(`${JSON.stringify(result)}\n`);
    process.exit(result.passed ? 0 : 1);
  }
  if (command === "cold" && process.argv.length === 5) {
    const result = classifyColdUartStream(
      fs.readFileSync(process.argv[3], "utf8"),
      process.argv[4],
    );
    process.stdout.write(`${JSON.stringify(result)}\n`);
    process.exit(result.classification_category === "uart_cold_delivers" ? 0 : 1);
  }
  throw new Error(
    "usage: ultra205-uart-capture-classifier.mjs preflight NATIVE UART | cold UART PREFLIGHT_SESSION",
  );
}

if (
  process.argv[1] !== undefined &&
  fs.realpathSync(fileURLToPath(import.meta.url)) === fs.realpathSync(process.argv[1])
) {
  main();
}
