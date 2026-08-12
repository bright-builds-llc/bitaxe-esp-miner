import { createHash } from "node:crypto";
import { chmod, mkdir, readFile, rename, stat, unlink, writeFile } from "node:fs/promises";
import path from "node:path";

import {
  internalCommandSpec,
  type AsicFrequencyTransitionEvidence,
  type AsicInitializationEvidence,
  type AutomationCategory,
} from "./contracts.generated.js";
import type { ProcessPort } from "./process.js";
import { assertWithinWorkspace } from "./workspace.js";

export type AsicFrequencyTransitionEvidenceOptions = {
  readonly sourceProjection: string;
  readonly attemptSourceCommit: string;
  readonly projection: string;
};

type FailureCategory = Extract<AutomationCategory, "evidence_invalid" | "process_failed">;

const expectedSourceProjection =
  "docs/parity/evidence/asic002-initialization/asic-initialization-projection.json";
const expectedSourceProjectionSha256 =
  "eee750561a7c1dcec1a5698b1e5827d3f1508d43655c3c4aa237097338dcf8d4";
const productionPath = "firmware/bitaxe/src/asic_adapter/production.rs";
const rampPaths = [
  "crates/bitaxe-asic/src/bm1366/frequency_voltage.rs",
  "crates/bitaxe-asic/src/bm1366/mining_ready.rs",
  "firmware/bitaxe/src/mining_actuation.rs",
  "firmware/bitaxe/src/mining_actuation_adapter.rs",
  "firmware/bitaxe/src/asic_adapter.rs",
  "firmware/bitaxe/src/asic_adapter/uart.rs",
] as const;
const semanticFragments = new Map<string, readonly string[]>([
  ["crates/bitaxe-asic/src/bm1366/mining_ready.rs", [
    "const FREQ_RAMP_START_QUARTER_MHZ: u32 = 50 * QUARTERS_PER_MHZ;",
    "const FREQ_RAMP_STEP_QUARTER_MHZ: u32 = 25;",
    "const FREQ_RAMP_DELAY_MS: u32 = 100;",
    "commands.push(Bm1366Command::DelayMs(FREQ_RAMP_DELAY_MS));",
    "pub const fn production_with_frequency_ramp() -> Self {",
  ]],
  ["firmware/bitaxe/src/mining_actuation.rs", [
    "PreparationStep::InitializeMiningReadyWithFrequencyRamp(profile.frequency()),",
  ]],
  ["firmware/bitaxe/src/mining_actuation_adapter.rs", [
    "MiningReadyInitOptions::production_with_frequency_ramp(),",
    "crate::asic_adapter::production::execute_mining_ready_actions(decision.actions())",
  ]],
]);

export class AsicFrequencyTransitionEvidenceError extends Error {
  public constructor(
    public readonly category: FailureCategory,
    message: string,
    public readonly publicValue: Readonly<Record<string, unknown>>,
  ) {
    super(message);
    this.name = "AsicFrequencyTransitionEvidenceError";
  }
}

function failure(category: FailureCategory, message: string): AsicFrequencyTransitionEvidenceError {
  return new AsicFrequencyTransitionEvidenceError(category, message, {
    stage: "sealed_frequency_transition_projection",
    hardware_rerun_used: false,
    projection_published: false,
  });
}

function sha256(value: string | Buffer): string {
  return createHash("sha256").update(value).digest("hex");
}

function lowerHex(value: string, length: number): boolean {
  return value.length === length && new RegExp(`^[0-9a-f]{${String(length)}}$`, "u").test(value);
}

async function childText(
  processPort: ProcessPort,
  program: string,
  args: readonly string[],
  context: string,
): Promise<string> {
  try {
    const outcome = await processPort.run(internalCommandSpec(program, [...args], (value) => value));
    if (outcome.timedOut || outcome.exitCode !== 0) {
      throw failure("evidence_invalid", `${context} did not pass`);
    }
    return outcome.stdout.trim();
  } catch (error) {
    if (error instanceof AsicFrequencyTransitionEvidenceError) throw error;
    throw failure("process_failed", `${context} launch failed`);
  }
}

async function requireAbsent(candidate: string, context: string): Promise<void> {
  try {
    await stat(candidate);
    throw failure("evidence_invalid", `${context} must be absent before projection`);
  } catch (error) {
    if (error instanceof AsicFrequencyTransitionEvidenceError) throw error;
    if ((error as NodeJS.ErrnoException).code !== "ENOENT") throw error;
  }
}

function parseSource(document: string): AsicInitializationEvidence {
  let value: unknown;
  try {
    value = JSON.parse(document);
  } catch {
    throw failure("evidence_invalid", "initialization source projection is malformed");
  }
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw failure("evidence_invalid", "initialization source projection must be an object");
  }
  return value as AsicInitializationEvidence;
}

function validateSourceFacts(source: AsicInitializationEvidence, attemptSourceCommit: string): void {
  const initialization = source.initialization;
  if (source.schema_version !== "bitaxe-asic-initialization-evidence-v1"
    || source.board !== 205
    || source.attempt_source_commit !== attemptSourceCommit
    || initialization.planned_step_count !== 9
    || initialization.accepted_preparation_event_count !== 18
    || initialization.invalid_preparation_event_count !== 0
    || initialization.terminal_preparation_step !== "retain_production_uart"
    || initialization.terminal_preparation_outcome !== "completed"
    || !initialization.all_preparation_steps_completed
    || !initialization.exactly_one_chip_detected
    || !initialization.mining_ready_initialization_completed
    || !initialization.production_uart_retained
    || !initialization.live_initialized_work_observed
    || !source.package_admitted
    || source.runtime_identity !== "trusted"
    || source.runtime_attestation_status !== "trusted"
    || source.campaign_terminal_category !== "submit_response_observed"
    || source.submit_outcome !== "accepted"
    || source.safety_status !== "fresh"
    || !source.mine_on_boot_disabled
    || !source.safe_stop_confirmed
    || !source.lease_cleanup_confirmed
    || !source.usb_cleanup_ready
    || source.hardware_rerun_used
    || source.redaction_status !== "passed") {
    throw failure("evidence_invalid", "frequency-transition source quorum is incomplete");
  }
}

function extractUniqueSpan(document: string, start: string, end: string): string {
  const startIndex = document.indexOf(start);
  if (startIndex === -1 || document.indexOf(start, startIndex + start.length) !== -1) {
    throw failure("evidence_invalid", "frequency-transition span start is not unique");
  }
  const endIndex = document.indexOf(end, startIndex + start.length);
  if (endIndex === -1 || document.indexOf(end, endIndex + end.length) !== -1) {
    throw failure("evidence_invalid", "frequency-transition span end is not unique");
  }
  return document.slice(startIndex, endIndex);
}

function requireUniqueFragment(document: string, fragment: string): void {
  const first = document.indexOf(fragment);
  if (first === -1 || document.indexOf(fragment, first + fragment.length) !== -1) {
    throw failure("evidence_invalid", "frequency-transition semantic fragment is not unique");
  }
}

async function validateSourceCompatibility(
  processPort: ProcessPort,
  gitProgram: string,
  attemptSourceCommit: string,
  currentSourceCommit: string,
): Promise<void> {
  for (const sourcePath of rampPaths) {
    await childText(processPort, gitProgram,
      ["diff", "--quiet", attemptSourceCommit, currentSourceCommit, "--", sourcePath],
      "frequency-transition module compatibility");
    const document = await childText(processPort, gitProgram,
      ["show", `${currentSourceCommit}:${sourcePath}`], "frequency-transition source admission");
    for (const fragment of semanticFragments.get(sourcePath) ?? []) {
      requireUniqueFragment(document, fragment);
    }
  }

  const [attemptProduction, currentProduction] = await Promise.all([
    childText(processPort, gitProgram, ["show", `${attemptSourceCommit}:${productionPath}`],
      "frequency-transition executor admission"),
    childText(processPort, gitProgram, ["show", `${currentSourceCommit}:${productionPath}`],
      "frequency-transition executor admission"),
  ]);
  const spans = [
    ["pub fn execute_mining_ready_actions(", "/// Executes the typed frequency-down"],
    ["fn execute_adapter_actions_on_state(", "#[cfg(test)]"],
  ] as const;
  for (const [start, end] of spans) {
    if (extractUniqueSpan(attemptProduction, start, end)
      !== extractUniqueSpan(currentProduction, start, end)) {
      throw failure("evidence_invalid", "frequency-transition executor span drifted");
    }
  }
}

export async function projectAsicFrequencyTransitionEvidence(
  workspaceRoot: string,
  options: AsicFrequencyTransitionEvidenceOptions,
  processPort: ProcessPort,
  gitProgram: string,
  sourceValidatorProgram: string,
  validatorProgram: string,
  admittedSourceSha256 = expectedSourceProjectionSha256,
): Promise<AsicFrequencyTransitionEvidence> {
  if (!lowerHex(options.attemptSourceCommit, 40)) {
    throw failure("evidence_invalid", "attempt source commit is invalid");
  }
  const sourceProjection = assertWithinWorkspace(workspaceRoot, options.sourceProjection);
  const projection = assertWithinWorkspace(workspaceRoot, options.projection);
  const candidate = `${projection}.candidate`;
  if (path.relative(workspaceRoot, sourceProjection) !== expectedSourceProjection) {
    throw failure("evidence_invalid", "initialization source projection path is invalid");
  }
  await requireAbsent(projection, "public projection");
  await requireAbsent(candidate, "projection candidate");

  const sourceDocument = await readFile(sourceProjection, "utf8");
  const sourceProjectionSha256 = sha256(sourceDocument);
  if (sourceProjectionSha256 !== admittedSourceSha256) {
    throw failure("evidence_invalid", "initialization source projection digest is invalid");
  }
  const source = parseSource(sourceDocument);
  await childText(processPort, sourceValidatorProgram, [sourceProjection],
    "initialization source validation");
  validateSourceFacts(source, options.attemptSourceCommit);

  const [currentSourceCommit, referenceCommit] = await Promise.all([
    childText(processPort, gitProgram, ["rev-parse", "HEAD"], "current source identity"),
    childText(processPort, gitProgram,
      ["-C", path.join(workspaceRoot, "reference/esp-miner"), "rev-parse", "HEAD"],
      "reference source identity"),
  ]);
  if (!lowerHex(currentSourceCommit, 40) || !lowerHex(referenceCommit, 40)) {
    throw failure("evidence_invalid", "current source identity is invalid");
  }
  if (source.reference_commit !== referenceCommit) {
    throw failure("evidence_invalid", "reference source identity drifted");
  }
  await childText(processPort, gitProgram,
    ["cat-file", "-e", `${options.attemptSourceCommit}^{commit}`], "attempt source admission");
  await childText(processPort, gitProgram,
    ["merge-base", "--is-ancestor", options.attemptSourceCommit, currentSourceCommit],
    "attempt source ancestry");
  await validateSourceCompatibility(
    processPort, gitProgram, options.attemptSourceCommit, currentSourceCommit,
  );
  const relevantPaths = [...rampPaths, productionPath, expectedSourceProjection];
  const dirty = await childText(processPort, gitProgram,
    ["status", "--porcelain", "--", ...relevantPaths], "frequency-transition worktree state");
  if (dirty !== "") throw failure("evidence_invalid", "frequency-transition paths have uncommitted drift");

  const requestSha256 = sha256(JSON.stringify({
    command: "project-asic-frequency-transition-evidence",
    source_projection: expectedSourceProjection,
    attempt_source_commit: options.attemptSourceCommit,
    current_source_commit: currentSourceCommit,
    projection: path.relative(workspaceRoot, projection),
  }));
  const evidence: AsicFrequencyTransitionEvidence = {
    schema_version: "bitaxe-asic-frequency-transition-evidence-v1",
    board: 205,
    attempt_source_commit: options.attemptSourceCommit,
    current_source_commit: currentSourceCommit,
    reference_commit: referenceCommit,
    workflow: {
      schema_version: "bitaxe-workflow-identity-v1",
      command: "project-asic-frequency-transition-evidence",
      request_sha256: requestSha256,
    },
    source: {
      initialization_projection_sha256: sourceProjectionSha256,
      initialization_projection_current_commit: source.current_source_commit,
      initialization_projection_valid: true,
    },
    frequency_transition: {
      profile: "conservative",
      start_frequency_mhz: 50,
      target_frequency_mhz: 400,
      step_quarter_mhz: 25,
      set_frequency_command_count: 56,
      inter_step_delay_count: 56,
      inter_step_delay_ms: 100,
      increasing: true,
      production_ramp_option_enabled: true,
      all_frequency_actions_completed: true,
      live_initialized_work_observed: true,
      accepted_submit_observed: true,
      ramp_modules_unchanged: true,
      executor_span_compatible: true,
    },
    package_admitted: true,
    runtime_identity: "trusted",
    runtime_attestation_status: "trusted",
    campaign_terminal_category: "submit_response_observed",
    submit_outcome: "accepted",
    safety_status: "fresh",
    mine_on_boot_disabled: true,
    safe_stop_confirmed: true,
    lease_cleanup_confirmed: true,
    usb_cleanup_ready: true,
    hardware_rerun_used: false,
    redaction_status: "passed",
  };

  await mkdir(path.dirname(projection), { recursive: true });
  try {
    await writeFile(candidate, `${JSON.stringify(evidence, null, 2)}\n`,
      { encoding: "utf8", flag: "wx", mode: 0o600 });
    await chmod(candidate, 0o600);
    await childText(processPort, validatorProgram, [candidate], "independent evidence validation");
    await rename(candidate, projection);
    await chmod(projection, 0o644);
  } catch (error) {
    await unlink(candidate).catch(() => undefined);
    throw error;
  }
  return evidence;
}
