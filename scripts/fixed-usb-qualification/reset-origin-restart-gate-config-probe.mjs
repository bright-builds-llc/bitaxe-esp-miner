import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { readFileSync } from "node:fs";
import { registerHooks, stripTypeScriptTypes } from "node:module";
import { dirname, extname, resolve } from "node:path";
import { pathToFileURL } from "node:url";
const gateRoot = dirname(resolve(process.argv[2]));
const gateUrl = pathToFileURL(`${gateRoot}/`).href;
registerHooks({
  resolve(specifier, context, next) {
    if (specifier.startsWith(".") && context.parentURL?.startsWith(gateUrl)) {
      const target = new URL(specifier, context.parentURL);
      if (!extname(target.pathname)) target.pathname += ".ts";
      return next(target.href, context);
    }
    return next(specifier, context);
  },
  load(url, context, next) {
    if (url.startsWith(gateUrl) && url.endsWith(".ts"))
      return {
        format: "module",
        shortCircuit: true,
        source: stripTypeScriptTypes(readFileSync(new URL(url), "utf8"), { mode: "transform" }),
      };
    return next(url, context);
  },
});
const chunks = [];
for await (const chunk of process.stdin) chunks.push(chunk);
const input = JSON.parse(Buffer.concat(chunks).toString("utf8"));
const { parseWorkerSerialAcceptanceConfiguration } = await import(
  pathToFileURL(resolve(gateRoot, "web/worker-serial-acceptance-config.ts"))
);
assert.equal(parseWorkerSerialAcceptanceConfiguration(input, input.expectedGateCommit).restartQualification, true);
assert.throws(() => parseWorkerSerialAcceptanceConfiguration({ ...input, restartPhase: "before-install" }, input.expectedGateCommit));
const source = await readFile(resolve(gateRoot, "web/worker-serial-acceptance.ts"), "utf8");
assert.match(source, /localJson\("\/context"\)\s*\.then\(configure\)/u);
process.stdout.write(
  JSON.stringify({ actual_gate_configuration_accepted: true, autoload_uses_same_payload: true, phase_key_rejected: true }),
);
