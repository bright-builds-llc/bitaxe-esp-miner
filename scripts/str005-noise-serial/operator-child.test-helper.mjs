// Real process/IPC test boundary; only repository-source admission is synthetic.
import { spawn } from "node:child_process";
import { once } from "node:events";
import { admitExecution, quoteJustArgument } from "./operator-execution.mjs";
const [root, mode, rawIndex] = process.argv.slice(2);
process.send({ ready: true });
const [permit] = await once(process, "message");
const admitted = await admitExecution(root, mode, Number(rawIndex), permit, {
  verifySources: async () => {},
  execFileSync() { throw Object.assign(new Error("synthetic holder absence"), { status: 1, signal: null, stdout: "", stderr: "" }); },
});
const child = spawn("just", [admitted.argv[0], ...admitted.argv.slice(1).map(quoteJustArgument)], { cwd: admitted.context.firmware_root, stdio: ["ignore", "inherit", "inherit"] });
const [code, signal] = await once(child, "close");
process.disconnect(); process.exitCode = code === 0 && signal === null ? 0 : 1;
