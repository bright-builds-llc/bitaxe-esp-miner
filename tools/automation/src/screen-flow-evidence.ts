import { createHash } from "node:crypto";
import { chmod, mkdir, readFile, rename, stat, unlink, writeFile } from "node:fs/promises";
import path from "node:path";

import {
  internalCommandSpec,
  type AutomationCategory,
  type ScreenFlowEvidence,
} from "./contracts.generated.js";
import type { ProcessPort } from "./process.js";
import { assertWithinWorkspace } from "./workspace.js";

export type ScreenFlowEvidenceOptions = {
  readonly sourceDisplayUat: string;
  readonly sourceCommandEffects: string;
  readonly attemptSourceCommit: string;
  readonly projection: string;
};

type FailureCategory = Extract<AutomationCategory, "evidence_invalid" | "process_failed">;

const expectedDisplayUat =
  "docs/parity/evidence/api009-command-effects/display-uat-projection-attempt-005.json";
const expectedDisplayUatSha256 =
  "a863fc0034f105c85ae3007cd45a532035bfd6e061dbbf1a915282a5cfa3314f";
const expectedCommandEffects =
  "docs/parity/evidence/api009-command-effects/command-effects-projection-attempt-046.json";
const expectedCommandEffectsSha256 =
  "216420e0a9d93cbbacced7415be0a234ed13c0d895dcb20eb1ff295ff434a8a3";
const expectedPlan = "docs/parity/work-plans/20260816T073911Z-UI-002/PLAN.md";
const expectedPlanSha256 =
  "b1317b895b5c39208713c150089a68e276d0264be8bdd60c4da4f43f512ddecb";
const expectedProjection =
  "docs/parity/evidence/ui002-screen-flow/screen-flow-projection.json";
const activeTask = "task-parity-ui002-screen-flow";

const unchangedPaths = [
  "crates/bitaxe-core/src/screen.rs",
  "crates/bitaxe-core/src/screen/frame.rs",
  "firmware/bitaxe/src/runtime_snapshot/screen.rs",
  "firmware/bitaxe/src/display_adapter.rs",
] as const;
const semanticPath = "firmware/bitaxe/src/operator_sensor_runtime.rs";

const sourceFragments = new Map<string, readonly string[]>([
  [unchangedPaths[0], [
    "pub const SCREEN_UPDATE_MS: u64 = 500;",
    "pub const INTRO_DELAY_MS: u64 = 3_000;",
    "pub const CAROUSEL_DELAY_MS: u64 = 10_000;",
    "} else if snapshot.show_new_block {",
    "frame([\"\", \"BITAXE IDENTIFY\", \"Hello!\", \"\"])",
    "fn notification(&mut self, snapshot: &ScreenSnapshot) -> Notification {",
  ]],
  [unchangedPaths[1], [
    "pub fn private_lines(&self) -> [&str; SCREEN_LINE_COUNT] {",
    "pub fn fits_ultra205(&self) -> bool {",
    "fn clean_line(value: &str) -> String {",
    "fn frame_with_notification(",
  ]],
  [unchangedPaths[2], [
    "pub fn collect_screen_snapshot(now_ms: u64) -> ScreenSnapshot {",
    "let command = screen_command_projection(now_ms);",
    "let pool_host = screen_pool_host(command.fallback_active);",
  ]],
  [unchangedPaths[3], [
    "pub fn render_runtime_screen(",
    "frame.private_lines(),",
    "self.power_policy",
  ]],
  [semanticPath, [
    "struct RuntimeDisplay {",
    "if display.maybe_last_frame.as_ref() == Some(&decision.frame) {",
    ".service_power(owner, &mut i2c_budget, uptime_ms, decision.priority_visible)",
    ".render_runtime_screen(owner, &mut i2c_budget, &decision.frame)",
    "disable_runtime_display(maybe_display, \"render_failed\", &error);",
  ]],
]);

const referenceFragments = [
  "#define SCREEN_UPDATE_MS 500",
  "static int delays_ms[MAX_SCREENS] = {0, 0, 0, 0, 0, 1000, 3000, 3000, 10000, 10000, 10000, 10000};",
  "static const char *notifications[] = {",
  "static bool screen_show(screen_t screen)",
  "static void screen_update_cb(lv_timer_t * timer)",
  "if (get_current_screen() != SCR_STATS) {",
  "lv_timer_create(screen_update_cb, SCREEN_UPDATE_MS, NULL);",
] as const;

export class ScreenFlowEvidenceError extends Error {
  public constructor(
    public readonly category: FailureCategory,
    message: string,
    public readonly publicValue: Readonly<Record<string, unknown>>,
  ) {
    super(message);
    this.name = "ScreenFlowEvidenceError";
  }
}

function failure(category: FailureCategory, message: string): ScreenFlowEvidenceError {
  return new ScreenFlowEvidenceError(category, message, {
    stage: "sealed_screen_flow_projection",
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
    if (error instanceof ScreenFlowEvidenceError) throw error;
    throw failure("process_failed", `${context} launch failed`);
  }
}

async function requireAbsent(candidate: string, context: string): Promise<void> {
  try {
    await stat(candidate);
    throw failure("evidence_invalid", `${context} must be absent before projection`);
  } catch (error) {
    if (error instanceof ScreenFlowEvidenceError) throw error;
    if ((error as NodeJS.ErrnoException).code !== "ENOENT") throw error;
  }
}

function parseObject(document: string, context: string): Record<string, unknown> {
  let value: unknown;
  try {
    value = JSON.parse(document);
  } catch {
    throw failure("evidence_invalid", `${context} is malformed`);
  }
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw failure("evidence_invalid", `${context} must be an object`);
  }
  return value as Record<string, unknown>;
}

function validateDisplayUat(uat: Record<string, unknown>, commandEffectsSha256: string): void {
  if (uat["schema_version"] !== "bitaxe-display-uat-evidence-v1"
    || uat["board"] !== 205
    || uat["identify_request_count"] !== 1
    || uat["machine_render_confirmed"] !== true
    || uat["machine_clear_confirmed"] !== true
    || uat["operator_render_confirmed"] !== true
    || uat["operator_clear_confirmed"] !== true
    || uat["build_identity_matches"] !== true
    || uat["usb_admission_confirmed"] !== true
    || uat["programmatic_evidence_sha256"] !== commandEffectsSha256
    || uat["redaction_status"] !== "passed") {
    throw failure("evidence_invalid", "screen-flow display UAT quorum is incomplete");
  }
}

function validateCommandEffects(
  command: Record<string, unknown>,
  attemptSourceCommit: string,
  referenceCommit: string,
): void {
  const maybeEffects = command["command_effects"];
  if (typeof maybeEffects !== "object" || maybeEffects === null || Array.isArray(maybeEffects)) {
    throw failure("evidence_invalid", "screen-flow command quorum is incomplete");
  }
  const effects = maybeEffects as Record<string, unknown>;
  if (command["schema_version"] !== "bitaxe-api-command-effects-evidence-v1"
    || command["board"] !== 205
    || command["source_commit"] !== attemptSourceCommit
    || command["reference_commit"] !== referenceCommit
    || effects["identify_status_baseline_confirmed"] !== true
    || effects["identify_request_count"] !== 1
    || effects["identify_render_receipt_confirmed"] !== true
    || effects["identify_clear_receipt_confirmed"] !== true
    || effects["retained_identify_transition_confirmed"] !== true
    || effects["serial_transition_witnesses_confirmed"] !== true
    || effects["identify_terminal_outcome"] !== "none"
    || effects["same_boot_and_package"] !== true
    || command["safe_stop_confirmed"] !== true
    || command["cleanup_complete"] !== true
    || command["recovery_attempted"] !== false
    || command["mining_state"] !== "disabled"
    || command["hardware_control_state"] !== "disabled"
    || command["redaction_status"] !== "passed") {
    throw failure("evidence_invalid", "screen-flow command quorum is incomplete");
  }
}

function requireUniqueFragment(document: string, fragment: string): void {
  const first = document.indexOf(fragment);
  if (first === -1 || document.indexOf(fragment, first + fragment.length) !== -1) {
    throw failure("evidence_invalid", "screen-flow semantic fragment is not unique");
  }
}

function validateTaskAndPlan(
  taskDocument: string,
  planDocument: string,
  admittedPlanSha256: string,
): void {
  const heading = `### ${activeTask} |`;
  const start = taskDocument.indexOf(heading);
  if (start === -1 || taskDocument.indexOf(heading, start + heading.length) !== -1) {
    throw failure("evidence_invalid", "UI-002 active task binding is invalid");
  }
  const maybeEnd = taskDocument.indexOf("\n### ", start + heading.length);
  const taskBlock = taskDocument.slice(start, maybeEnd === -1 ? taskDocument.length : maybeEnd);
  for (const required of [
    expectedPlan,
    "No new hardware attempt or human checkpoint is authorized or\nrequired.",
  ]) {
    if (!taskBlock.includes(required)) {
      throw failure("evidence_invalid", "UI-002 active task binding is incomplete");
    }
  }
  if (sha256(planDocument) !== admittedPlanSha256
    || !planDocument.includes("- Parity row: `UI-002`")
    || !planDocument.includes(`- Active task: \`${activeTask}\``)
    || !planDocument.includes("`bitaxe-screen-flow-evidence-v1`")) {
    throw failure("evidence_invalid", "UI-002 immutable plan binding is invalid");
  }
}

async function validateSourceCompatibility(
  processPort: ProcessPort,
  gitProgram: string,
  workspaceRoot: string,
  attemptSourceCommit: string,
  currentSourceCommit: string,
  referenceCommit: string,
): Promise<void> {
  for (const sourcePath of unchangedPaths) {
    await childText(processPort, gitProgram,
      ["diff", "--quiet", attemptSourceCommit, currentSourceCommit, "--", sourcePath],
      "screen-flow module compatibility");
    const document = await childText(processPort, gitProgram,
      ["show", `${currentSourceCommit}:${sourcePath}`], "screen-flow source admission");
    for (const fragment of sourceFragments.get(sourcePath) ?? []) {
      requireUniqueFragment(document, fragment);
    }
  }
  for (const commit of [attemptSourceCommit, currentSourceCommit]) {
    const document = await childText(processPort, gitProgram,
      ["show", `${commit}:${semanticPath}`], "screen owner semantic admission");
    for (const fragment of sourceFragments.get(semanticPath) ?? []) {
      requireUniqueFragment(document, fragment);
    }
  }
  const referenceDocument = await childText(processPort, gitProgram,
    ["-C", path.join(workspaceRoot, "reference/esp-miner"), "show",
      `${referenceCommit}:main/screen.c`], "screen-flow reference admission");
  for (const fragment of referenceFragments) requireUniqueFragment(referenceDocument, fragment);
}

export async function projectScreenFlowEvidence(
  workspaceRoot: string,
  options: ScreenFlowEvidenceOptions,
  processPort: ProcessPort,
  gitProgram: string,
  validatorProgram: string,
  admittedDisplayUatSha256 = expectedDisplayUatSha256,
  admittedCommandEffectsSha256 = expectedCommandEffectsSha256,
  admittedPlanSha256 = expectedPlanSha256,
): Promise<ScreenFlowEvidence> {
  if (!lowerHex(options.attemptSourceCommit, 40)) {
    throw failure("evidence_invalid", "attempt source commit is invalid");
  }
  const displayUatPath = assertWithinWorkspace(workspaceRoot, options.sourceDisplayUat);
  const commandEffectsPath = assertWithinWorkspace(workspaceRoot, options.sourceCommandEffects);
  const projection = assertWithinWorkspace(workspaceRoot, options.projection);
  const candidate = `${projection}.candidate`;
  if (path.relative(workspaceRoot, displayUatPath) !== expectedDisplayUat
    || path.relative(workspaceRoot, commandEffectsPath) !== expectedCommandEffects
    || path.relative(workspaceRoot, projection) !== expectedProjection) {
    throw failure("evidence_invalid", "screen-flow projection path is invalid");
  }
  await requireAbsent(projection, "public projection");
  await requireAbsent(candidate, "projection candidate");

  const [displayUatDocument, commandEffectsDocument, taskDocument, planDocument] =
    await Promise.all([
      readFile(displayUatPath, "utf8"),
      readFile(commandEffectsPath, "utf8"),
      readFile(path.join(workspaceRoot, "TASKS.md"), "utf8"),
      readFile(path.join(workspaceRoot, expectedPlan), "utf8"),
    ]);
  if (sha256(displayUatDocument) !== admittedDisplayUatSha256
    || sha256(commandEffectsDocument) !== admittedCommandEffectsSha256) {
    throw failure("evidence_invalid", "screen-flow source projection digest is invalid");
  }
  validateTaskAndPlan(taskDocument, planDocument, admittedPlanSha256);
  const displayUat = parseObject(displayUatDocument, "display UAT projection");
  const commandEffects = parseObject(commandEffectsDocument, "command-effects projection");
  validateDisplayUat(displayUat, admittedCommandEffectsSha256);

  const [currentSourceCommit, upstreamCommit, referenceCommit] = await Promise.all([
    childText(processPort, gitProgram, ["rev-parse", "HEAD"], "current source identity"),
    childText(processPort, gitProgram, ["rev-parse", "@{upstream}"], "upstream source identity"),
    childText(processPort, gitProgram,
      ["-C", path.join(workspaceRoot, "reference/esp-miner"), "rev-parse", "HEAD"],
      "reference source identity"),
  ]);
  if (!lowerHex(currentSourceCommit, 40) || currentSourceCommit !== upstreamCommit
    || !lowerHex(referenceCommit, 40)) {
    throw failure("evidence_invalid", "screen-flow source identity is invalid");
  }
  validateCommandEffects(commandEffects, options.attemptSourceCommit, referenceCommit);
  await childText(processPort, gitProgram,
    ["cat-file", "-e", `${options.attemptSourceCommit}^{commit}`], "attempt source admission");
  await childText(processPort, gitProgram,
    ["merge-base", "--is-ancestor", options.attemptSourceCommit, currentSourceCommit],
    "attempt source ancestry");
  await validateSourceCompatibility(processPort, gitProgram, workspaceRoot,
    options.attemptSourceCommit, currentSourceCommit, referenceCommit);

  const relevantPaths = [
    ...unchangedPaths,
    semanticPath,
    expectedDisplayUat,
    expectedCommandEffects,
    expectedPlan,
    "TASKS.md",
    "crates/bitaxe-automation-contracts/src/screen_flow_evidence.rs",
    "crates/bitaxe-automation-contracts/src/bin/validate_screen_flow_evidence.rs",
    "tools/automation/src/screen-flow-evidence.ts",
  ];
  const dirty = await childText(processPort, gitProgram,
    ["status", "--porcelain", "--", ...relevantPaths], "screen-flow worktree state");
  if (dirty !== "") throw failure("evidence_invalid", "screen-flow paths have uncommitted drift");

  const requestSha256 = sha256(JSON.stringify({
    command: "project-screen-flow-evidence",
    source_display_uat: expectedDisplayUat,
    source_command_effects: expectedCommandEffects,
    attempt_source_commit: options.attemptSourceCommit,
    current_source_commit: currentSourceCommit,
    plan_sha256: admittedPlanSha256,
    projection: expectedProjection,
  }));
  const evidence: ScreenFlowEvidence = {
    schema_version: "bitaxe-screen-flow-evidence-v1",
    board: 205,
    attempt_source_commit: options.attemptSourceCommit,
    current_source_commit: currentSourceCommit,
    reference_commit: referenceCommit,
    workflow: {
      schema_version: "bitaxe-workflow-identity-v1",
      command: "project-screen-flow-evidence",
      request_sha256: requestSha256,
    },
    source: {
      display_uat_projection_sha256: sha256(displayUatDocument),
      command_effects_projection_sha256: sha256(commandEffectsDocument),
      source_task_sha256: sha256(taskDocument),
      plan_sha256: admittedPlanSha256,
      source_semantics_admitted: true,
      reference_semantics_admitted: true,
    },
    screen_flow: {
      identify_request_count: 1,
      machine_render_confirmed: true,
      machine_clear_confirmed: true,
      operator_render_confirmed: true,
      operator_clear_confirmed: true,
      priority_page_count: 6,
      intro_page_count: 2,
      carousel_page_count: 4,
      screen_update_ms: 500,
      intro_delay_ms: 3_000,
      carousel_delay_ms: 10_000,
      notification_mask_count: 8,
      paused_notification_admitted: true,
      identify_override_admitted: true,
      new_block_statistics_pin_admitted: true,
      bounded_private_frame_admitted: true,
      side_effect_free_projection_admitted: true,
      retained_screen_owner: true,
      change_only_rendering: true,
      priority_power_visibility_admitted: true,
      display_failure_isolated: true,
      compatible_path_count: 5,
    },
    build_identity_matches: true,
    usb_admission_confirmed: true,
    safe_stop_confirmed: true,
    cleanup_complete: true,
    mining_state: "disabled",
    hardware_control_state: "disabled",
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
