import { once } from "node:events";
// Tests only: synthetic native/publication prerequisites; actual managed supervisor.
import { testOperations } from "./test-fixture.mjs";
import { runSupervisor } from "./operator-supervisor.mjs";
const incoming = once(process, "message");
const root = process.argv[2], operations = await testOperations(root);
const [input] = await incoming;
await runSupervisor(root, input.binding, operations).catch(error => { process.stderr.write(`${error.code ?? "test_failed"}\n`); process.exitCode = 1; if (process.connected) process.disconnect(); });
