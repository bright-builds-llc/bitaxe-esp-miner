import { basename, resolve } from "node:path";
import { check, SCOPES } from "./values.mjs";

export const TASK_ID = "task-str005-v2-serial-qualification";
const ACTIONS = ["preflight", "serve", "recover", "finalize", "review", "close-permission", "review-permission"];
const PREFLIGHT = ["scope", "firmware-root", "gate-root", "package-manifest", "fixture-binary", "predecessor-receipt"];

/** Validate all options before opening any path, allocating an attempt or reading signing material. */
export function parseArgs(argv) {
  const [action, ...args] = argv;
  check(ACTIONS.includes(action), "v2_action_invalid");
  const extra = action === "preflight" ? [...PREFLIGHT, "supersede-permission"] : action === "serve" ? ["authority-directory"] :
    action === "finalize" ? ["cleanup-receipt"] : [];
  const allowed = new Set(["private-root", ...extra]), raw = {};
  check(args.length % 2 === 0, "v2_option_value_missing");
  for (let index = 0; index < args.length; index += 2) {
    const [flag, value] = args.slice(index, index + 2), name = flag.replace(/^--/u, "");
    check(flag.startsWith("--") && allowed.has(name) && !Object.hasOwn(raw, name), "v2_option_rejected");
    check(typeof value === "string" && value.length > 0 && !value.startsWith("--"), "v2_option_value_missing");
    raw[name] = value;
  }
  const required = ["private-root", ...(action === "preflight" ? PREFLIGHT : action === "finalize" ? ["cleanup-receipt"] : [])];
  check(required.every((key) => Object.hasOwn(raw, key)), "v2_required_option");
  check(raw["private-root"] === resolve(raw["private-root"]), "v2_absolute_root_required");
  if (action === "preflight") check(SCOPES.includes(raw.scope), "v2_scope");
  if (raw["supersede-permission"] !== undefined) check(raw.scope === "channel" &&
    raw["supersede-permission"] === resolve(raw["supersede-permission"]), "v2_permission_scope");
  return { action, options: { privateRoot: raw["private-root"], scope: raw.scope,
    firmwareRoot: raw["firmware-root"], gateRoot: raw["gate-root"], manifest: raw["package-manifest"],
    fixtureBinary: raw["fixture-binary"], predecessorReceipt: raw["predecessor-receipt"],
    authorityDirectory: raw["authority-directory"], cleanupReceipt: raw["cleanup-receipt"], supersedePermission: raw["supersede-permission"] } };
}

export function attemptName(root, scope) {
  check(SCOPES.includes(scope), "v2_scope");
  const match = /^(channel|share)-([0-9]{3,})$/u.exec(basename(root));
  check(match !== null && match[1] === scope, "v2_attempt_name");
  const ordinal = Number(match[2]);
  check(Number.isSafeInteger(ordinal) && ordinal > 0 && String(ordinal).padStart(3, "0") === match[2], "v2_attempt_ordinal");
  return ordinal;
}

/** Archive content never supplies effect eligibility, including a matching archived stable ID. */
export function requireActiveTask(text) {
  let active = false;
  const all = [], eligible = [];
  for (const line of text.split(/\r?\n/u)) {
    if (line.startsWith("## ")) active = line === "## Active";
    const match = /^### (task-[^\s|]+)(?:\s|$)/u.exec(line);
    if (!match) continue;
    all.push(match[1]); if (active) eligible.push(match[1]);
  }
  check(all.filter((id) => id === TASK_ID).length === 1 && eligible.filter((id) => id === TASK_ID).length === 1,
    "v2_live_task_inactive");
}

export function requireAuthorityOption(scope, options) {
  check(SCOPES.includes(scope), "v2_scope");
  check(scope === "share" ? typeof options.authorityDirectory === "string" && options.authorityDirectory.length > 0 :
    options.authorityDirectory === undefined, "v2_authority_scope");
}
