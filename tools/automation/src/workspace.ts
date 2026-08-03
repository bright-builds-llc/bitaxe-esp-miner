import path from "node:path";

export function anchoredPath(workspaceRoot: string, candidate: string): string {
  return path.isAbsolute(candidate) ? path.normalize(candidate) : path.resolve(workspaceRoot, candidate);
}

export function assertWithinWorkspace(workspaceRoot: string, candidate: string): string {
  const anchored = anchoredPath(workspaceRoot, candidate);
  const relative = path.relative(workspaceRoot, anchored);
  if (relative === "" || (!relative.startsWith("..") && !path.isAbsolute(relative))) return anchored;
  throw new Error("path must remain within the workspace");
}
