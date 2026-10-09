// `just bwg-restoration preflight|serve|finish|publish`: one task-gated BWG-007 serial restoration attempt
// (firmware ADR-0035). Usage: scripts/bwg-restoration/README.md. Effects need the active task's enable line.
import { fstatSync } from "node:fs";
import { mkdir, readFile, stat } from "node:fs/promises";
import { basename, dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { parseArgs } from "node:util";
import { digest, protectedPath, QualificationError, readJson, requireCondition, within, writeNew } from "../fixed-usb-qualification/contract.mjs";
import { parseDetector } from "../hardware-operator/detector.mjs";
import { inventory, verifyInventory } from "../str005-noise-serial/files.mjs";
import { processSnapshot, requireGone, requireLsofAbsent, requireNoHolders } from "../str005-noise-serial/host-resources.mjs";
import { RESTORATION_PORT, RESULT_SCHEMA, SCENARIO_RESULT_SCHEMA, SCENARIOS } from "./contract.mjs";
import { loadRestorationContext, preflight } from "./preflight.mjs";
import { buildProjection, PROJECTION_DIRECTORY, publishProjectionSet } from "./projection.mjs";
import { campaignResult, poolScanValues, scanAttemptRoot, sealedResult } from "./seal.mjs";
import { createRestorationSupervisor } from "./server.mjs";

const string = { type: "string" };
const FINAL_DETECTOR_FRESH_MS = 300000;
const ACTIONS = {
  preflight: { options: { "private-root": string, "firmware-root": string, "gate-root": string, "firmware-commit": string, "gate-commit": string,
    manifest: string, "authority-directory": string, "pool-credentials": string, detector: string, bun: string },
  required: ["private-root", "firmware-root", "gate-root", "firmware-commit", "gate-commit", "manifest", "authority-directory", "pool-credentials", "detector"] },
  serve: { options: { "private-root": string, "authority-directory": string, "pool-credentials": string, bun: string },
    required: ["private-root", "authority-directory", "pool-credentials"] },
  finish: { options: { "private-root": string, "pool-credentials": string }, required: ["private-root", "pool-credentials"] },
  publish: { options: { "private-root": string }, required: ["private-root"] },
};

export function argumentsFor(argv) {
  const [action, ...rest] = argv;
  const spec = ACTIONS[action];
  requireCondition(spec !== undefined, "restoration_action");
  let values;
  try { ({ values } = parseArgs({ args: rest, options: spec.options, strict: true, allowPositionals: false })); } catch { throw new QualificationError("restoration_arguments"); }
  // Finish scans the attempt for the exact pool values, so it cannot seal without them (ADR-0036).
  if (action === "finish") requireCondition(values["pool-credentials"] !== undefined, "restoration_finish_pool_credentials_required");
  for (const name of spec.required) requireCondition(values[name] !== undefined, "restoration_arguments");
  const base = process.env.BUILD_WORKING_DIRECTORY ?? process.cwd();
  const path = (name) => values[name] === undefined ? undefined : resolve(base, values[name]);
  return { action, options: { privateRoot: path("private-root"), firmwareRoot: path("firmware-root"), gateRoot: path("gate-root"),
    firmwareCommit: values["firmware-commit"], gateCommit: values["gate-commit"], manifest: path("manifest"), authorityDirectory: path("authority-directory"),
    detector: path("detector"), poolCredentials: path("pool-credentials"), bun: values.bun ?? "bun" } };
}

const absentOrRefuse = (path, code) => protectedPath(path).then(() => requireCondition(false, code), (error) => { if (error.code !== "ENOENT") throw error; });

async function serve(options, operations = {}) {
  for (const fd of [1, 2]) requireCondition(fstatSync(fd).isFile() && (fstatSync(fd).mode & 0o777) === 0o600, "restoration_private_output");
  const context = await loadRestorationContext(options.privateRoot);
  await absentOrRefuse(resolve(options.privateRoot, "server-owner.json"), "restoration_server_used");
  requireLsofAbsent(["-nP", `-iTCP:${RESTORATION_PORT}`, "-sTCP:LISTEN", "-t"]);
  const server = await createRestorationSupervisor({ ...options, context }, operations);
  await new Promise((done, fail) => { server.once("error", fail); server.listen(RESTORATION_PORT, "127.0.0.1", done); });
  const owner = (await processSnapshot()).find((row) => row.pid === process.pid);
  requireCondition(owner, "restoration_owner");
  await writeNew(resolve(options.privateRoot, "server-owner.json"), { owner, port: RESTORATION_PORT });
  process.stdout.write(`restoration_url=http://127.0.0.1:${RESTORATION_PORT}/\n`);
  await new Promise((done) => { for (const signal of ["SIGINT", "SIGTERM"]) process.once(signal, done); });
  await server.closeRestorationResources();
  await new Promise((done) => server.close(done));
  return { server_released: true };
}

/**
 * Seal after release: owner group gone, port free, fresh same-device detector, no serial holder. Then scan every
 * attempt file for pool values and credential shapes and write `result.json` with only the scan counts.
 */
async function finish(options) {
  const root = options.privateRoot, context = await loadRestorationContext(root);
  await absentOrRefuse(resolve(root, "sealed-inventory.json"), "restoration_already_sealed");
  const server = await readJson(resolve(root, "server-owner.json")).catch(() => null);
  if (server) await requireGone([server.owner]);
  requireLsofAbsent(["-nP", `-iTCP:${RESTORATION_PORT}`, "-sTCP:LISTEN", "-t"]);
  const detectorPath = resolve(dirname(root), "final-detector.stdout.log");
  await protectedPath(detectorPath);
  requireCondition(Date.now() - (await stat(detectorPath)).mtimeMs <= FINAL_DETECTOR_FRESH_MS, "restoration_final_detector_stale");
  const device = parseDetector(await readFile(detectorPath, "utf8"));
  requireCondition(device.physical === context.physical_identity_sha256, "restoration_final_detector_identity");
  requireNoHolders(device.port);
  return seal(root, context, options.poolCredentials);
}

/** The pure-file part of finish: scan, write the sealed result, then the sealed inventory. */
export async function seal(root, context, poolCredentials) {
  await absentOrRefuse(resolve(root, "result.json"), "restoration_already_sealed");
  const values = await poolScanValues(context.firmware_root, poolCredentials);
  const campaign = await campaignResult(root, context);
  const result = sealedResult(campaign, await scanAttemptRoot(root, values));
  await writeNew(resolve(root, "result.json"), { result, sha256: digest(JSON.stringify(result)) });
  await writeNew(resolve(root, "sealed-inventory.json"), { files: await inventory(root) });
  return { restoration_sealed: true, result: result.result, failure: result.failure ?? null, credential_scan: result.credential_scan };
}

/** The records of one scenario, as written, digested in order. */
async function scenarioRecordsDigest(root, scenario) {
  const lines = (await readFile(resolve(root, "records.jsonl"), "utf8").catch(() => "")).split("\n").filter(Boolean);
  return digest(lines.filter((line) => JSON.parse(line).scenario === scenario).join("\n"));
}

/** All-or-nothing publication of eight passed scenarios from one sealed, unchanged root on one device. */
async function publish(options, operations = {}) {
  const root = options.privateRoot, context = await loadRestorationContext(root);
  const sealed = await readJson(resolve(root, "sealed-inventory.json"));
  await verifyInventory(root, sealed.files, new Set(["sealed-inventory.json"]));
  const final = (await readJson(resolve(root, "result.json"))).result;
  requireCondition(final.schema === RESULT_SCHEMA && final.result === "passed" && final.scenarios.length === SCENARIOS.length, "restoration_not_passed");
  requireCondition(Number.isSafeInteger(final.credential_scan?.files) && final.credential_scan.files > 0 && final.credential_scan.hits === 0,
    "restoration_credential_scan_missing");
  const contextSha256 = digest(JSON.stringify(context)), projections = [];
  for (const [index, scenario] of SCENARIOS.entries()) {
    const record = await readJson(resolve(root, `scenario-${String(index + 1).padStart(2, "0")}-${scenario}.json`));
    const value = record.value;
    requireCondition(record.sha256 === digest(JSON.stringify(value)) && value.schema === SCENARIO_RESULT_SCHEMA && value.scenario === scenario &&
      value.result === "passed" && value.context_sha256 === contextSha256 && value.physical_identity_sha256 === context.physical_identity_sha256,
    "restoration_scenario_identity");
    projections.push(buildProjection({ attemptId: `bwg007-${basename(root)}`, context, scenarioResult: value,
      recordsSha256: await scenarioRecordsDigest(root, scenario), scenarioResultSha256: record.sha256 }));
  }
  const directory = within(context.firmware_root, resolve(context.firmware_root, PROJECTION_DIRECTORY));
  await mkdir(directory, { recursive: true, mode: 0o755 });
  const targets = await publishProjectionSet(directory, projections, operations);
  return { restoration_published: true, projections: targets.length };
}

export async function main(argv, operations = {}) {
  const { action, options } = argumentsFor(argv);
  if (action === "preflight") return preflight(options, operations);
  if (action === "serve") return serve(options, operations);
  if (action === "finish") return finish(options);
  return publish(options, operations);
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main(process.argv.slice(2)).then((result) => { process.stdout.write(`${JSON.stringify(result)}\n`); }, (error) => {
    process.stderr.write(`restoration_failed=${error instanceof QualificationError || /^[a-z][a-z0-9_]*$/u.test(error?.code ?? "") ? error.code : "restoration_operation_failed"}\n`);
    process.exitCode = 1;
  });
}
