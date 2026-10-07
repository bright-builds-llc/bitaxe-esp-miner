// Run repo `just` recipes from the workspace with private (0600) output files, like the human command surface.
import { spawn } from "node:child_process";
import { once } from "node:events";
import { readFileSync } from "node:fs";
import { open, readFile } from "node:fs/promises";
import { resolve } from "node:path";
import { refuse } from "./errors.mjs";
import { hostEnvironment, workspace } from "./host.mjs";
import { parseDetector } from "./detector.mjs";

// `just` re-parses `{{ args }}` through the shell, so every argument after the recipe is single-quoted.
export const quoteJustArgument = (value) => `'${String(value).replaceAll("'", "'\\''")}'`;

/** Recipes declared `[positional-arguments]` receive "$@" verbatim, so quoting them would add literal quotes. */
export function positionalRecipes(justfileText) {
  const names = new Set();
  const lines = justfileText.split(/\r?\n/u);
  lines.forEach((line, index) => {
    if (line.trim() !== "[positional-arguments]") return;
    const maybeRecipe = lines.slice(index + 1).find((next) => !next.trim().startsWith("[") && next.trim() !== "");
    const maybeName = maybeRecipe?.match(/^@?([A-Za-z0-9_-]+)/u)?.[1];
    if (maybeName) names.add(maybeName);
  });
  return names;
}

function justArguments(args, root) {
  let positional = new Set();
  try { positional = positionalRecipes(readFileSync(resolve(root, "Justfile"), "utf8")); } catch (error) { if (error.code !== "ENOENT") throw error; }
  return positional.has(args[0]) ? args.map(String) : [args[0], ...args.slice(1).map(quoteJustArgument)];
}

async function privateOutputs(stdoutPath, stderrPath) {
  const stdout = await open(stdoutPath, "wx", 0o600);
  try { return { stdout, stderr: await open(stderrPath, "wx", 0o600) }; } catch (error) { await stdout.close(); throw error; }
}

const environment = (operations) => hostEnvironment(operations.env ?? process.env);

/** Run one recipe to completion; returns its exit code (null when killed by a signal). */
export async function runJust(args, { stdoutPath, stderrPath }, operations = {}) {
  const { stdout, stderr } = await privateOutputs(stdoutPath, stderrPath);
  let child;
  try {
    const root = workspace(environment(operations));
    child = spawn("just", justArguments(args, root), { cwd: root, env: environment(operations), stdio: ["ignore", stdout.fd, stderr.fd] });
  } finally { await stdout.close(); await stderr.close(); }
  const [code] = await once(child, "close");
  return code;
}

/** Start one long-lived recipe (a `serve`) in its own session so it outlives this command. */
export async function startJustDetached(args, { stdoutPath, stderrPath }, operations = {}) {
  const { stdout, stderr } = await privateOutputs(stdoutPath, stderrPath);
  try {
    const root = workspace(environment(operations));
    const child = spawn("just", justArguments(args, root), { cwd: root, env: environment(operations),
      detached: true, stdio: ["ignore", stdout.fd, stderr.fd] });
    child.unref();
    return child;
  } finally { await stdout.close(); await stderr.close(); }
}

/** Fresh detection into private files; only an admitted Serial/JTAG runtime device passes. */
export async function detect(stdoutPath, stderrPath, operations = {}) {
  const code = await runJust(["detect-ultra205"], { stdoutPath, stderrPath }, operations);
  refuse(code === 0, "detector_failed");
  return parseDetector(await readFile(stdoutPath, "utf8"));
}
