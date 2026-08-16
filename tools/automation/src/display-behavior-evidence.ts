import { createHash } from "node:crypto";
import { chmod, mkdir, readFile, rename, stat, unlink, writeFile } from "node:fs/promises";
import path from "node:path";

import {
  internalCommandSpec,
  type AutomationCategory,
  type DisplayBehaviorEvidence,
} from "./contracts.generated.js";
import type { ProcessPort } from "./process.js";
import { assertWithinWorkspace } from "./workspace.js";

export type DisplayBehaviorEvidenceOptions = {
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
const expectedPlan = "docs/parity/work-plans/20260816T064239Z-UI-001/PLAN.md";
const expectedPlanSha256 =
  "ae4602e841750d14e86e759b5c2815834567beafcdef63503e88eb49c138b966";
const activeTask = "task-parity-ui001-display-behavior";

const unchangedPaths = [
  "crates/bitaxe-core/src/display.rs",
  "crates/bitaxe-config/src/display.rs",
  "firmware/bitaxe/src/display_adapter.rs",
  "firmware/bitaxe/src/startup.rs",
] as const;
const semanticPath = "firmware/bitaxe/src/operator_sensor_runtime.rs";

const sourceFragments = new Map<string, readonly string[]>([
  [unchangedPaths[0], [
    "pub const ULTRA205_DISPLAY_NAME: &str = \"SSD1306 (128x32)\";",
    "0 => Ok(Self::Rotate0),",
    "90 => Ok(Self::Rotate90),",
    "180 => Ok(Self::Rotate180),",
    "270 => Ok(Self::Rotate270),",
    "-1 => Ok(Self::AlwaysOn),",
    "0 => Ok(Self::ActivityOnly),",
    "pub enum DisplayPowerCommand {",
  ]],
  [unchangedPaths[1], [
    "pub fn load_ultra205_display_configuration(",
    "let inverted = match snapshot.maybe_stored_value(\"invertscreen\")",
    "let timeout_minutes = match snapshot.maybe_stored_value(\"displayTimeout\")",
  ]],
  [unchangedPaths[2], [
    "pub struct RuntimeDisplayOwner {",
    "render_debug_text(bus.startup_display(), frame, configuration, true)?;",
    ".command_at(now_ms, priority_visible)",
    ".set_display_on(on)",
  ]],
  [unchangedPaths[3], [
    "bitaxe_config::load_ultra205_display_configuration(&confirmed_settings)",
    "display_adapter::RuntimeDisplayOwner::initialize(",
  ]],
  [semanticPath, [
    "struct RuntimeDisplay {",
    ".service_power(owner, &mut i2c_budget, uptime_ms, decision.priority_visible)",
    ".render_runtime_screen(owner, &mut i2c_budget, &decision.frame)",
    "disable_runtime_display(maybe_display, \"render_failed\", &error);",
  ]],
]);

const referenceFragments = new Map<string, readonly string[]>([
  ["main/display.c", [
    "bool invert_screen = nvs_config_get_bool(NVS_CONFIG_INVERT_SCREEN);",
    "uint16_t rotation = nvs_config_get_u16(NVS_CONFIG_ROTATION);",
    "ESP_RETURN_ON_ERROR(display_on(true), TAG, \"Display on failed\");",
  ]],
  ["main/screen.c", [
    "int32_t display_timeout_config = nvs_config_get_i32(NVS_CONFIG_DISPLAY_TIMEOUT);",
    "bool is_identify_mode = module->identify_mode_time_ms > 0;",
    "display_on(enable_display);",
  ]],
]);

export class DisplayBehaviorEvidenceError extends Error {
  public constructor(
    public readonly category: FailureCategory,
    message: string,
    public readonly publicValue: Readonly<Record<string, unknown>>,
  ) {
    super(message);
    this.name = "DisplayBehaviorEvidenceError";
  }
}

function failure(category: FailureCategory, message: string): DisplayBehaviorEvidenceError {
  return new DisplayBehaviorEvidenceError(category, message, {
    stage: "sealed_display_behavior_projection",
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
    if (error instanceof DisplayBehaviorEvidenceError) throw error;
    throw failure("process_failed", `${context} launch failed`);
  }
}

async function requireAbsent(candidate: string, context: string): Promise<void> {
  try {
    await stat(candidate);
    throw failure("evidence_invalid", `${context} must be absent before projection`);
  } catch (error) {
    if (error instanceof DisplayBehaviorEvidenceError) throw error;
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
    throw failure("evidence_invalid", "display UAT quorum is incomplete");
  }
}

function validateCommandEffects(
  command: Record<string, unknown>,
  attemptSourceCommit: string,
  referenceCommit: string,
): void {
  const maybeEffects = command["command_effects"];
  if (typeof maybeEffects !== "object" || maybeEffects === null || Array.isArray(maybeEffects)) {
    throw failure("evidence_invalid", "command-effects display quorum is incomplete");
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
    throw failure("evidence_invalid", "command-effects display quorum is incomplete");
  }
}

function requireUniqueFragment(document: string, fragment: string): void {
  const first = document.indexOf(fragment);
  if (first === -1 || document.indexOf(fragment, first + fragment.length) !== -1) {
    throw failure("evidence_invalid", "display semantic fragment is not unique");
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
    throw failure("evidence_invalid", "UI-001 active task binding is invalid");
  }
  const maybeEnd = taskDocument.indexOf("\n### ", start + heading.length);
  const taskBlock = taskDocument.slice(start, maybeEnd === -1 ? taskDocument.length : maybeEnd);
  for (const required of [expectedPlan, "No new hardware attempt is authorized or required."]) {
    if (!taskBlock.includes(required)) {
      throw failure("evidence_invalid", "UI-001 active task binding is incomplete");
    }
  }
  if (sha256(planDocument) !== admittedPlanSha256
    || !planDocument.includes("- Parity row: `UI-001`")
    || !planDocument.includes(`- Active task: \`${activeTask}\``)
    || !planDocument.includes("`bitaxe-display-behavior-evidence-v1`")) {
    throw failure("evidence_invalid", "UI-001 immutable plan binding is invalid");
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
      "display-owned module compatibility");
    const document = await childText(processPort, gitProgram,
      ["show", `${currentSourceCommit}:${sourcePath}`], "display source admission");
    for (const fragment of sourceFragments.get(sourcePath) ?? []) {
      requireUniqueFragment(document, fragment);
    }
  }
  for (const commit of [attemptSourceCommit, currentSourceCommit]) {
    const document = await childText(processPort, gitProgram,
      ["show", `${commit}:${semanticPath}`], "display owner semantic admission");
    for (const fragment of sourceFragments.get(semanticPath) ?? []) {
      requireUniqueFragment(document, fragment);
    }
  }
  for (const [referencePath, fragments] of referenceFragments) {
    const document = await childText(processPort, gitProgram,
      ["-C", path.join(workspaceRoot, "reference/esp-miner"), "show",
        `${referenceCommit}:${referencePath}`], "display reference admission");
    for (const fragment of fragments) requireUniqueFragment(document, fragment);
  }
}

export async function projectDisplayBehaviorEvidence(
  workspaceRoot: string,
  options: DisplayBehaviorEvidenceOptions,
  processPort: ProcessPort,
  gitProgram: string,
  validatorProgram: string,
  admittedDisplayUatSha256 = expectedDisplayUatSha256,
  admittedCommandEffectsSha256 = expectedCommandEffectsSha256,
  admittedPlanSha256 = expectedPlanSha256,
): Promise<DisplayBehaviorEvidence> {
  if (!lowerHex(options.attemptSourceCommit, 40)) {
    throw failure("evidence_invalid", "attempt source commit is invalid");
  }
  const displayUatPath = assertWithinWorkspace(workspaceRoot, options.sourceDisplayUat);
  const commandEffectsPath = assertWithinWorkspace(workspaceRoot, options.sourceCommandEffects);
  const projection = assertWithinWorkspace(workspaceRoot, options.projection);
  const candidate = `${projection}.candidate`;
  if (path.relative(workspaceRoot, displayUatPath) !== expectedDisplayUat
    || path.relative(workspaceRoot, commandEffectsPath) !== expectedCommandEffects) {
    throw failure("evidence_invalid", "display source projection path is invalid");
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
    throw failure("evidence_invalid", "display source projection digest is invalid");
  }
  validateTaskAndPlan(taskDocument, planDocument, admittedPlanSha256);
  const displayUat = parseObject(displayUatDocument, "display UAT projection");
  const commandEffects = parseObject(commandEffectsDocument, "command-effects projection");
  validateDisplayUat(displayUat, admittedCommandEffectsSha256);

  const [currentSourceCommit, referenceCommit] = await Promise.all([
    childText(processPort, gitProgram, ["rev-parse", "HEAD"], "current source identity"),
    childText(processPort, gitProgram,
      ["-C", path.join(workspaceRoot, "reference/esp-miner"), "rev-parse", "HEAD"],
      "reference source identity"),
  ]);
  if (!lowerHex(currentSourceCommit, 40) || !lowerHex(referenceCommit, 40)) {
    throw failure("evidence_invalid", "display source identity is invalid");
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
  ];
  const dirty = await childText(processPort, gitProgram,
    ["status", "--porcelain", "--", ...relevantPaths], "display-behavior worktree state");
  if (dirty !== "") throw failure("evidence_invalid", "display-behavior paths have uncommitted drift");

  const requestSha256 = sha256(JSON.stringify({
    command: "project-display-behavior-evidence",
    source_display_uat: expectedDisplayUat,
    source_command_effects: expectedCommandEffects,
    attempt_source_commit: options.attemptSourceCommit,
    current_source_commit: currentSourceCommit,
    plan_sha256: admittedPlanSha256,
    projection: path.relative(workspaceRoot, projection),
  }));
  const evidence: DisplayBehaviorEvidence = {
    schema_version: "bitaxe-display-behavior-evidence-v1",
    board: 205,
    attempt_source_commit: options.attemptSourceCommit,
    current_source_commit: currentSourceCommit,
    reference_commit: referenceCommit,
    workflow: {
      schema_version: "bitaxe-workflow-identity-v1",
      command: "project-display-behavior-evidence",
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
    display: {
      identify_request_count: 1,
      machine_render_confirmed: true,
      machine_clear_confirmed: true,
      operator_render_confirmed: true,
      operator_clear_confirmed: true,
      exact_panel_admitted: true,
      supported_rotation_count: 4,
      inversion_state_count: 2,
      timeout_mode_count: 3,
      retained_display_owner: true,
      configuration_before_first_render: true,
      edge_triggered_power_commands: true,
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
