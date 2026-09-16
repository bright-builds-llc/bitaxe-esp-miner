// Gated command process. The parent observes/prearms it before permitting execution.
import { spawn } from "node:child_process";
import { resolve } from "node:path";
import { once } from "node:events";
import { missing } from "../fixed-usb-qualification/contract.mjs";
import { admitExecution, quoteJustArgument } from "./operator-execution.mjs";
import { proof, check } from "./files.mjs";
const [root, mode, number] = process.argv.slice(2), index = Number(number);
check(process.send && ["detect", "flash"].includes(mode) && Number.isInteger(index) && index >= 0 && index <= 4, "noise_operator_arguments");
process.send({ ready: true });
const [message] = await once(process, "message");
const { context, argv } = await admitExecution(root, mode, index, message);
await missing(resolve(root, "failure.json"));
const args = [argv[0], ...argv.slice(1).map(quoteJustArgument)];
const child = spawn("just", args, { cwd: context.firmware_root, stdio: ["ignore", "inherit", "inherit"], env: process.env });
const [code, signal] = await once(child, "close");
if (process.send) process.send({ code, signal });
process.disconnect(); process.exitCode = code === 0 && signal === null ? 0 : 1;
