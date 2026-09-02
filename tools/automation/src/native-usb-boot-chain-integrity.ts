import { spawn } from "node:child_process";
import { createHash } from "node:crypto";
import { chmod, lstat, mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import { requireNoOwnedUsbProcesses } from "./native-usb-transition-recovery.js";
import { runCampaignProcess } from "./stratum-v2-campaign.js";
import type { JsonObject } from "./stratum-v2-campaign-preflight.js";
import { validateTcpPayloadRecoveryTooling } from "./stratum-v2-tcp-recovery-tooling.js";
import { sourceWorkspaceRoot } from "./workspace.js";

type Action = "preflight" | "capture" | "read" | "finalize";
type Args = { action: Action; port: string; packageManifest: string; restoreBundle: string; privateRoot: string; projection: string; plan: string };
const plan = "docs/parity/work-plans/20260902T022334Z-NATIVE-USB-BOOT-CHAIN-INTEGRITY/PLAN.md";
const planSha256 = "4eb4ae0d412d0cb4b56ccd640f407ae000929c564f7ae78554bfb4a893553fa1";
const root = "scratch/native-usb-boot-chain-integrity/attempt-001";
const projection = "docs/parity/evidence/native-usb-boot-chain-integrity/boot-chain-projection-001.json";
const manifest = "bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json";
const bundle = "scratch/str005-installed-package-recovery/recovery-006/restore-bundle.private.json";
const predecessor = "scratch/native-usb-owner-recovery/attempt-001/recovery-result.private.json";
const fail = (category: string): never => { throw new Error(category); };
const object = (value: unknown): JsonObject => typeof value === "object" && value !== null && !Array.isArray(value) ? value as JsonObject : fail("evidence_invalid");
const sha256 = (value: string): string => createHash("sha256").update(value).digest("hex");
async function absent(candidate: string): Promise<boolean> { try { await lstat(candidate); return false; } catch (error) { if ((error as NodeJS.ErrnoException).code === "ENOENT") return true; throw error; } }
async function mode(candidate: string, expected: number, directory = false): Promise<void> { const value = await lstat(candidate); if (value.isSymbolicLink() || (directory ? !value.isDirectory() : !value.isFile()) || (value.mode & 0o777) !== expected) fail("protected_mode"); }
async function privateWrite(candidate: string, value: unknown): Promise<void> { await writeFile(candidate, `${JSON.stringify(value, null, 2)}\n`, { flag: "wx", mode: 0o600 }); await chmod(candidate, 0o600); }

export function parseBootChainArgs(action: string | undefined, values: readonly string[]): Args {
  if (!(["preflight", "capture", "read", "finalize"] as const).includes(action as Action)) fail("invalid_invocation");
  const options = new Map<string, string | true>();
  for (let i = 0; i < values.length; i += 1) {
    const key = values[i] ?? fail("invalid_invocation");
    if (key === "--redact-evidence") { options.set(key, true); continue; }
    const optionValue = values[i + 1] ?? fail("invalid_invocation");
    if (!key.startsWith("--") || optionValue.startsWith("--")) fail("invalid_invocation");
    options.set(key, optionValue); i += 1;
  }
  const value = (key: string): string => typeof options.get(key) === "string" ? options.get(key) as string : fail("invalid_invocation");
  const args = { action: action as Action, port: value("--port"), packageManifest: value("--package-manifest"), restoreBundle: value("--restore-bundle"), privateRoot: value("--private-root"), projection: value("--projection"), plan: value("--plan") };
  if (value("--board") !== "205" || args.packageManifest !== manifest || args.restoreBundle !== bundle || args.privateRoot !== root || args.projection !== projection || args.plan !== plan || options.get("--redact-evidence") !== true) fail("invalid_invocation");
  return args;
}
export function bootChainWorkspaceRoot(environment = process.env, cwd = process.cwd()): string { const configured = environment["BUILD_WORKSPACE_DIRECTORY"]; return sourceWorkspaceRoot(configured === undefined ? [cwd] : [configured, cwd]); }

async function admission(workspace: string, rootAbsent: boolean): Promise<void> {
  if ((await absent(path.join(workspace, root))) !== rootAbsent || !(await absent(path.join(workspace, projection)))) fail("outputs");
  if (!rootAbsent) await mode(path.join(workspace, root), 0o700, true);
  const [planText, tasks, head, status, sync, manifestText] = await Promise.all([readFile(path.join(workspace, plan), "utf8"), readFile(path.join(workspace, "TASKS.md"), "utf8"), runCampaignProcess(workspace, "git", ["rev-parse", "HEAD"], 5000), runCampaignProcess(workspace, "git", ["status", "--porcelain", "--untracked-files=all"], 5000), runCampaignProcess(workspace, "git", ["rev-list", "--left-right", "--count", "HEAD...@{upstream}"], 5000), readFile(path.join(workspace, manifest), "utf8")]);
  if (sha256(planText) !== planSha256 || !tasks.includes("### task-native-usb-boot-chain-integrity-205") || status.stdout.trim() !== "" || sync.stdout.trim() !== "0\t0" || object(JSON.parse(manifestText))["source_commit"] !== head.stdout.trim()) fail("source_identity");
  await mode(path.join(workspace, bundle), 0o600); await mode(path.join(workspace, predecessor), 0o600);
  const previous = object(JSON.parse(await readFile(path.join(workspace, predecessor), "utf8")));
  if (previous["terminal_category"] !== "application_missing" || previous["cleanup_complete"] !== true || previous["device_write_observed"] !== false) fail("predecessor_state");
  await validateTcpPayloadRecoveryTooling(workspace, runCampaignProcess); await requireNoOwnedUsbProcesses();
}

async function runHelper(workspace: string, helper: string, intent: string, result: string): Promise<JsonObject> {
  const child = spawn("/usr/bin/xcrun", ["swift", path.join(workspace, helper), "--intent", intent, "--result", result], { cwd: workspace, stdio: "ignore" });
  const exit = await new Promise<number | null>((resolve, reject) => { child.once("error", reject); child.once("close", resolve); });
  if (exit !== 0) fail("checkpoint_failed"); await mode(result, 0o600); const value = object(JSON.parse(await readFile(result, "utf8"))); if (value["status"] !== "accepted") fail("checkpoint_cancelled"); return value;
}

export function projectBootChain(machine: JsonObject, evaluator: string): JsonObject {
  const booleans = ["bootloader_match", "partition_table_match", "otadata_match", "partition_table_valid", "selected_partition_bundle_match", "selected_app_digest_match", "selected_app_header_valid", "selected_app_identity_match", "physical_identity_match", "cleanup_complete"];
  const counts = ["rom_admission_count", "metadata_read_count", "selected_app_read_count", "application_exit_count"];
  if (machine["schema_version"] !== "bitaxe-native-usb-boot-chain-private-v1" || booleans.some(key => typeof machine[key] !== "boolean") || counts.some(key => !Number.isInteger(machine[key]) || Number(machine[key]) < 0 || Number(machine[key]) > 3) || machine["device_write_observed"] !== false || machine["host_network_effect"] !== false || machine["redaction_status"] !== "passed" || !["boot_chain_exact", "boot_chain_mismatch"].includes(String(machine["terminal_category"])) || !/^[0-9a-f]{64}$/u.test(evaluator)) fail("finalization");
  const keys = ["source_commit", "reference_commit", "plan_sha256", "manifest_sha256", "restore_bundle_sha256", "display_category", ...booleans, "ota_selection_category", "selected_partition_category", ...counts, "device_write_observed", "host_network_effect", "terminal_category", "redaction_status"];
  const result: JsonObject = { schema_version: "bitaxe-native-usb-boot-chain-projection-v1", evaluator_sha256: evaluator }; for (const key of keys) result[key] = machine[key]; return result;
}

export async function runBootChain(workspace: string, args: Args): Promise<JsonObject> {
  if (args.action === "preflight") { await admission(workspace, true); return { schema_version: "bitaxe-native-usb-boot-chain-preflight-v1", status: "ready", device_effect: false }; }
  if (args.action === "capture") { await admission(workspace, true); await mkdir(path.join(workspace, root), { recursive: true, mode: 0o700 }); await chmod(path.join(workspace, root), 0o700); const intent = path.join(workspace, root, "display-intent.private.json"); const result = path.join(workspace, root, "display-capture.private.json"); await privateWrite(intent, { operation: "prompt" }); await runHelper(workspace, "tools/automation/src/macos-native-usb-boot-chain-display.swift", intent, result); return { schema_version: "bitaxe-native-usb-boot-chain-capture-v1", status: "accepted" }; }
  await admission(workspace, false);
  if (args.action === "read") { await mode(path.join(workspace, root, "display-capture.private.json"), 0o600); if (!(await absent(path.join(workspace, root, "machine-result.private.json")))) fail("consumed"); const intent = path.join(workspace, root, "manual-intent.private.json"); const checkpoint = path.join(workspace, root, "manual-bootstrap.private.json"); await privateWrite(intent, { schema_version: "bitaxe-native-usb-owner-recovery-prompt-v1", operation: "prompt" }); await runHelper(workspace, "tools/automation/src/macos-native-usb-owner-recovery.swift", intent, checkpoint); const child = await runCampaignProcess(workspace, path.join(workspace, "bazel-bin/tools/flash/flash"), ["boot-chain-readback", "--board", "205", "--port", args.port, "--package-manifest", manifest, "--restore-bundle", bundle, "--private-root", root, "--plan", plan, "--manual-checkpoint", path.relative(workspace, checkpoint), "--redact-evidence"], 900000); if (child.exitCode !== 0) fail("readback_failed"); const machine = object(JSON.parse(await readFile(path.join(workspace, root, "machine-result.private.json"), "utf8"))); return { schema_version: "bitaxe-native-usb-boot-chain-read-v1", status: "accepted", terminal_category: machine["terminal_category"] }; }
  const machine = object(JSON.parse(await readFile(path.join(workspace, root, "machine-result.private.json"), "utf8"))); const sources = await Promise.all(["tools/flash/src/boot_chain.rs", "tools/flash/src/environment/boot_chain.rs", "tools/automation/src/native-usb-boot-chain-integrity.ts", "tools/automation/src/macos-native-usb-boot-chain-display.swift"].map(async candidate => ({ path: candidate, source: await readFile(path.join(workspace, candidate), "utf8") }))); const publicValue = projectBootChain(machine, sha256(JSON.stringify(sources))); const target = path.join(workspace, projection); await mkdir(path.dirname(target), { recursive: true }); await writeFile(target, `${JSON.stringify(publicValue, null, 2)}\n`, { flag: "wx", mode: 0o644 }); await chmod(target, 0o644); return { schema_version: "bitaxe-native-usb-boot-chain-finalize-v1", status: "accepted", terminal_category: publicValue["terminal_category"] };
}
