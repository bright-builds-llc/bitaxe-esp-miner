import { fstatSync } from "node:fs";
import { mkdir, readFile, stat } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { once } from "node:events";
import { cleanPushed, git, ignored, missing, protectedPath } from "../fixed-usb-qualification/contract.mjs";
import { inventory, privateRoot, proof, verifyInventory, writeNew } from "../str005-noise-serial/files.mjs";
import { processSnapshot, requireLsofAbsent, requireNoHolders, sameProcess } from "./host-resources.mjs";
import { createRecoveryServer, recoveryAssets } from "./recovery-server.mjs";
import { recoveryConclusion, validateRecoveryParts } from "./recovery-evidence.mjs";
import { check, sha256 } from "./values.mjs";
import { validateRecoveryDetector } from "./recovery-admission.mjs";

const task = "task-str005-failure-recovery-accounting";
const oldSeal = "14d2122208b2040f1482074c77328cd3a59c651e45bc17e7be7af2648d8f5950";
const oldContext = "a453de1753acc78bfa3eaeebfd9f42339528ec4fb41704fd50d390a1cd1ff5c4";
const contract = "docs/hardware/str005-failure-recovery-accounting.md";

async function source(firmwareRoot) {
  const commit = git(firmwareRoot, ["rev-parse", "HEAD"]); cleanPushed(firmwareRoot, commit);
  const tasks = await readFile(resolve(firmwareRoot, "TASKS.md"), "utf8");
  const active = tasks.split("## Active\n")[1]?.split(/^## /mu)[0] ?? "";
  check(active.includes(`### ${task} |`) && tasks.split(`### ${task} |`).length === 2, "recovery_task_inactive");
  return { commit, contractSha256: sha256(await readFile(resolve(firmwareRoot, contract))) };
}
async function predecessor(firmwareRoot) {
  const oldRoot = resolve(firmwareRoot, "scratch/str005-v2-serial/share-002");
  const sealed = await proof(oldRoot, "sealed-inventory.json");
  check(sealed.sha256 === oldSeal, "recovery_predecessor_seal");
  await verifyInventory(oldRoot, sealed.value.files, new Set(["sealed-inventory.json", "projection.json"]));
  const wrapped = (await proof(oldRoot, "context.json")).value, context = wrapped.context;
  check(wrapped.sha256 === oldContext && sha256(JSON.stringify(context)) === oldContext, "recovery_predecessor_context");
  const assets = await recoveryAssets(firmwareRoot, oldRoot, context);
  check(sha256(assets.page) === context.gate_page_sha256 && sha256(assets.bundle) === context.gate_bundle_sha256 &&
    sha256(JSON.stringify(assets.trust)) === context.trust_sha256, "recovery_asset_identity");
  cleanPushed(context.gate_root, context.gate_commit);
  return { context, assets, oldRoot };
}
export async function main(argv) {
  const [action, flag, path, ...extra] = argv;
  check(["preflight", "serve", "finish"].includes(action) && flag === "--private-root" &&
    typeof path === "string" && resolve(path) === path && extra.length === 0, "recovery_arguments");
  const firmwareRoot = process.env.BUILD_WORKSPACE_DIRECTORY ?? git(process.cwd(), ["rev-parse", "--show-toplevel"]);
  const root = resolve(path); ignored(firmwareRoot, root);
  if (action === "preflight") {
    await missing(root); await privateRoot(dirname(root));
    const published = await source(firmwareRoot), { context } = await predecessor(firmwareRoot);
    await mkdir(root, { mode: 0o700 });
    await writeNew(resolve(root, "context.json"), { schema: "str005-failure-recovery-v1", ...published,
      predecessorSeal: oldSeal, predecessorContext: oldContext, firmwareCommit: context.firmware_commit,
      gateCommit: context.gate_commit, appElfSha256: context.app_elf_sha256 });
    return { preflight: "passed", device_effects: false };
  }
  await privateRoot(root); await missing(resolve(root, "sealed-inventory.json"));
  const bound = (await proof(root, "context.json")).value;
  check(bound.schema === "str005-failure-recovery-v1" && bound.predecessorSeal === oldSeal && bound.predecessorContext === oldContext,
    "recovery_context");
  const published = await source(firmwareRoot);
  check(bound.commit === published.commit && bound.contractSha256 === published.contractSha256, "recovery_source_changed");
  const { context, assets, oldRoot } = await predecessor(firmwareRoot);
  if (action === "finish") return finish(root, context);
  for (const fd of [1, 2]) check(fstatSync(fd).isFile() && (fstatSync(fd).mode & 0o777) === 0o600, "recovery_protected_output");
  check(fstatSync(1).ino !== fstatSync(2).ino || fstatSync(1).dev !== fstatSync(2).dev, "recovery_distinct_output");
  await missing(resolve(root, "server-owner.json"));
  const detectorPath = resolve(dirname(root), "detector.stdout.log"); await protectedPath(detectorPath);
  const detected = await readFile(detectorPath, "utf8"), age = Date.now() - (await stat(detectorPath)).mtimeMs;
  const previous = (await proof(oldRoot, "install-4.claim.json")).value.detector;
  validateRecoveryDetector(detected, previous, age); requireNoHolders(previous.port);
  const server = createRecoveryServer({ root, context, ...assets });
  const done = new Promise(resolveDone => server.once("close", resolveDone));
  let maybeRelease;
  const stop = () => { maybeRelease ??= server.release(); };
  for (const signal of ["SIGINT", "SIGTERM"]) process.once(signal, stop);
  try {
    server.listen(0, "127.0.0.1"); await once(server, "listening");
    const owner = (await processSnapshot()).find(row => row.pid === process.pid);
    check(owner, "recovery_owner_missing");
    await writeNew(resolve(root, "server-owner.json"), { owner, port: server.address().port, serialPort: previous.port,
      detectorSha256: sha256(detected), startedAtUnixMs: Date.now() });
    process.stdout.write(`recovery_url=http://127.0.0.1:${server.address().port}/\n`);
    await done;
  } finally {
    stop(); await maybeRelease;
    for (const signal of ["SIGINT", "SIGTERM"]) process.removeListener(signal, stop);
  }
  return { server_released: true };
}
async function finish(root, context) {
  const server = (await proof(root, "server-owner.json")).value;
  const current = await processSnapshot();
  check(!current.some(row => sameProcess(row, server.owner) || row.ppid === server.owner.pid), "recovery_server_live");
  requireLsofAbsent(["-nP", `-iTCP:${server.port}`, "-sTCP:LISTEN", "-t"]); requireNoHolders(server.serialPort);
  const parts = {};
  for (const name of ["ledger", "original_budget", "diagnostics", "status", "state", "closed", "finished"]) {
    try { parts[name] = (await proof(root, `${name}.json`)).value; }
    catch (error) { if (error.code !== "ENOENT") throw error; }
  }
  validateRecoveryParts(parts, context);
  const result = { ...recoveryConclusion(parts), host_resources_released: true, predecessor_unchanged: true,
    firmwareCommit: context.firmware_commit, gateCommit: context.gate_commit };
  await writeNew(resolve(root, "result.json"), result);
  await writeNew(resolve(root, "sealed-inventory.json"), { files: await inventory(root) });
  return result;
}
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main(process.argv.slice(2)).then(value => process.stdout.write(`${JSON.stringify(value)}\n`)).catch(error => {
    const code = typeof error.code === "string" && /^(?:recovery|noise)_[a-z_]+$/u.test(error.code) ? error.code : "recovery_operation_rejected";
    process.stdout.write(`${JSON.stringify({ error: code })}\n`); process.exitCode = 1;
  });
}
