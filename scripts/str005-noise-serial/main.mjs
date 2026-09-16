import { fstatSync } from "node:fs";
import { once } from "node:events";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { requireValue as check } from "./contract.mjs";

const ACTIONS = ["preflight", "serve", "recover", "finalize", "review"];
const PREFLIGHT = ["firmware-root", "gate-root", "package-manifest", "fixture-binary", "attempt-ordinal", "predecessor-receipt"];

/** Parse the frozen CLI without reading any caller-selected path or private input. */
export function parseArgs(argv) {
  const [action, ...args] = argv;
  check(ACTIONS.includes(action), "noise_action_invalid");
  const allowed = new Set(["private-root", ...(action === "preflight" ? PREFLIGHT : []),
    ...(action === "serve" ? ["port"] : []), ...(action === "recover" ? ["attempt-root"] : []),
    ...(action === "finalize" ? ["cleanup-receipt", "recovery-receipt"] : [])]);
  const options = {};
  check(args.length % 2 === 0, "noise_option_value_missing");
  for (let index = 0; index < args.length; index += 2) {
    const key = args[index].replace(/^--/u, ""), value = args[index + 1];
    check(args[index].startsWith("--") && allowed.has(key) && !Object.hasOwn(options, key), "noise_option_rejected");
    check(value.length > 0 && !value.startsWith("--"), "noise_option_value_missing");
    options[key] = value;
  }
  const required = ["private-root", ...(action === "preflight" ? PREFLIGHT : []),
    ...(action === "recover" ? ["attempt-root"] : []), ...(action === "finalize" ? ["cleanup-receipt"] : [])];
  check(required.every((key) => Object.hasOwn(options, key)), "noise_required_option");
  check(resolve(options["private-root"]) === options["private-root"], "noise_absolute_root_required");
  if (options.port !== undefined) check(/^[1-9][0-9]{0,4}$/u.test(options.port) && Number(options.port) <= 65535, "noise_port_invalid");
  if (options["attempt-ordinal"] !== undefined) check(/^[1-9][0-9]*$/u.test(options["attempt-ordinal"]) && Number.isSafeInteger(Number(options["attempt-ordinal"])), "noise_ordinal_invalid");
  return { action, options };
}

/** Canonical v2 workflow. Recovery remains unavailable before input-dependent effects. */
export async function main(argv, operations = {}) {
  const { action, options: raw } = parseArgs(argv);
  if (action === "recover" || raw["recovery-receipt"] !== undefined)
    check(false, "noise_recovery_unavailable");
  const options = { privateRoot: raw["private-root"], firmwareRoot: raw["firmware-root"], gateRoot: raw["gate-root"],
    manifest: raw["package-manifest"], fixtureBinary: raw["fixture-binary"], attemptOrdinal: raw["attempt-ordinal"], predecessorReceipt: raw["predecessor-receipt"] };
  if (action === "preflight") return (await import("./context.mjs")).preflight(options, operations);
  if (action === "finalize") return (await import("./finalize.mjs")).finalize(options.privateRoot, raw["cleanup-receipt"], operations);
  if (action === "review") return (await import("./finalize.mjs")).review(options.privateRoot, operations);
  const stdout = fstatSync(1);
  check(stdout.isFile() && (stdout.mode & 0o777) === 0o600, "noise_protected_stdout_required");
  const server = await (await import("./server.mjs")).createSupervisor(options, operations);
  server.listen(Number(raw.port ?? 0), "127.0.0.1"); await once(server, "listening"); await server.qualificationReady;
  process.stdout.write(`qualification_url=http://127.0.0.1:${server.address().port}/\n`);
  for (const signal of ["SIGINT", "SIGTERM"]) process.once(signal, () => { server.close(); server.closeIdleConnections(); });
  await once(server, "close"); await server.closeQualificationResources();
  return { supervisor: "closed", device_effects: false, hardware_qualified: false };
}
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main(process.argv.slice(2)).then((value) => process.stdout.write(`${JSON.stringify(value)}\n`)).catch((error) => {
    process.stdout.write(`${JSON.stringify({ ready: false, device_effects: false, hardware_qualified: false,
      error: typeof error.code === "string" && /^noise_[a-z_]+$/u.test(error.code) ? error.code : "noise_operation_rejected" })}\n`);
    process.exitCode = 1;
  });
}
