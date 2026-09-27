import { once } from "node:events";
// Tests only: real daemon lifecycle, real IPC and parent, fixed synthetic native operations.
import { fileURLToPath } from "node:url";
import { testOperations } from "./test-fixture.mjs";
import { runOperator } from "./operator-daemon.mjs";
const incoming = once(process, "message");
const root = process.argv[2], operations = await testOperations(root);
operations.supervisorProgram = fileURLToPath(new URL("./composition-supervisor.mjs", import.meta.url)); operations.installChildProgram = fileURLToPath(new URL("./composition-install-child.mjs", import.meta.url));
const [input] = await incoming;
await runOperator(root, input, operations).catch(error => { process.stderr.write(`${error.code ?? "test_failed"}\n`); process.exitCode = 1; if (process.connected) process.disconnect(); });
