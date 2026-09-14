#!/usr/bin/env node
import { CADENCE_SCHEMA } from "./cadence-contract.mjs";
import { cadencePreflight } from "./cadence-preflight.mjs";
import { resetOriginPreflight } from "./reset-origin-context.mjs";
import { createResetOriginSupervisor } from "./reset-origin-server.mjs";
import { judgeResetOrigin, readResetOrigin } from "./reset-origin-judge.mjs";
import { startupRecoveryPreflight, createStartupRecoverySupervisor, judgeStartupRecovery,
  readStartupRecovery, consumeStartupRecoveryInstall } from "./cadence-startup-recovery.mjs";
import { closeUnissued } from "./cadence-unissued.mjs";
import { closePremining, reviewPremining } from "./cadence-premining.mjs";
import { noMiningPreflight, NO_MINING_SCHEMA } from "./no-mining-context.mjs";
import { createNoMiningSupervisor } from "./no-mining-server.mjs";
import { finishNoMining } from "./no-mining-judge.mjs";
import { recoverSampleSeal } from "./sample-seal.mjs";
import { once } from "node:events";
import { fstatSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { preflight, loadContext } from "./preflight.mjs";
import { iterativeBootstrap, iterativePreflight, recoveryPreflight } from "./iterative-preflight.mjs";
import { createUnreservedContinuation } from "./unreserved.mjs";
import { finishIterative } from "./iterative-judge.mjs";
import { createSuccessor } from "./successor.mjs";
import { amendPolicy } from "./amendment.mjs";
import { createSupervisor } from "./server.mjs";
import { finishWindow, recordCycle } from "./store.mjs";
import { protectedPath, QualificationError, readJson, requireCondition } from "./contract.mjs";

const KEYS = { "--supersede-unstarted": "supersedeUnstarted", "--supersede-startup": "supersedeStartup", "--supersede-premining": "supersedePremining", "--supersede-unissued": "supersedeUnissued", "--observer-binary": "observerBinary", "--recovery-phase": "recoveryPhase", "--original-campaign-record": "originalCampaignRecord", "--retained-runtime-from": "retainedRuntimeFrom", "--suggested-difficulty": "suggestedDifficulty", "--cycles-from": "cyclesFrom", "--purpose": "purpose", "--previous-receipt": "previousReceipt", "--firmware-root": "firmwareRoot", "--gate-root": "gateRoot", "--firmware-commit": "firmwareCommit", "--gate-commit": "gateCommit",
  "--manifest": "manifest", "--private-root": "privateRoot", "--authority-directory": "authorityDirectory", "--pool-credentials": "poolCredentials",
  "--cooling-input": "coolingInput", "--predecessor-root": "predecessorRoot", "--bun": "bun", "--port": "port", "--window": "window", "--input": "input",
  "--qualification-source-commit": "qualificationSourceCommit", "--gate-qualification-source-commit": "gateQualificationSourceCommit" };
export async function main(args, operations = {}) {
  const [command, ...rest] = args;
  const options = {};
  requireCondition(rest.length % 2 === 0, "argument_shape");
  for (let index = 0; index < rest.length; index += 2) {
    const key = KEYS[rest[index]];
    requireCondition(key && options[key] === undefined && !rest[index + 1].startsWith("--"), "argument_shape");
    options[key] = rest[index + 1];
  }
  requireCondition(options.privateRoot, "private_root_required");
  const allowed = {
    "reset-origin-preflight": ["firmwareRoot", "gateRoot", "firmwareCommit", "gateCommit", "manifest", "privateRoot", "input", "originalCampaignRecord", "predecessorRoot", "previousReceipt", "qualificationSourceCommit", "supersedeUnstarted"],
    "reset-origin-serve": ["privateRoot", "port", "bun"],
    "reset-origin-judge": ["privateRoot", "input"],
    "reset-origin-review": ["privateRoot"],
    "cadence-preflight": ["firmwareRoot", "gateRoot", "firmwareCommit", "gateCommit", "manifest", "privateRoot", "authorityDirectory", "bun", "previousReceipt", "input", "suggestedDifficulty", "observerBinary", "supersedeUnissued", "supersedePremining", "supersedeStartup"],
    "cadence-startup-recovery-preflight": ["firmwareRoot", "gateRoot", "firmwareCommit", "gateCommit", "manifest", "privateRoot", "input", "originalCampaignRecord", "predecessorRoot", "previousReceipt"],
    "cadence-startup-recovery-serve": ["privateRoot", "port", "bun"],
    "cadence-startup-recovery-judge": ["privateRoot", "input"],
    "cadence-startup-recovery-review": ["privateRoot"],
    "cadence-startup-recovery-consume-install": ["privateRoot"],
    "cadence-judge": ["privateRoot", "input"],
    "cadence-close-unissued": ["privateRoot", "input"],
    "cadence-close-premining": ["privateRoot"],
    "cadence-review-premining": ["privateRoot"],
    "iterative-recover-seal": ["privateRoot"],
    "iterative-continue-unreserved": ["privateRoot", "predecessorRoot", "input", "qualificationSourceCommit", "authorityDirectory", "bun"],
    "iterative-bootstrap": ["privateRoot", "input"],
    "iterative-judge": ["privateRoot", "input"],
    "iterative-preflight": ["firmwareRoot", "gateRoot", "firmwareCommit", "gateCommit", "manifest", "privateRoot", "authorityDirectory", "bun", "purpose", "previousReceipt", "input", "cyclesFrom", "suggestedDifficulty", "retainedRuntimeFrom", "qualificationSourceCommit"],
    "recovery-preflight": ["firmwareRoot", "gateRoot", "firmwareCommit", "gateCommit", "manifest", "privateRoot", "authorityDirectory", "bun", "recoveryPhase", "previousReceipt", "input", "cyclesFrom", "suggestedDifficulty"],
    "no-mining-preflight": ["firmwareRoot", "gateRoot", "firmwareCommit", "gateCommit", "manifest", "privateRoot", "originalCampaignRecord"],
    "no-mining-serve": ["privateRoot", "port", "bun"],
    "no-mining-judge": ["privateRoot", "input"],
    preflight: ["firmwareRoot", "gateRoot", "firmwareCommit", "gateCommit", "manifest", "privateRoot", "authorityDirectory", "bun"],
    serve: ["privateRoot", "authorityDirectory", "poolCredentials", "port", "bun"],
    judge: ["privateRoot", "window"],
    "record-cycle": ["privateRoot", "input"],
    "create-successor": ["privateRoot", "predecessorRoot", "input", "coolingInput"],
    "amend-policy": ["privateRoot", "qualificationSourceCommit", "gateQualificationSourceCommit"],
  }[command];
  requireCondition(allowed && Object.keys(options).every((key) => allowed.includes(key)), "command_arguments");
  if (command === "reset-origin-preflight") {
    for (const key of ["firmwareRoot", "gateRoot", "firmwareCommit", "gateCommit", "manifest", "input", "originalCampaignRecord", "predecessorRoot", "qualificationSourceCommit"])
      requireCondition(options[key], "preflight_argument_missing");
    return resetOriginPreflight(options);
  }
  if (command === "reset-origin-serve") return serveSupervisor(() => createResetOriginSupervisor(options), options);
  if (command === "reset-origin-judge") {
    requireCondition(options.input, "input_required");
    return judgeResetOrigin(resolve(options.privateRoot), options.input);
  }
  if (command === "reset-origin-review") {
    await readResetOrigin(resolve(options.privateRoot, "result.json"));
    return { observation_verified: true, device_recovery_claimed: false, mining_authorized: false };
  }
  if (command === "cadence-startup-recovery-preflight") {
    for (const key of ["firmwareRoot", "gateRoot", "firmwareCommit", "gateCommit", "manifest", "input", "originalCampaignRecord", "predecessorRoot"])
      requireCondition(options[key], "preflight_argument_missing");
    return startupRecoveryPreflight(options);
  }
  if (command === "cadence-startup-recovery-serve") return serveSupervisor(() => createStartupRecoverySupervisor(options), options);
  if (command === "cadence-startup-recovery-consume-install") return consumeStartupRecoveryInstall(resolve(options.privateRoot));
  if (command === "cadence-startup-recovery-judge") {
    requireCondition(options.input, "input_required");
    return judgeStartupRecovery(resolve(options.privateRoot), options.input);
  }
  if (command === "cadence-startup-recovery-review") {
    await readStartupRecovery(resolve(options.privateRoot, "result.json"));
    return { startup_recovery_verified: true, mining_authorized: false, qualification_pass: false };
  }
  if (command === "iterative-recover-seal") return recoverSampleSeal(resolve(options.privateRoot));
  if (command === "iterative-continue-unreserved") {
    for (const key of ["predecessorRoot", "input", "qualificationSourceCommit", "authorityDirectory"]) requireCondition(options[key], "continuation_argument_missing");
    return createUnreservedContinuation({ ...options, originRoot: options.predecessorRoot });
  }
  if (command === "iterative-bootstrap") {
    requireCondition(options.input, "input_required");
    return iterativeBootstrap(resolve(options.privateRoot), options.input);
  }
  if (command === "cadence-close-premining") return closePremining(resolve(options.privateRoot));
  if (command === "cadence-review-premining") return reviewPremining(resolve(options.privateRoot), operations);
  if (command === "cadence-close-unissued") {
    requireCondition(options.input, "input_required");
    await protectedPath(options.input);
    return closeUnissued(resolve(options.privateRoot), await readJson(options.input));
  }
  if (command === "cadence-preflight") {
    for (const key of ["firmwareRoot", "gateRoot", "firmwareCommit", "gateCommit", "manifest", "authorityDirectory", "previousReceipt", "input", "observerBinary"]) requireCondition(options[key], "preflight_argument_missing");
    return cadencePreflight(options);
  }
  if (command === "recovery-preflight") {
    for (const key of ["firmwareRoot", "gateRoot", "firmwareCommit", "gateCommit", "manifest", "authorityDirectory", "recoveryPhase", "previousReceipt", "input"]) requireCondition(options[key], "preflight_argument_missing");
    return recoveryPreflight(options);
  }
  if (command === "iterative-preflight") {
    for (const key of ["firmwareRoot", "gateRoot", "firmwareCommit", "gateCommit", "manifest", "authorityDirectory", "purpose", "previousReceipt", "input"]) requireCondition(options[key], "preflight_argument_missing");
    return iterativePreflight(options);
  }
  if (command === "no-mining-preflight") {
    for (const key of ["firmwareRoot", "gateRoot", "firmwareCommit", "gateCommit", "manifest"]) requireCondition(options[key], "preflight_argument_missing");
    return noMiningPreflight(options);
  }
  if (command === "preflight") {
    for (const key of ["firmwareRoot", "gateRoot", "firmwareCommit", "gateCommit", "manifest", "authorityDirectory"]) requireCondition(options[key], "preflight_argument_missing");
    return preflight(options);
  }
  const context = await loadContext(resolve(options.privateRoot));
  if (context.schema === CADENCE_SCHEMA) requireCondition(["serve", "record-cycle", "cadence-judge"].includes(command), "cadence_command_required");
  if (context.schema === NO_MINING_SCHEMA) requireCondition(["no-mining-serve", "no-mining-judge", "record-cycle"].includes(command), "no_mining_command_required");
  if (command === "no-mining-judge") {
    requireCondition(options.input, "input_required");
    return finishNoMining(resolve(options.privateRoot), context, options.input);
  }
  if (command === "cadence-judge") {
    requireCondition(context.schema === CADENCE_SCHEMA && options.input, "cadence_judge_arguments");
    return finishIterative(resolve(options.privateRoot), context, options.input);
  }
  if (command === "iterative-judge") {
    requireCondition(options.input, "input_required");
    return finishIterative(resolve(options.privateRoot), context, options.input);
  }
  if (command === "create-successor") {
    requireCondition(options.predecessorRoot && options.input, "successor_arguments_missing");
    return createSuccessor(resolve(options.privateRoot), context, options.predecessorRoot, options.input, options.coolingInput);
  }
  if (command === "amend-policy") {
    requireCondition(options.qualificationSourceCommit && options.gateQualificationSourceCommit, "amendment_arguments_missing");
    return amendPolicy(resolve(options.privateRoot), context, options);
  }
  if (command === "serve" || command === "no-mining-serve") {
    if (command === "serve") requireCondition(options.authorityDirectory && options.poolCredentials, "serve_argument_missing");
    return serveSupervisor(() => command === "no-mining-serve" ? createNoMiningSupervisor({ ...options, context }) : createSupervisor({ ...options, context }), options);
  }
  if (command === "judge") return finishWindow(resolve(options.privateRoot), context, Number(options.window));
  if (command === "record-cycle") {
    requireCondition(options.input, "input_required");
    await protectedPath(options.input);
    return recordCycle(resolve(options.privateRoot), context, await readJson(options.input));
  }
  throw new QualificationError("command_unavailable");
}

async function serveSupervisor(create, options) {
  const stdout = fstatSync(1);
  requireCondition(stdout.isFile() && (stdout.mode & 0o777) === 0o600, "protected_stdout_required");
  const port = Number(options.port ?? 0);
  requireCondition(Number.isInteger(port) && port >= 0 && port <= 65535, "port_argument");
  const server = await create();
  server.listen(port, "127.0.0.1");
  await once(server, "listening");
  process.stdout.write(`qualification_url=http://127.0.0.1:${server.address().port}/\n`);
  for (const signal of ["SIGINT", "SIGTERM"]) process.once(signal, () => { server.close(); server.closeIdleConnections(); });
  await once(server, "close");
  if (server.closeQualificationResources) await server.closeQualificationResources();
  return { supervisor: "closed", device_effects: false };
}
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main(process.argv.slice(2)).then((result) => process.stdout.write(`${JSON.stringify(result)}\n`))
    .catch((error) => { process.stderr.write(`qualification_failed=${error instanceof QualificationError ? error.code : "local_operation_failed"}\n`); process.exitCode = 1; });
}
