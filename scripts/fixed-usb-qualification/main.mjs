#!/usr/bin/env node
import { noMiningPreflight, NO_MINING_SCHEMA } from "./no-mining-context.mjs";
import { createNoMiningSupervisor } from "./no-mining-server.mjs";
import { finishNoMining } from "./no-mining-judge.mjs";
import { recoverSampleSeal } from "./sample-seal.mjs";
import { once } from "node:events";
import { fstatSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { preflight, loadContext } from "./preflight.mjs";
import { iterativeBootstrap, iterativePreflight } from "./iterative-preflight.mjs";
import { createUnreservedContinuation } from "./unreserved.mjs";
import { finishIterative } from "./iterative-judge.mjs";
import { createSuccessor } from "./successor.mjs";
import { amendPolicy } from "./amendment.mjs";
import { createSupervisor } from "./server.mjs";
import { finishWindow, recordCycle } from "./store.mjs";
import { protectedPath, QualificationError, readJson, requireCondition } from "./contract.mjs";

const KEYS = { "--original-campaign-record": "originalCampaignRecord", "--retained-runtime-from": "retainedRuntimeFrom", "--suggested-difficulty": "suggestedDifficulty", "--cycles-from": "cyclesFrom", "--purpose": "purpose", "--previous-receipt": "previousReceipt", "--firmware-root": "firmwareRoot", "--gate-root": "gateRoot", "--firmware-commit": "firmwareCommit", "--gate-commit": "gateCommit",
  "--manifest": "manifest", "--private-root": "privateRoot", "--authority-directory": "authorityDirectory", "--pool-credentials": "poolCredentials",
  "--cooling-input": "coolingInput", "--predecessor-root": "predecessorRoot", "--bun": "bun", "--port": "port", "--window": "window", "--input": "input",
  "--qualification-source-commit": "qualificationSourceCommit", "--gate-qualification-source-commit": "gateQualificationSourceCommit" };
export async function main(args) {
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
    "iterative-recover-seal": ["privateRoot"],
    "iterative-continue-unreserved": ["privateRoot", "predecessorRoot", "input", "qualificationSourceCommit", "authorityDirectory", "bun"],
    "iterative-bootstrap": ["privateRoot", "input"],
    "iterative-judge": ["privateRoot", "input"],
    "iterative-preflight": ["firmwareRoot", "gateRoot", "firmwareCommit", "gateCommit", "manifest", "privateRoot", "authorityDirectory", "bun", "purpose", "previousReceipt", "input", "cyclesFrom", "suggestedDifficulty", "retainedRuntimeFrom", "qualificationSourceCommit"],
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
  if (command === "iterative-recover-seal") return recoverSampleSeal(resolve(options.privateRoot));
  if (command === "iterative-continue-unreserved") {
    for (const key of ["predecessorRoot", "input", "qualificationSourceCommit", "authorityDirectory"]) requireCondition(options[key], "continuation_argument_missing");
    return createUnreservedContinuation({ ...options, originRoot: options.predecessorRoot });
  }
  if (command === "iterative-bootstrap") {
    requireCondition(options.input, "input_required");
    return iterativeBootstrap(resolve(options.privateRoot), options.input);
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
  if (context.schema === NO_MINING_SCHEMA) requireCondition(["no-mining-serve", "no-mining-judge", "record-cycle"].includes(command), "no_mining_command_required");
  if (command === "no-mining-judge") {
    requireCondition(options.input, "input_required");
    return finishNoMining(resolve(options.privateRoot), context, options.input);
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
    const stdout = fstatSync(1);
    requireCondition(stdout.isFile() && (stdout.mode & 0o777) === 0o600, "protected_stdout_required");
    const port = Number(options.port ?? 0);
    requireCondition(Number.isInteger(port) && port >= 0 && port <= 65535, "port_argument");
    const server = command === "no-mining-serve" ? await createNoMiningSupervisor({ ...options, context }) : await createSupervisor({ ...options, context });
    server.listen(port, "127.0.0.1");
    await once(server, "listening");
    process.stdout.write(`qualification_url=http://127.0.0.1:${server.address().port}/\n`);
    for (const signal of ["SIGINT", "SIGTERM"]) process.once(signal, () => { server.close(); server.closeIdleConnections(); });
    await once(server, "close");
    return { supervisor: "closed", device_effects: false };
  }
  if (command === "judge") return finishWindow(resolve(options.privateRoot), context, Number(options.window));
  if (command === "record-cycle") {
    requireCondition(options.input, "input_required");
    await protectedPath(options.input);
    return recordCycle(resolve(options.privateRoot), context, await readJson(options.input));
  }
  throw new QualificationError("command_unavailable");
}
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main(process.argv.slice(2)).then((result) => process.stdout.write(`${JSON.stringify(result)}\n`))
    .catch((error) => { process.stderr.write(`qualification_failed=${error instanceof QualificationError ? error.code : "local_operation_failed"}\n`); process.exitCode = 1; });
}
