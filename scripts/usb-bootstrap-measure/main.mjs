import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { check, code } from "./values.mjs";
export function parseArgs(argv) {
  const [action, ...args] = argv; check(["preflight", "operator-start", "operator-request", "operator-status", "finalize", "review"].includes(action), "bootstrap_action");
  const preflight = ["firmware-root", "gate-root", "package-manifest", "predecessor-receipt"];
  const required = ["private-root", ...(action === "preflight" ? preflight : action === "operator-request" ? ["request"] : [])];
  const allowed = [...required, ...(action === "operator-status" ? ["request-id"] : [])], raw = {}; check(args.length % 2 === 0, "bootstrap_options");
  for (let i = 0; i < args.length; i += 2) { const [flag, value] = args.slice(i, i + 2); check(flag.startsWith("--") && allowed.includes(flag.slice(2)) && typeof value === "string" && value.length > 0 && !value.startsWith("--") && !Object.hasOwn(raw, flag.slice(2)), "bootstrap_options"); raw[flag.slice(2)] = value; }
  check(required.every(key => Object.hasOwn(raw, key)) && raw["private-root"] === resolve(raw["private-root"]), "bootstrap_options");
  return { action, options: { privateRoot: raw["private-root"], firmwareRoot: raw["firmware-root"], gateRoot: raw["gate-root"], manifest: raw["package-manifest"],
    predecessorReceipt: raw["predecessor-receipt"], requestFile: raw.request, requestId: raw["request-id"] } };
}
export async function main(argv, operations = {}) {
  const { action, options } = parseArgs(argv);
  if (action === "preflight") return (await import("./context.mjs")).preflight(options, operations);
  if (["review", "finalize"].includes(action)) return (await import("./finalize.mjs"))[action](options.privateRoot, operations);
  return (await import("./operator-client.mjs"))[{ "operator-start": "operatorStart", "operator-request": "operatorRequest", "operator-status": "operatorStatus" }[action]](options, operations);
}
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) main(process.argv.slice(2)).then(value => process.stdout.write(`${JSON.stringify(value)}\n`)).catch(error => {
  process.stdout.write(`${JSON.stringify({ ready: false, hardware_qualified: false, error: code(error) })}\n`); process.exitCode = 1;
});
