// `just ultra205-soak preflight|serve|finish`: one task-gated upstream-default soak (firmware ADR-0033).
// Usage: scripts/fixed-usb-soak/README.md. Effects need the active soak task's exact enable line.
import { fstatSync } from "node:fs";
import { readFile, stat } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { parseArgs } from "node:util";
import { digest, protectedPath, QualificationError, readJson, requireCondition, writeNew } from "../fixed-usb-qualification/contract.mjs";
import { inventory } from "../str005-noise-serial/files.mjs";
import { processSnapshot, requireGone, requireLsofAbsent, requireNoHolders } from "../str005-noise-serial/host-resources.mjs";
import { parseDetector } from "../hardware-operator/detector.mjs";
import { SOAK_PORT } from "./contract.mjs";
import { loadSoakContext, preflight } from "./preflight.mjs";
import { createSoakSupervisor } from "./server.mjs";

const string = { type: "string" };
const FINAL_DETECTOR_FRESH_MS = 300000;
const ACTIONS = {
  preflight: { options: { "private-root": string, "firmware-root": string, "gate-root": string, "firmware-commit": string, "gate-commit": string,
    manifest: string, "authority-directory": string, detector: string, bun: string },
  required: ["private-root", "firmware-root", "gate-root", "firmware-commit", "gate-commit", "manifest", "authority-directory", "detector"] },
  serve: { options: { "private-root": string, "authority-directory": string, "pool-credentials": string, bun: string }, required: ["private-root", "authority-directory", "pool-credentials"] },
  finish: { options: { "private-root": string }, required: ["private-root"] },
};

export function argumentsFor(argv) {
  const [action, ...rest] = argv;
  const spec = ACTIONS[action];
  requireCondition(spec !== undefined, "soak_action");
  let values;
  try { ({ values } = parseArgs({ args: rest, options: spec.options, strict: true, allowPositionals: false })); } catch { throw new QualificationError("soak_arguments"); }
  for (const name of spec.required) requireCondition(values[name] !== undefined, "soak_arguments");
  const base = process.env.BUILD_WORKING_DIRECTORY ?? process.cwd();
  const path = (name) => values[name] === undefined ? undefined : resolve(base, values[name]);
  return { action, options: { privateRoot: path("private-root"), firmwareRoot: path("firmware-root"), gateRoot: path("gate-root"),
    firmwareCommit: values["firmware-commit"], gateCommit: values["gate-commit"], manifest: path("manifest"), authorityDirectory: path("authority-directory"),
    detector: path("detector"), poolCredentials: path("pool-credentials"), bun: values.bun ?? "bun" } };
}

async function serve(options) {
  for (const fd of [1, 2]) requireCondition(fstatSync(fd).isFile() && (fstatSync(fd).mode & 0o777) === 0o600, "soak_private_output");
  const context = await loadSoakContext(options.privateRoot);
  await protectedPath(resolve(options.privateRoot, "server-owner.json")).then(() => requireCondition(false, "soak_server_used"), (error) => { if (error.code !== "ENOENT") throw error; });
  requireLsofAbsent(["-nP", `-iTCP:${SOAK_PORT}`, "-sTCP:LISTEN", "-t"]);
  const server = await createSoakSupervisor({ ...options, context });
  await new Promise((done, fail) => { server.once("error", fail); server.listen(SOAK_PORT, "127.0.0.1", done); });
  const owner = (await processSnapshot()).find((row) => row.pid === process.pid);
  requireCondition(owner, "soak_owner");
  await writeNew(resolve(options.privateRoot, "server-owner.json"), { owner, port: SOAK_PORT });
  process.stdout.write(`soak_url=http://127.0.0.1:${SOAK_PORT}/\n`);
  await new Promise((done) => { for (const signal of ["SIGINT", "SIGTERM"]) process.once(signal, done); });
  await server.closeSoakResources();
  await new Promise((done) => server.close(done));
  return { server_released: true };
}

/** Seal after release: owner group gone, port free, fresh same-device detector, no serial holder. */
async function finish(options) {
  const root = options.privateRoot, context = await loadSoakContext(root);
  await protectedPath(resolve(root, "sealed-inventory.json")).then(() => requireCondition(false, "soak_already_sealed"), (error) => { if (error.code !== "ENOENT") throw error; });
  const server = await readJson(resolve(root, "server-owner.json")).catch(() => null);
  if (server) await requireGone([server.owner]);
  requireLsofAbsent(["-nP", `-iTCP:${SOAK_PORT}`, "-sTCP:LISTEN", "-t"]);
  const detectorPath = resolve(dirname(root), "final-detector.stdout.log");
  await protectedPath(detectorPath);
  // `just` runs this through `bazel run`, which can take minutes when Bazel discards its analysis cache;
  // a 60 s bound left every seal stale. Release itself is proven above by the gone owner and free port.
  requireCondition(Date.now() - (await stat(detectorPath)).mtimeMs <= FINAL_DETECTOR_FRESH_MS, "soak_final_detector_stale");
  const device = parseDetector(await readFile(detectorPath, "utf8"));
  requireCondition(device.physical === context.physical_identity_sha256, "soak_final_detector_identity");
  requireNoHolders(device.port);
  const maybeResult = await readJson(resolve(root, "result.json")).catch(() => null);
  if (!maybeResult) {
    const result = { schema: "fixed-usb-soak-result-v1", result: "unverified", failures: ["completion_missing"], parity_promotion: false };
    await writeNew(resolve(root, "result.json"), { result, sha256: digest(JSON.stringify(result)) });
  }
  await writeNew(resolve(root, "sealed-inventory.json"), { files: await inventory(root) });
  const sealed = (await readJson(resolve(root, "result.json"))).result;
  return { soak_sealed: true, result: sealed.result, failures: sealed.failures ?? [] };
}

export async function main(argv) {
  const { action, options } = argumentsFor(argv);
  if (action === "preflight") return preflight(options);
  if (action === "serve") return serve(options);
  return finish(options);
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main(process.argv.slice(2)).then((result) => { process.stdout.write(`${JSON.stringify(result)}\n`); }, (error) => {
    process.stderr.write(`soak_failed=${error instanceof QualificationError || /^[a-z][a-z0-9_]*$/u.test(error?.code ?? "") ? error.code : "soak_operation_failed"}\n`);
    process.exitCode = 1;
  });
}
