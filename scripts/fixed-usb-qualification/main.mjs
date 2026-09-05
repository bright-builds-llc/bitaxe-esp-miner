#!/usr/bin/env node
import { once } from "node:events";
import { fstatSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { preflight, loadContext } from "./preflight.mjs";
import { createSupervisor } from "./server.mjs";
import { finishWindow, recordCycle } from "./store.mjs";
import { protectedPath, QualificationError, readJson, requireCondition } from "./contract.mjs";

const KEYS = { "--firmware-root": "firmwareRoot", "--gate-root": "gateRoot", "--firmware-commit": "firmwareCommit", "--gate-commit": "gateCommit",
  "--manifest": "manifest", "--private-root": "privateRoot", "--authority-directory": "authorityDirectory", "--pool-credentials": "poolCredentials",
  "--bun": "bun", "--port": "port", "--window": "window", "--input": "input" };
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
    preflight: ["firmwareRoot", "gateRoot", "firmwareCommit", "gateCommit", "manifest", "privateRoot", "authorityDirectory", "bun"],
    serve: ["privateRoot", "authorityDirectory", "poolCredentials", "port", "bun"],
    judge: ["privateRoot", "window"],
    "record-cycle": ["privateRoot", "input"],
  }[command];
  requireCondition(allowed && Object.keys(options).every((key) => allowed.includes(key)), "command_arguments");
  if (command === "preflight") {
    for (const key of ["firmwareRoot", "gateRoot", "firmwareCommit", "gateCommit", "manifest", "authorityDirectory"]) requireCondition(options[key], "preflight_argument_missing");
    return preflight(options);
  }
  const context = await loadContext(resolve(options.privateRoot));
  if (command === "serve") {
    requireCondition(options.authorityDirectory && options.poolCredentials, "serve_argument_missing");
    const stdout = fstatSync(1);
    requireCondition(stdout.isFile() && (stdout.mode & 0o777) === 0o600, "protected_stdout_required");
    const port = Number(options.port ?? 0);
    requireCondition(Number.isInteger(port) && port >= 0 && port <= 65535, "port_argument");
    const server = await createSupervisor({ ...options, context });
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
