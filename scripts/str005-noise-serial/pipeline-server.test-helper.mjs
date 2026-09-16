// Fresh process executes the real HTTP server; only repository/hardware inputs are synthetic.
import { spawn } from "node:child_process";
import { once } from "node:events";
import { readFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { createSupervisor } from "./server.mjs";
import { nodeRuntimeEnvironment } from "./node-runtime.mjs";
const root = process.argv[2], { context } = JSON.parse(await readFile(`${root}/context.json`));
const operations = {
  cleanPushed() {}, ignored() {},
  git(path) { return path === context.firmware_root ? context.firmware_commit : context.gate_commit; },
  inspectPredecessor: async () => ({ inventorySha256: context.predecessor.inventorySha256 }),
  inspectNative: async () => context.native_readiness,
  networkInterfaces: () => ({ synthetic: [{ address: "192.168.1.20", family: "IPv4", netmask: "255.255.255.0", internal: false }] }),
  execFileSync() { throw Object.assign(new Error("synthetic serial absence"), { status: 1, signal: null, stdout: "", stderr: "" }); },
  spawn(_command, _args, options) {
    const child = spawn(process.execPath, [fileURLToPath(new URL("./pipeline-fixture-child.test-helper.mjs", import.meta.url)), root, context.attempt_id], {
      ...options, env: { ...options.env, ...nodeRuntimeEnvironment() },
    });
    child.stderr.on("data", (bytes) => process.stderr.write(bytes));
    return child;
  },
};
const server = await createSupervisor({ privateRoot: root }, operations);
server.listen(0, "127.0.0.1"); await once(server, "listening"); await server.qualificationReady;
process.send({ origin: `http://127.0.0.1:${server.address().port}` });
await once(process, "message");
server.closeAllConnections(); await new Promise((done) => server.close(done)); await server.closeQualificationResources();
process.disconnect();
