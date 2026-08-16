import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { mkdtemp, mkdir, readFile, stat, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import {
  DisplayBehaviorEvidenceError,
  projectDisplayBehaviorEvidence,
} from "./display-behavior-evidence.js";
import { createFakeProcessPort, type ProcessOutcome, type ProcessPort } from "./process.js";

const attemptCommit = "a".repeat(40);
const currentCommit = "b".repeat(40);
const referenceCommit = "c".repeat(40);
const ok = (stdout = ""): ProcessOutcome => ({ exitCode: 0, stdout, stderr: "", timedOut: false });
const digest = (value: string): string => createHash("sha256").update(value).digest("hex");

const sources = new Map<string, string>([
  ["crates/bitaxe-core/src/display.rs", `
pub const ULTRA205_DISPLAY_NAME: &str = "SSD1306 (128x32)";
0 => Ok(Self::Rotate0),
90 => Ok(Self::Rotate90),
180 => Ok(Self::Rotate180),
270 => Ok(Self::Rotate270),
-1 => Ok(Self::AlwaysOn),
0 => Ok(Self::ActivityOnly),
pub enum DisplayPowerCommand {
`],
  ["crates/bitaxe-config/src/display.rs", `
pub fn load_ultra205_display_configuration(
let inverted = match snapshot.maybe_stored_value("invertscreen")
let timeout_minutes = match snapshot.maybe_stored_value("displayTimeout")
`],
  ["firmware/bitaxe/src/display_adapter.rs", `
pub struct RuntimeDisplayOwner {
render_debug_text(bus.startup_display(), frame, configuration, true)?;
.command_at(now_ms, priority_visible)
.set_display_on(on)
`],
  ["firmware/bitaxe/src/startup.rs", `
bitaxe_config::load_ultra205_display_configuration(&confirmed_settings)
display_adapter::RuntimeDisplayOwner::initialize(
`],
  ["firmware/bitaxe/src/operator_sensor_runtime.rs", `
struct RuntimeDisplay {
.service_power(owner, &mut i2c_budget, uptime_ms, decision.priority_visible)
.render_runtime_screen(owner, &mut i2c_budget, &decision.frame)
disable_runtime_display(maybe_display, "render_failed", &error);
`],
  ["main/display.c", `
bool invert_screen = nvs_config_get_bool(NVS_CONFIG_INVERT_SCREEN);
uint16_t rotation = nvs_config_get_u16(NVS_CONFIG_ROTATION);
ESP_RETURN_ON_ERROR(display_on(true), TAG, "Display on failed");
`],
  ["main/screen.c", `
int32_t display_timeout_config = nvs_config_get_i32(NVS_CONFIG_DISPLAY_TIMEOUT);
bool is_identify_mode = module->identify_mode_time_ms > 0;
display_on(enable_display);
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

async function fixture(name: string, operatorClear = true) {
  const root = await mkdtemp(path.join(os.tmpdir(), `bitaxe-display-${name}-`));
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
    operator_clear_confirmed: operatorClear,
    build_identity_matches: true,
    usb_admission_confirmed: true,
    programmatic_evidence_sha256: commandSha256,
    redaction_status: "passed",
  }, null, 2)}\n`;
  await writeFile(commandPath, commandDocument);
  await writeFile(uatPath, uatDocument);

  const planPath = path.join(root, "docs/parity/work-plans/20260816T064239Z-UI-001/PLAN.md");
  await mkdir(path.dirname(planPath), { recursive: true });
  const planDocument = `# Plan

- Parity row: \`UI-001\`
- Active task: \`task-parity-ui001-display-behavior\`
- Contract: \`bitaxe-display-behavior-evidence-v1\`
`;
  await writeFile(planPath, planDocument);
  await writeFile(path.join(root, "TASKS.md"), `
### task-parity-ui001-display-behavior | 2026-08-04 | Display

Plan: \`docs/parity/work-plans/20260816T064239Z-UI-001/PLAN.md\`.

No new hardware attempt is authorized or required.

### next-task | later | Other
`);
  return {
    root,
    projection: path.join(root,
      "docs/parity/evidence/ui001-display-behavior/display-behavior-projection.json"),
    options: {
      sourceDisplayUat: uatPath,
      sourceCommandEffects: commandPath,
      attemptSourceCommit: attemptCommit,
      projection: path.join(root,
        "docs/parity/evidence/ui001-display-behavior/display-behavior-projection.json"),
    },
    uatSha256: digest(uatDocument),
    commandSha256,
    planSha256: digest(planDocument),
  };
}

function fakePort(options: {
  readonly sourceDrift?: boolean;
  readonly duplicateFragment?: boolean;
  readonly dirty?: boolean;
  readonly validatorFailure?: boolean;
  readonly launchFailure?: boolean;
} = {}): ProcessPort {
  return createFakeProcessPort(async (spec) => {
    if (options.launchFailure) throw new Error("launch failed");
    if (options.validatorFailure && spec.program === "validator") {
      return { exitCode: 1, stdout: "", stderr: "rejected", timedOut: false };
    }
    if (spec.args[0] === "rev-parse") return ok(`${currentCommit}\n`);
    if (spec.args[0] === "-C" && spec.args[2] === "rev-parse") return ok(`${referenceCommit}\n`);
    if (spec.args[0] === "status") return ok(options.dirty ? " M display.rs\n" : "");
    if (spec.args[0] === "diff" && options.sourceDrift) {
      return { exitCode: 1, stdout: "", stderr: "", timedOut: false };
    }
    if (spec.args[0] === "show" || (spec.args[0] === "-C" && spec.args[2] === "show")) {
      const target = spec.args[0] === "-C" ? spec.args[3] ?? "" : spec.args[1] ?? "";
      for (const [suffix, source] of sources) {
        if (target.endsWith(suffix)) {
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
  return projectDisplayBehaviorEvidence(
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

async function captureError(promise: Promise<unknown>): Promise<DisplayBehaviorEvidenceError> {
  try {
    await promise;
    assert.fail("expected projection failure");
  } catch (error) {
    assert.ok(error instanceof DisplayBehaviorEvidenceError);
    return error;
  }
}

test("accepted display transaction emits only closed row evidence", async () => {
  // Arrange
  const value = await fixture("ready");

  // Act
  const evidence = await projectFixture(value, fakePort());

  // Assert
  assert.equal(evidence.display.identify_request_count, 1);
  assert.equal(evidence.display.operator_clear_confirmed, true);
  assert.equal(evidence.hardware_rerun_used, false);
  assert.equal((await stat(value.projection)).mode & 0o777, 0o644);
  assert.doesNotMatch(await readFile(value.projection, "utf8"),
    /voltage|millivolt|hostname|origin|usbmodem|ssid|password|private\/|scratch\//iu);
});

for (const [name, operatorClear, options, category] of [
  ["incomplete-operator-quorum", false, {}, "evidence_invalid"],
  ["source-drift", true, { sourceDrift: true }, "evidence_invalid"],
  ["duplicate-fragment", true, { duplicateFragment: true }, "evidence_invalid"],
  ["dirty-source", true, { dirty: true }, "evidence_invalid"],
  ["validator-rejected", true, { validatorFailure: true }, "evidence_invalid"],
  ["launch-failed", true, { launchFailure: true }, "process_failed"],
] as const) {
  test(`${name} withholds final display evidence`, async () => {
    // Arrange
    const value = await fixture(name, operatorClear);

    // Act
    const error = await captureError(projectFixture(value, fakePort(options)));

    // Assert
    assert.equal(error.category, category);
    assert.deepEqual(error.publicValue, {
      stage: "sealed_display_behavior_projection",
      hardware_rerun_used: false,
      projection_published: false,
    });
    await assert.rejects(readFile(value.projection, "utf8"), { code: "ENOENT" });
    await assert.rejects(readFile(`${value.projection}.candidate`, "utf8"), { code: "ENOENT" });
  });
}
