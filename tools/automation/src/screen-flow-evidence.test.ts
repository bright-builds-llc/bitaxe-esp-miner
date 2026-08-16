import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { mkdtemp, mkdir, readFile, stat, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import {
  projectScreenFlowEvidence,
  ScreenFlowEvidenceError,
} from "./screen-flow-evidence.js";
import { createFakeProcessPort, type ProcessOutcome, type ProcessPort } from "./process.js";

const attemptCommit = "a".repeat(40);
const currentCommit = "b".repeat(40);
const referenceCommit = "c".repeat(40);
const ok = (stdout = ""): ProcessOutcome => ({ exitCode: 0, stdout, stderr: "", timedOut: false });
const digest = (value: string): string => createHash("sha256").update(value).digest("hex");

const sources = new Map<string, string>([
  ["crates/bitaxe-core/src/screen.rs", `
pub const SCREEN_UPDATE_MS: u64 = 500;
pub const INTRO_DELAY_MS: u64 = 3_000;
pub const CAROUSEL_DELAY_MS: u64 = 10_000;
} else if snapshot.show_new_block {
frame(["", "BITAXE IDENTIFY", "Hello!", ""])
fn notification(&mut self, snapshot: &ScreenSnapshot) -> Notification {
`],
  ["crates/bitaxe-core/src/screen/frame.rs", `
pub fn private_lines(&self) -> [&str; SCREEN_LINE_COUNT] {
pub fn fits_ultra205(&self) -> bool {
fn clean_line(value: &str) -> String {
fn frame_with_notification(
`],
  ["firmware/bitaxe/src/runtime_snapshot/screen.rs", `
pub fn collect_screen_snapshot(now_ms: u64) -> ScreenSnapshot {
let command = screen_command_projection(now_ms);
let pool_host = screen_pool_host(command.fallback_active);
`],
  ["firmware/bitaxe/src/display_adapter.rs", `
pub fn render_runtime_screen(
frame.private_lines(),
self.power_policy
`],
  ["firmware/bitaxe/src/operator_sensor_runtime.rs", `
struct RuntimeDisplay {
if display.maybe_last_frame.as_ref() == Some(&decision.frame) {
.service_power(owner, &mut i2c_budget, uptime_ms, decision.priority_visible)
.render_runtime_screen(owner, &mut i2c_budget, &decision.frame)
disable_runtime_display(maybe_display, "render_failed", &error);
`],
  ["main/screen.c", `
#define SCREEN_UPDATE_MS 500
static int delays_ms[MAX_SCREENS] = {0, 0, 0, 0, 0, 1000, 3000, 3000, 10000, 10000, 10000, 10000};
static const char *notifications[] = {
static bool screen_show(screen_t screen)
static void screen_update_cb(lv_timer_t * timer)
if (get_current_screen() != SCR_STATS) {
lv_timer_create(screen_update_cb, SCREEN_UPDATE_MS, NULL);
`],
]);

function commandEffects() {
  return {
    schema_version: "bitaxe-api-command-effects-evidence-v1",
    board: 205,
    source_commit: attemptCommit,
    reference_commit: referenceCommit,
    command_effects: {
      identify_status_baseline_confirmed: true,
      identify_request_count: 1,
      identify_render_receipt_confirmed: true,
      identify_clear_receipt_confirmed: true,
      retained_identify_transition_confirmed: true,
      serial_transition_witnesses_confirmed: true,
      identify_terminal_outcome: "none",
      same_boot_and_package: true,
    },
    safe_stop_confirmed: true,
    cleanup_complete: true,
    recovery_attempted: false,
    mining_state: "disabled",
    hardware_control_state: "disabled",
    redaction_status: "passed",
  };
}

async function fixture(name: string, options: {
  readonly operatorClear?: boolean;
  readonly taskValid?: boolean;
  readonly planValid?: boolean;
  readonly existingCandidate?: boolean;
} = {}) {
  const root = await mkdtemp(path.join(os.tmpdir(), `bitaxe-screen-flow-${name}-`));
  await writeFile(path.join(root, "MODULE.bazel"), "module(name = \"fixture\")\n");
  const commandPath = path.join(root,
    "docs/parity/evidence/api009-command-effects/command-effects-projection-attempt-046.json");
  const uatPath = path.join(root,
    "docs/parity/evidence/api009-command-effects/display-uat-projection-attempt-005.json");
  await mkdir(path.dirname(commandPath), { recursive: true });
  const commandDocument = `${JSON.stringify(commandEffects(), null, 2)}\n`;
  const commandSha256 = digest(commandDocument);
  const uatDocument = `${JSON.stringify({
    schema_version: "bitaxe-display-uat-evidence-v1",
    board: 205,
    identify_request_count: 1,
    machine_render_confirmed: true,
    machine_clear_confirmed: true,
    operator_render_confirmed: true,
    operator_clear_confirmed: options.operatorClear ?? true,
    build_identity_matches: true,
    usb_admission_confirmed: true,
    programmatic_evidence_sha256: commandSha256,
    redaction_status: "passed",
  }, null, 2)}\n`;
  await writeFile(commandPath, commandDocument);
  await writeFile(uatPath, uatDocument);

  const planPath = path.join(root, "docs/parity/work-plans/20260816T073911Z-UI-002/PLAN.md");
  await mkdir(path.dirname(planPath), { recursive: true });
  const planDocument = `# Plan

- Parity row: \`UI-002\`
- Active task: \`task-parity-ui002-screen-flow\`
- Contract: \`${options.planValid === false ? "wrong" : "bitaxe-screen-flow-evidence-v1"}\`
`;
  await writeFile(planPath, planDocument);
  await writeFile(path.join(root, "TASKS.md"), `
### task-parity-ui002-screen-flow | 2026-08-04 | Screen

Plan: \`docs/parity/work-plans/20260816T073911Z-UI-002/PLAN.md\`.

${options.taskValid === false ? "Hardware contract missing." : "No new hardware attempt or human checkpoint is authorized or\nrequired."}

### next-task | later | Other
`);
  const projection = path.join(root,
    "docs/parity/evidence/ui002-screen-flow/screen-flow-projection.json");
  if (options.existingCandidate === true) {
    await mkdir(path.dirname(projection), { recursive: true });
    await writeFile(`${projection}.candidate`, "occupied\n");
  }
  return {
    root,
    projection,
    options: {
      sourceDisplayUat: uatPath,
      sourceCommandEffects: commandPath,
      attemptSourceCommit: attemptCommit,
      projection,
    },
    uatSha256: digest(uatDocument),
    commandSha256,
    planSha256: digest(planDocument),
  };
}

function fakePort(options: {
  readonly sourceDrift?: boolean;
  readonly duplicateFragment?: boolean;
  readonly missingFragment?: boolean;
  readonly dirty?: boolean;
  readonly upstreamDrift?: boolean;
  readonly validatorFailure?: boolean;
  readonly launchFailure?: boolean;
} = {}): ProcessPort {
  return createFakeProcessPort(async (spec) => {
    if (options.launchFailure) throw new Error("launch failed");
    if (options.validatorFailure && spec.program === "validator") {
      return { exitCode: 1, stdout: "", stderr: "rejected", timedOut: false };
    }
    if (spec.args[0] === "rev-parse") {
      return ok(spec.args[1] === "@{upstream}" && options.upstreamDrift
        ? `${"d".repeat(40)}\n`
        : `${currentCommit}\n`);
    }
    if (spec.args[0] === "-C" && spec.args[2] === "rev-parse") {
      return ok(`${referenceCommit}\n`);
    }
    if (spec.args[0] === "status") return ok(options.dirty ? " M screen.rs\n" : "");
    if (spec.args[0] === "diff" && options.sourceDrift) {
      return { exitCode: 1, stdout: "", stderr: "", timedOut: false };
    }
    if (spec.args[0] === "show" || (spec.args[0] === "-C" && spec.args[2] === "show")) {
      const target = spec.args[0] === "-C" ? spec.args[3] ?? "" : spec.args[1] ?? "";
      for (const [suffix, source] of sources) {
        if (target.endsWith(suffix)) {
          if (options.missingFragment && suffix.endsWith("screen.rs")) return ok("");
          return ok(options.duplicateFragment && suffix.endsWith("operator_sensor_runtime.rs")
            ? `${source}\n${source}`
            : source);
        }
      }
    }
    return ok();
  });
}

async function projectFixture(
  value: Awaited<ReturnType<typeof fixture>>,
  processPort: ProcessPort,
) {
  return projectScreenFlowEvidence(
    value.root,
    value.options,
    processPort,
    "git",
    "validator",
    value.uatSha256,
    value.commandSha256,
    value.planSha256,
  );
}

async function captureError(promise: Promise<unknown>): Promise<ScreenFlowEvidenceError> {
  try {
    await promise;
    assert.fail("expected projection failure");
  } catch (error) {
    assert.ok(error instanceof ScreenFlowEvidenceError);
    return error;
  }
}

test("accepted screen-flow transaction emits only closed row evidence", async () => {
  // Arrange
  const value = await fixture("ready");

  // Act
  const evidence = await projectFixture(value, fakePort());

  // Assert
  assert.equal(evidence.screen_flow.priority_page_count, 6);
  assert.equal(evidence.screen_flow.carousel_page_count, 4);
  assert.equal(evidence.screen_flow.operator_clear_confirmed, true);
  assert.equal(evidence.hardware_rerun_used, false);
  assert.equal((await stat(value.projection)).mode & 0o777, 0o644);
  assert.doesNotMatch(await readFile(value.projection, "utf8"),
    /voltage|millivolt|hostname|origin|usbmodem|ssid|password|private\/|scratch\//iu);
});

for (const [name, fixtureOptions, processOptions, category] of [
  ["incomplete-operator-quorum", { operatorClear: false }, {}, "evidence_invalid"],
  ["task-drift", { taskValid: false }, {}, "evidence_invalid"],
  ["plan-drift", { planValid: false }, {}, "evidence_invalid"],
  ["candidate-survives", { existingCandidate: true }, {}, "evidence_invalid"],
  ["source-drift", {}, { sourceDrift: true }, "evidence_invalid"],
  ["missing-fragment", {}, { missingFragment: true }, "evidence_invalid"],
  ["duplicate-fragment", {}, { duplicateFragment: true }, "evidence_invalid"],
  ["dirty-source", {}, { dirty: true }, "evidence_invalid"],
  ["upstream-drift", {}, { upstreamDrift: true }, "evidence_invalid"],
  ["validator-rejected", {}, { validatorFailure: true }, "evidence_invalid"],
  ["launch-failed", {}, { launchFailure: true }, "process_failed"],
] as const) {
  test(`${name} withholds final screen-flow evidence`, async () => {
    // Arrange
    const value = await fixture(name, fixtureOptions);

    // Act
    const error = await captureError(projectFixture(value, fakePort(processOptions)));

    // Assert
    assert.equal(error.category, category);
    assert.deepEqual(error.publicValue, {
      stage: "sealed_screen_flow_projection",
      hardware_rerun_used: false,
      projection_published: false,
    });
    await assert.rejects(readFile(value.projection, "utf8"), { code: "ENOENT" });
  });
}
