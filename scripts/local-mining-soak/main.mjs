// `just local-mining-soak preflight|run|pause`: the first local mining soak (ADR-0038 task 6), gated
// by an exact line in the active task. `pause` is the recovery action: it pauses the device and
// confirms the ASIC powered down, for a run that died. The station endpoint comes from a completed
// `just otawww-endpoint` handoff in the same session; the pool from the ignored
// pool-credentials file, used at run time only.
import { mkdir, open, readFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { git, ignored, missing, protectedPath, requireCondition } from "../fixed-usb-qualification/contract.mjs";
import { readPoolForSigning } from "../fixed-usb-qualification/authority.mjs";
import { privateRoot, proof, writeNew } from "../str005-noise-serial/files.mjs";
import { deviceClient } from "./device.mjs";
import { poolSettings } from "./pool.mjs";
import { pauseAndConfirm, runSoak } from "./run.mjs";

export const SOAK_TASK = "task-local-mining-evidence-soaks";
export const SOAK_TASK_LINE = "Ultra 205 local mining soak 001 hardware: enabled.";
const CONTEXT_SCHEMA = "local-mining-soak-context-v1";
/** The endpoint must come from this session: a handoff at most this old. */
const ENDPOINT_MAXIMUM_AGE_MS = 30 * 60_000;
const ACTIONS = { preflight: ["--private-root", "--endpoint-root", "--package-manifest", "--pool-credentials"],
  run: ["--private-root", "--pool-credentials"], pause: ["--private-root"] };

function argumentsFor(argv) {
  const [action, ...rest] = argv; requireCondition(Object.hasOwn(ACTIONS, action), "soak_action");
  const options = {};
  for (let index = 0; index < rest.length; index += 2) {
    requireCondition(ACTIONS[action].includes(rest[index]) && typeof rest[index + 1] === "string" &&
      !Object.hasOwn(options, rest[index]), "soak_option");
    options[rest[index]] = rest[index + 1];
  }
  requireCondition(ACTIONS[action].every((key) => Object.hasOwn(options, key)), "soak_option");
  return { action, options };
}

/** The soak task must be active exactly once and carry the exact enable line in its own block. */
export function requireSoakTask(tasksText) {
  let section = "", inBlock = false, count = 0, enabled = false;
  for (const line of tasksText.split(/\r?\n/u)) {
    if (line.startsWith("## ")) { section = line; inBlock = false; }
    if (line.startsWith("### ")) {
      inBlock = section === "## Active" && line.slice(4).split(/\s/u)[0] === SOAK_TASK;
      if (inBlock) count += 1;
    }
    if (inBlock && line.trim() === SOAK_TASK_LINE) enabled = true;
  }
  requireCondition(count === 1, "soak_task_ambiguous");
  requireCondition(enabled, "soak_task_disabled");
}

async function preflight(firmwareRoot, root, options) {
  requireSoakTask(await readFile(resolve(firmwareRoot, "TASKS.md"), "utf8"));
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
  await readPoolForSigning(firmwareRoot, resolve(firmwareRoot, options["--pool-credentials"]));
  await mkdir(root, { mode: 0o700 });
  await writeNew(resolve(root, "context.json"), { schema: CONTEXT_SCHEMA, firmware_commit: manifest.source_commit,
    app_elf_sha256: manifest.app_elf_sha256, endpoint_root: endpointRoot, boot_ordinal: endpoint.bootOrdinal });
  return { preflight: "passed", device_effects: false };
}

async function run(firmwareRoot, root, options) {
  requireSoakTask(await readFile(resolve(firmwareRoot, "TASKS.md"), "utf8"));
  await privateRoot(root);
  const context = (await proof(root, "context.json")).value; requireCondition(context.schema === CONTEXT_SCHEMA, "soak_context");
  await missing(resolve(root, "result.json"));
  const endpoint = (await proof(context.endpoint_root, "endpoint.private.json")).value;
  const pool = await readPoolForSigning(firmwareRoot, resolve(firmwareRoot, options["--pool-credentials"]));
  const journal = await open(resolve(root, "journal.ndjson"), "wx", 0o600);
  let interrupted = false;
  // Every signal only asks the loop to stop; the loop then pauses the device.
  const interrupt = () => { interrupted = true; };
  for (const signal of ["SIGINT", "SIGTERM", "SIGHUP"]) process.on(signal, interrupt);
  try {
    const verdict = await runSoak({
      client: deviceClient(endpoint), settings: poolSettings(pool),
      expected: { firmwareCommit: context.firmware_commit, appElfSha256: context.app_elf_sha256, bootOrdinal: context.boot_ordinal },
      now: Date.now, sleep: (ms) => new Promise((done) => setTimeout(done, ms)),
      record: (event) => journal.appendFile(`${JSON.stringify(event)}\n`), isInterrupted: () => interrupted,
    });
    await writeNew(resolve(root, "result.json"), verdict);
    return verdict;
  } finally {
    await journal.close();
    for (const signal of ["SIGINT", "SIGTERM", "SIGHUP"]) process.removeListener(signal, interrupt);
  }
}

/** Recovery: pause the device a run left mining, and confirm the ASIC powered down. */
async function pause(root) {
  await privateRoot(root);
  const context = (await proof(root, "context.json")).value; requireCondition(context.schema === CONTEXT_SCHEMA, "soak_context");
  const endpoint = (await proof(context.endpoint_root, "endpoint.private.json")).value;
  const events = [];
  const maybeFinal = await pauseAndConfirm({ client: deviceClient(endpoint), now: Date.now,
    sleep: (ms) => new Promise((done) => setTimeout(done, ms)), note: async (event) => { events.push(event.event); } });
  const outcome = { schema: "local-mining-soak-recovery-v1", paused: events.includes("paused"), asic_stopped: maybeFinal !== undefined };
  await writeNew(resolve(root, `recovery-${Date.now()}.json`), outcome);
  requireCondition(outcome.asic_stopped, "soak_recovery_unconfirmed");
  return outcome;
}

export async function main(argv) {
  const { action, options } = argumentsFor(argv);
  const firmwareRoot = process.env.BUILD_WORKSPACE_DIRECTORY ?? git(process.cwd(), ["rev-parse", "--show-toplevel"]);
  const root = resolve(firmwareRoot, options["--private-root"]);
  if (action === "preflight") return preflight(firmwareRoot, root, options);
  return action === "run" ? run(firmwareRoot, root, options) : pause(root);
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) main(process.argv.slice(2)).then((value) => {
  process.stdout.write(`${JSON.stringify(value)}\n`);
}).catch((error) => {
  process.stdout.write(`${JSON.stringify({ error: typeof error.code === "string" ? error.code : "soak_failed" })}\n`);
  process.exitCode = 1;
});
