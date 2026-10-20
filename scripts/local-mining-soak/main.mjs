// `just local-mining-soak preflight|run|pause`: the local mining soaks, the lease interruption
// check and the boot-resume check (ADR-0038), each gated by an exact line in the active task. `pause`
// is the recovery action for a run that died: it pauses the device, turns mining on boot off if it
// reads on, and confirms the ASIC powered down. The
// station endpoint comes from a completed `just otawww-endpoint` handoff in the same session; the
// pool from the ignored pool-credentials file, used at run time only. A lease check watches the
// `just ultra205-soak --interrupt-local-mining` private root for its result.
import { lstat, mkdir, open, readFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { git, ignored, missing, protectedPath, requireCondition } from "../fixed-usb-qualification/contract.mjs";
import { readPoolForSigning } from "../fixed-usb-qualification/authority.mjs";
import { privateRoot, proof, writeNew } from "../str005-noise-serial/files.mjs";
import { requireNoHolders } from "../str005-noise-serial/host-resources.mjs";
import { parseDetector } from "../hardware-operator/detector.mjs";
import { DETECTOR_FRESH_MS, stoppedProof } from "./stopped-proof.mjs";
import { deviceClient } from "./device.mjs";
import { poolSettings } from "./pool.mjs";
import { runSoak } from "./run.mjs";
import { pauseThenRestore, runBootResume } from "./boot-resume.mjs";
import { restoreAndStop, runOverheat, settingsOf } from "./overheat.mjs";
import { SOAKS } from "./judge.mjs";

export const SOAK_TASK = "task-local-mining-evidence-soaks";
/** The task that owns a soak: its own `task`, or the evidence soaks' task. */
export const soakTask = (soakId) => SOAKS[soakId]?.task ?? SOAK_TASK;
/** Each soak has its own enable line in the active task. */
export const soakTaskLine = (soakId) => SOAKS[soakId]?.enableLine ?? `Ultra 205 local mining soak ${soakId} hardware: enabled.`;
const CONTEXT_SCHEMA = "local-mining-soak-context-v1";
/** The endpoint must come from this session: a handoff at most this old. */
const ENDPOINT_MAXIMUM_AGE_MS = 30 * 60_000;
const ACTIONS = { preflight: ["--private-root", "--endpoint-root", "--package-manifest", "--pool-credentials", "--soak"],
  run: ["--private-root", "--pool-credentials"], pause: ["--private-root"],
  "stopped-proof": ["--private-root", "--endpoint-root", "--installed-manifest", "--detector"] };
/** Options an action accepts but does not require: a lease check names the Gate soak's private root. */
const OPTIONAL = { preflight: ["--lease-root"], run: [], pause: [], "stopped-proof": [] };

function argumentsFor(argv) {
  const [action, ...rest] = argv; requireCondition(Object.hasOwn(ACTIONS, action), "soak_action");
  const options = {};
  for (let index = 0; index < rest.length; index += 2) {
    requireCondition([...ACTIONS[action], ...OPTIONAL[action]].includes(rest[index]) && typeof rest[index + 1] === "string" &&
      !Object.hasOwn(options, rest[index]), "soak_option");
    options[rest[index]] = rest[index + 1];
  }
  requireCondition(ACTIONS[action].every((key) => Object.hasOwn(options, key)), "soak_option");
  return { action, options };
}

/** A test-only firmware runs only the check made for it, and that check only on it (ADR-0040). */
export function requireFirmwareVariant(manifest, soak) {
  requireCondition((manifest.firmware_variant ?? "production") === (soak.firmwareVariant ?? "production"), "soak_firmware_variant");
}

/** The soak task must be active exactly once and carry the exact enable line in its own block. */
export function requireSoakTask(tasksText, soakId) {
  const enableLine = soakTaskLine(soakId), task = soakTask(soakId);
  let section = "", inBlock = false, count = 0, enabled = false;
  for (const line of tasksText.split(/\r?\n/u)) {
    if (line.startsWith("## ")) { section = line; inBlock = false; }
    if (line.startsWith("### ")) {
      inBlock = section === "## Active" && line.slice(4).split(/\s/u)[0] === task;
      if (inBlock) count += 1;
    }
    if (inBlock && line.trim() === enableLine) enabled = true;
  }
  requireCondition(count === 1, "soak_task_ambiguous");
  requireCondition(enabled, "soak_task_disabled");
}

async function preflight(firmwareRoot, root, options) {
  const soakId = options["--soak"];
  requireCondition(Object.hasOwn(SOAKS, soakId), "soak_unknown");
  requireSoakTask(await readFile(resolve(firmwareRoot, "TASKS.md"), "utf8"), soakId);
  const leases = SOAKS[soakId].kind === "lease";
  requireCondition(leases === Object.hasOwn(options, "--lease-root"), "soak_lease_root");
  // The Gate soak's root must not exist yet, so only a result written after this preflight ends the lease phase.
  const maybeLeaseRoot = leases ? resolve(firmwareRoot, options["--lease-root"]) : null;
  if (maybeLeaseRoot !== null) { await missing(maybeLeaseRoot); ignored(firmwareRoot, maybeLeaseRoot); }
  await missing(root); await protectedPath(dirname(root), true); ignored(firmwareRoot, root);
  const endpointRoot = resolve(firmwareRoot, options["--endpoint-root"]);
  await privateRoot(endpointRoot);
  const handoff = (await proof(endpointRoot, "result.json")).value;
  requireCondition(handoff.schema === "otawww-endpoint-result-v1" && handoff.endpoint_recorded === true &&
    handoff.page_closed === true && handoff.host_released === true, "soak_endpoint_handoff");
  const endpoint = (await proof(endpointRoot, "endpoint.private.json")).value;
  requireCondition(Date.now() - endpoint.hostReceivedAtMs <= ENDPOINT_MAXIMUM_AGE_MS, "soak_endpoint_stale");
  const manifest = JSON.parse(await readFile(resolve(firmwareRoot, options["--package-manifest"]), "utf8"));
  requireCondition(/^[0-9a-f]{40}$/u.test(manifest.source_commit) && /^[0-9a-f]{64}$/u.test(manifest.app_elf_sha256) &&
    manifest.build_identity?.source_dirty === false, "soak_package");
  requireCondition(git(firmwareRoot, ["rev-parse", "HEAD"]).trim() === manifest.source_commit, "soak_package_not_head");
  requireFirmwareVariant(manifest, SOAKS[soakId]);
  await readPoolForSigning(firmwareRoot, resolve(firmwareRoot, options["--pool-credentials"]));
  await mkdir(root, { mode: 0o700 });
  await writeNew(resolve(root, "context.json"), { schema: CONTEXT_SCHEMA, soak: soakId, firmware_commit: manifest.source_commit,
    app_elf_sha256: manifest.app_elf_sha256, endpoint_root: endpointRoot, boot_ordinal: endpoint.bootOrdinal,
    ...(maybeLeaseRoot === null ? {} : { lease_root: maybeLeaseRoot }) });
  return { preflight: "passed", device_effects: false };
}

async function run(firmwareRoot, root, options) {
  await privateRoot(root);
  const context = (await proof(root, "context.json")).value; requireCondition(context.schema === CONTEXT_SCHEMA, "soak_context");
  requireCondition(Object.hasOwn(SOAKS, context.soak), "soak_unknown");
  requireSoakTask(await readFile(resolve(firmwareRoot, "TASKS.md"), "utf8"), context.soak);
  await missing(resolve(root, "result.json"));
  const leases = SOAKS[context.soak].kind === "lease";
  requireCondition(!leases || typeof context.lease_root === "string", "soak_lease_root");
  if (leases) await missing(resolve(context.lease_root, "result.json"));
  const endpoint = (await proof(context.endpoint_root, "endpoint.private.json")).value;
  const pool = await readPoolForSigning(firmwareRoot, resolve(firmwareRoot, options["--pool-credentials"]));
  const journal = await open(resolve(root, "journal.ndjson"), "wx", 0o600);
  let interrupted = false;
  // Every signal only asks the loop to stop; the loop then pauses the device.
  const interrupt = () => { interrupted = true; };
  for (const signal of ["SIGINT", "SIGTERM", "SIGHUP"]) process.on(signal, interrupt);
  try {
    const kind = SOAKS[context.soak].kind;
    const runner = kind === "boot-resume" ? runBootResume : kind === "overheat" ? runOverheat : runSoak;
    const verdict = await runner({
      client: deviceClient(endpoint), settings: poolSettings(pool),
      expected: { firmwareCommit: context.firmware_commit, appElfSha256: context.app_elf_sha256, bootOrdinal: context.boot_ordinal },
      // Monotonic, so a wall-clock correction never stretches the run deadline.
      now: () => Math.round(performance.now()), sleep: (ms) => new Promise((done) => setTimeout(done, ms)),
      record: (event) => journal.appendFile(`${JSON.stringify(event)}\n`), isInterrupted: () => interrupted,
      soak: SOAKS[context.soak], ...(leases ? { leaseResult: () => readLeaseResult(context.lease_root) } : {}),
      // The raw retained log can hold the station address: mode 0600 in the private root only.
      keepLogs: (text, capture) => keepPrivateText(resolve(root, `break-log-${String(capture).padStart(3, "0")}.private.txt`), text),
    });
    await writeNew(resolve(root, "result.json"), verdict);
    return verdict;
  } finally {
    await journal.close();
    for (const signal of ["SIGINT", "SIGTERM", "SIGHUP"]) process.removeListener(signal, interrupt);
  }
}

/**
 * The run's journaled baseline sample, or null when it never admitted one: such a run changed no
 * setting, because the baseline is journaled before any write.
 */
export async function maybeJournaledBaseline(root) {
  let text;
  try { text = await readFile(resolve(root, "journal.ndjson"), "utf8"); } catch { return null; }
  return baselineOfJournal(text);
}

/** The baseline sample of journal text; a torn last line, from a run that died mid-append, is skipped. */
export function baselineOfJournal(text) {
  for (const line of text.split("\n").filter(Boolean)) {
    let event;
    try { event = JSON.parse(line); } catch { continue; }
    if (event.event === "baseline") return event.sample;
  }
  return null;
}

/** Raw text in a new mode-0600 file; it never overwrites. */
async function keepPrivateText(path, text) {
  const file = await open(path, "wx", 0o600);
  try { await file.writeFile(text); await file.sync(); } finally { await file.close(); }
}

/**
 * The Gate soak's verdict once its `result.json` exists, else null: only the result and its closed
 * failure categories. A file still being written reads as absent until the next sample.
 */
export async function readLeaseResult(leaseRoot) {
  let value;
  try { value = JSON.parse(await readFile(resolve(leaseRoot, "result.json"), "utf8")); } catch { return null; }
  const verdict = value?.result;
  if (typeof verdict?.result !== "string" || !Array.isArray(verdict.failures)) return null;
  return { result: verdict.result, failures: verdict.failures.filter((failure) => /^[a-z0-9_]{1,80}$/u.test(failure)) };
}

/**
 * The proof that admits a local mining core-dump read: a new private root holding
 * `stopped-proof.json`. The endpoint comes from a same-session handoff; the installed firmware from
 * its package manifest; the physical identity from a fresh named-port detector.
 */
async function writeStoppedProof(firmwareRoot, root, options) {
  await missing(root); await protectedPath(dirname(root), true); ignored(firmwareRoot, root);
  const endpointRoot = resolve(firmwareRoot, options["--endpoint-root"]);
  await privateRoot(endpointRoot);
  const handoff = (await proof(endpointRoot, "result.json")).value;
  requireCondition(handoff.schema === "otawww-endpoint-result-v1" && handoff.endpoint_recorded === true &&
    handoff.page_closed === true && handoff.host_released === true, "soak_endpoint_handoff");
  const endpoint = (await proof(endpointRoot, "endpoint.private.json")).value;
  requireCondition(Date.now() - endpoint.hostReceivedAtMs <= ENDPOINT_MAXIMUM_AGE_MS, "soak_endpoint_stale");
  const manifest = JSON.parse(await readFile(resolve(firmwareRoot, options["--installed-manifest"]), "utf8"));
  requireCondition(/^[0-9a-f]{40}$/u.test(manifest.source_commit) && /^[0-9a-f]{64}$/u.test(manifest.app_elf_sha256) &&
    manifest.build_identity?.source_dirty === false, "soak_package");
  const detectorPath = resolve(firmwareRoot, options["--detector"]);
  await protectedPath(detectorPath);
  requireCondition(Date.now() - (await lstat(detectorPath)).mtimeMs <= DETECTOR_FRESH_MS, "stopped_proof_detector_stale");
  const value = await stoppedProof({ client: deviceClient(endpoint),
    installed: { firmwareCommit: manifest.source_commit, appElfSha256: manifest.app_elf_sha256 }, bootOrdinal: endpoint.bootOrdinal,
    detector: parseDetector(await readFile(detectorPath, "utf8")), sourceCommit: git(firmwareRoot, ["rev-parse", "HEAD"]).trim(),
    requireReleased: requireNoHolders, sleep: (ms) => new Promise((done) => setTimeout(done, ms)), now: Date.now });
  await mkdir(root, { mode: 0o700 });
  await writeNew(resolve(root, "stopped-proof.json"), value);
  return { stopped_proof: "written", device_effects: false };
}

/**
 * Recovery: pause the device a run left mining, turn mining on boot off only if it reads on (the
 * boot-resume check may have left it on), and confirm the ASIC powered down.
 */
async function pause(root) {
  await privateRoot(root);
  const context = (await proof(root, "context.json")).value; requireCondition(context.schema === CONTEXT_SCHEMA, "soak_context");
  const endpoint = (await proof(context.endpoint_root, "endpoint.private.json")).value;
  const events = [];
  const recovery = { client: deviceClient(endpoint), now: Date.now, sleep: (ms) => new Promise((done) => setTimeout(done, ms)),
    note: async (event) => { events.push(event.event); } };
  let { mineOnBootOff, maybeFinal } = await pauseThenRestore(recovery);
  // An overheat check that died may have left its raised pair, the fan at 100% or a device pause:
  // restore the settings its journaled baseline recorded.
  const maybeBaseline = SOAKS[context.soak]?.kind === "overheat" ? await maybeJournaledBaseline(root) : null;
  const overheat = maybeBaseline !== null;
  let settled = true;
  if (overheat) ({ settled, maybeFinal } = await restoreAndStop({ ...recovery, original: settingsOf(maybeBaseline) }));
  const outcome = { schema: "local-mining-soak-recovery-v1", mine_on_boot_off: mineOnBootOff, paused: events.includes("paused"),
    asic_stopped: maybeFinal !== undefined,
    ...(overheat ? { recovery_settled: settled, settings_restored: events.includes("settings_restored") } : {}) };
  await writeNew(resolve(root, `recovery-${Date.now()}.json`), outcome);
  requireCondition(outcome.mine_on_boot_off && outcome.asic_stopped && outcome.recovery_settled !== false &&
    outcome.settings_restored !== false, "soak_recovery_unconfirmed");
  return outcome;
}

export async function main(argv) {
  const { action, options } = argumentsFor(argv);
  const firmwareRoot = process.env.BUILD_WORKSPACE_DIRECTORY ?? git(process.cwd(), ["rev-parse", "--show-toplevel"]);
  const root = resolve(firmwareRoot, options["--private-root"]);
  if (action === "preflight") return preflight(firmwareRoot, root, options);
  if (action === "stopped-proof") return writeStoppedProof(firmwareRoot, root, options);
  return action === "run" ? run(firmwareRoot, root, options) : pause(root);
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) main(process.argv.slice(2)).then((value) => {
  process.stdout.write(`${JSON.stringify(value)}\n`);
}).catch((error) => {
  process.stdout.write(`${JSON.stringify({ error: typeof error.code === "string" ? error.code : "soak_failed" })}\n`);
  process.exitCode = 1;
});
