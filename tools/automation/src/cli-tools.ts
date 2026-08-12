import path from "node:path";

export function toolProgram(root: string, relative: string): string {
  const maybeRunfiles = process.env["RUNFILES_DIR"];
  return maybeRunfiles === undefined
    ? path.join(root, "bazel-bin", relative)
    : path.join(maybeRunfiles, "_main", relative);
}

export function flashProgram(root: string): string {
  return toolProgram(root, "tools/flash/flash");
}

export function deviceSessionProgram(root: string): string {
  return toolProgram(root, "tools/device-session/device-session");
}

export function stringNumber(value: string | undefined): number | undefined {
  return value === undefined ? undefined : Number(value);
}
