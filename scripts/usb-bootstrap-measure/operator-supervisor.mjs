import { resolve } from "node:path";
import { once } from "node:events";
import { admitOperatorParent } from "./operator-parent.mjs";
import { createSupervisor } from "./server.mjs";
import { check, object } from "./values.mjs";
export async function runSupervisor(root, binding, operations = {}) {
  await admitOperatorParent(root, binding);
  const server = await createSupervisor({ privateRoot: root }, operations);
  const stop = () => { server.closeQualificationResources().catch(() => { process.exitCode = 1; }); };
  process.once("SIGTERM", stop); process.once("SIGINT", stop);
  const closed = once(server, "close"); server.listen(0, "127.0.0.1"); await server.qualificationReady;
  process.stdout.write(`qualification_url=http://127.0.0.1:${server.address().port}/\n`);
  await closed; process.removeListener("SIGTERM", stop); process.removeListener("SIGINT", stop);
  if (process.connected) process.disconnect();
}
if (process.argv[1]?.endsWith("/operator-supervisor.mjs")) {
  check(process.connected && process.argv.length === 3, "bootstrap_operator_owner");
  const timer = setTimeout(() => process.exit(1), 10000);
  process.once("message", input => {
    clearTimeout(timer); object(input, ["binding"]);
    runSupervisor(resolve(process.argv[2]), input.binding).catch(() => { process.exitCode = 1; if (process.connected) process.disconnect(); });
  });
}
