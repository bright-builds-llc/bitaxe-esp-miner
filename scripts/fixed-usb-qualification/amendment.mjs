import { lstat, readdir } from "node:fs/promises";
import { resolve } from "node:path";
import { cleanPushed, digest, exactObject, fileDigest, git, hex, protectedPath, readJson,
  REQUIRED_CYCLES, requireCondition, WINDOW_MS, writeNew } from "./contract.mjs";
import { validateCycle } from "./judge.mjs";
import { verifyArtifactSnapshot } from "./snapshot.mjs";

const FIRMWARE_FILES = new Set([
  "AGENTS.md", "TASKS.md", "docs/project/project-decisions.md",
  "docs/adr/0021-fixed-serial-jtag-worker-transport.md", "docs/adr/0022-four-cycle-fixed-usb-qualification.md",
  "docs/hardware/native-usb-ownership.md",
  ...["amendment.mjs", "snapshot.mjs", "contract.mjs", "preflight.mjs", "store.mjs",
    "judge.mjs", "server.mjs", "main.mjs", "supervisor.test.mjs"].map((name) => `scripts/fixed-usb-qualification/${name}`),
]);
const GATE_FILES = new Set(["docs/protocol/bwg-worker-serial-0.1.md", "docs/adr/0095-four-cycle-worker-qualification.md",
  ".scratch/bwg-worker-serial/issues/06-four-cycle-qualification.md"]);
const IDENTITY_FIELDS = ["firmware_commit", "gate_commit", "app_elf_sha256", "campaign_id", "window_limits_ms"];

export function checkQualificationSource(root, original, qualified, role) {
  requireCondition(hex(original, 40) && hex(qualified, 40), "qualification_commit");
  cleanPushed(root, qualified);
  requireCondition(git(root, ["merge-base", original, qualified]) === original, "qualification_ancestry");
  const allowed = role === "firmware" ? FIRMWARE_FILES : GATE_FILES;
  const changes = git(root, ["diff", "--name-status", "--no-renames", original, qualified]);
  for (const line of changes.split("\n").filter(Boolean)) {
    const [status, path, extra] = line.split("\t");
    requireCondition(["A", "M"].includes(status) && extra === undefined && allowed.has(path), "qualification_diff_forbidden");
    requireCondition(git(root, ["ls-tree", qualified, "--", path]).startsWith("100644 blob "), "qualification_file_mode");
  }
}

function unchangedLimits(context) {
  requireCondition(JSON.stringify(context.window_limits_ms) === JSON.stringify([180000, 30000, 30000]) &&
    JSON.stringify(WINDOW_MS) === JSON.stringify(context.window_limits_ms) && REQUIRED_CYCLES === 4,
  "qualification_limits_changed");
}
async function cycleReceipts(root, context) {
  let previous;
  const receipts = [];
  for (let cycle = 1; cycle <= REQUIRED_CYCLES; cycle += 1) {
    const path = resolve(root, `cycle-${cycle}.json`);
    await protectedPath(path);
    previous = validateCycle(await readJson(path), context, previous);
    receipts.push({ cycle, sha256: await fileDigest(path) });
  }
  return { receipts, baseline_id: previous.baseline_id };
}
async function verifyPolicy(root, context, policy, operations) {
  exactObject(policy, ["schema", "context_sha256", "required_no_mining_cycles", "previous_required_no_mining_cycles",
    "qualification_source_commit", "gate_qualification_source_commit", "artifact_snapshot_sha256", "cycle_receipts",
    "baseline_id", ...IDENTITY_FIELDS]);
  requireCondition(policy.schema === "fixed-usb-qualification-amendment-v1" &&
    policy.context_sha256 === digest(JSON.stringify(context)) && policy.required_no_mining_cycles === REQUIRED_CYCLES &&
    policy.previous_required_no_mining_cycles === 20 && !Object.hasOwn(context, "required_no_mining_cycles"), "amendment_policy");
  unchangedLimits(context);
  for (const key of IDENTITY_FIELDS) {
    requireCondition(JSON.stringify(policy[key]) === JSON.stringify(context[key]), "amendment_identity");
  }
  const check = operations.checkQualificationSource ?? checkQualificationSource;
  check(context.firmware_root, context.firmware_commit, policy.qualification_source_commit, "firmware");
  check(context.gate_root, context.gate_commit, policy.gate_qualification_source_commit, "gate");
  const cycles = await cycleReceipts(root, context);
  requireCondition(JSON.stringify(cycles.receipts) === JSON.stringify(policy.cycle_receipts) &&
    cycles.baseline_id === policy.baseline_id, "amendment_cycle_drift");
  const snapshot = await verifyArtifactSnapshot(root, context);
  requireCondition(snapshot.receipt_sha256 === policy.artifact_snapshot_sha256, "amendment_snapshot_drift");
  return { policy, gate_root: snapshot.gate_root };
}

export async function loadAmendment(root, context, operations = {}) {
  const path = resolve(root, "policy-amendment.json");
  try { await lstat(path); } catch (error) { if (error.code === "ENOENT") return undefined; throw error; }
  await protectedPath(path);
  const record = await readJson(path);
  exactObject(record, ["amendment", "sha256"]);
  requireCondition(record.sha256 === digest(JSON.stringify(record.amendment)), "amendment_integrity");
  return verifyPolicy(root, context, record.amendment, operations);
}

export async function amendPolicy(root, context, options, operations = {}) {
  await protectedPath(root, true);
  requireCondition(!(await readdir(root)).some((name) => /^window-\d+\./u.test(name)), "amendment_after_window");
  const cycles = await cycleReceipts(root, context);
  const snapshot = await verifyArtifactSnapshot(root, context);
  const identity = Object.fromEntries(IDENTITY_FIELDS.map((key) => [key, context[key]]));
  const policy = { schema: "fixed-usb-qualification-amendment-v1", context_sha256: digest(JSON.stringify(context)),
    required_no_mining_cycles: REQUIRED_CYCLES, previous_required_no_mining_cycles: 20,
    qualification_source_commit: options.qualificationSourceCommit,
    gate_qualification_source_commit: options.gateQualificationSourceCommit,
    artifact_snapshot_sha256: snapshot.receipt_sha256, cycle_receipts: cycles.receipts,
    baseline_id: cycles.baseline_id, ...identity };
  await verifyPolicy(root, context, policy, operations);
  const sha256 = digest(JSON.stringify(policy));
  await writeNew(resolve(root, "policy-amendment.json"), { amendment: policy, sha256 });
  return { schema: "fixed-usb-policy-amendment-result-v1", amendment_sha256: sha256,
    required_no_mining_cycles: REQUIRED_CYCLES, original_context_unchanged: true, device_effects: false };
}
