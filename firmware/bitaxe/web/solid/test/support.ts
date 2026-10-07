import { readFile } from "node:fs/promises";
import path from "node:path";
import vm from "node:vm";

/** Workspace-relative root inside Bazel runfiles, or the working directory outside Bazel. */
export function runfileRoot(): string {
  const maybeRunfiles = process.env["RUNFILES_DIR"] ?? process.env["JS_BINARY__RUNFILES"];
  return maybeRunfiles === undefined ? process.cwd() : path.join(maybeRunfiles, "_main");
}

export const CURRENT_ASSETS = "firmware/bitaxe/static/www/assets";
export const SOLID_PACKAGE = "firmware/bitaxe/web/solid";

export async function readWorkspaceFile(relativePath: string): Promise<string> {
  return readFile(path.join(runfileRoot(), relativePath), "utf8");
}

/** Evaluates one of the current variant's classic scripts and returns the global it installs. */
export async function evaluateCurrentScript(fileName: string, globalName: string, context: Record<string, unknown> = {}): Promise<unknown> {
  const source = await readWorkspaceFile(`${CURRENT_ASSETS}/${fileName}`);
  const sandbox: Record<string, unknown> = { ...context };
  vm.runInNewContext(source, sandbox, { filename: fileName });
  return sandbox[globalName];
}

/** Deep-copies a value through JSON so results from another VM realm compare structurally. */
export function plain<T>(value: T): T {
  return JSON.parse(JSON.stringify(value)) as T;
}
