import { existsSync, realpathSync } from "node:fs";
import path from "node:path";

export function sourceWorkspaceRoot(starts: readonly string[]): string {
  for (const start of starts) {
    let candidate = path.resolve(start);
    while (true) {
      const moduleFile = path.join(candidate, "MODULE.bazel");
      if (existsSync(moduleFile)) {
        const resolved = path.dirname(realpathSync(moduleFile));
        if (existsSync(path.join(resolved, ".git"))) return resolved;
      }
      const parent = path.dirname(candidate);
      if (parent === candidate) break;
      candidate = parent;
    }
  }
  throw new Error("source workspace unavailable");
}

export function anchoredPath(workspaceRoot: string, candidate: string): string {
  return path.isAbsolute(candidate) ? path.normalize(candidate) : path.resolve(workspaceRoot, candidate);
}

export function assertWithinWorkspace(workspaceRoot: string, candidate: string): string {
  const anchored = anchoredPath(workspaceRoot, candidate);
  const relative = path.relative(workspaceRoot, anchored);
  if (relative === "" || (!relative.startsWith("..") && !path.isAbsolute(relative))) return anchored;
  throw new Error("path must remain within the workspace");
}
