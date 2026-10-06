// Run repo `just` recipes from the workspace with private (0600) output files, like the human command surface.
import { spawn } from "node:child_process";
import { once } from "node:events";
import { open, readFile } from "node:fs/promises";
import { refuse } from "./errors.mjs";
import { hostEnvironment, workspace } from "./host.mjs";
import { parseDetector } from "./detector.mjs";

// `just` re-parses `{{ args }}` through the shell, so every argument after the recipe is single-quoted.
export const quoteJustArgument = (value) => `'${String(value).replaceAll("'", "'\\''")}'`;
const justArguments = (args) => [args[0], ...args.slice(1).map(quoteJustArgument)];

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
    child = spawn("just", justArguments(args), { cwd: workspace(environment(operations)), env: environment(operations), stdio: ["ignore", stdout.fd, stderr.fd] });
  } finally { await stdout.close(); await stderr.close(); }
  const [code] = await once(child, "close");
  return code;
}

/** Start one long-lived recipe (a `serve`) in its own session so it outlives this command. */
export async function startJustDetached(args, { stdoutPath, stderrPath }, operations = {}) {
  const { stdout, stderr } = await privateOutputs(stdoutPath, stderrPath);
  try {
    const child = spawn("just", justArguments(args), { cwd: workspace(environment(operations)), env: environment(operations),
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
