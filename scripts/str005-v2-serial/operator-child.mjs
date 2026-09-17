// The prearmed parent is the only producer of a one-shot execution permit.
import { spawn } from "node:child_process";
import { once } from "node:events";
import { resolve } from "node:path";
import { missing } from "../fixed-usb-qualification/contract.mjs";
import { admitExecution, quoteJustArgument } from "./operator-execution.mjs";
import { check } from "./values.mjs";
const [root, mode, number, ...extra] = process.argv.slice(2), index = Number(number);
check(process.send && extra.length === 0 && ["detect", "flash"].includes(mode) && Number.isInteger(index) && index >= 0 && index <= 4,
  "v2_operator_arguments");
process.send({ ready: true });
const [message] = await once(process, "message");
const { context, argv } = await admitExecution(root, mode, index, message);
await missing(resolve(root, "failure.json"));
const child = spawn("just", [argv[0], ...argv.slice(1).map(quoteJustArgument)], {
  cwd: context.firmware_root, stdio: ["ignore", "inherit", "inherit"], env: process.env,
});
const [code, signal] = await once(child, "close");
if (process.send) process.send({ code, signal });
process.disconnect(); process.exitCode = code === 0 && signal === null ? 0 : 1;
