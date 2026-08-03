import assert from "node:assert/strict";
import { readFile, readdir } from "node:fs/promises";
import path from "node:path";
import test from "node:test";

const automationExtensions = new Set([".sh", ".mjs", ".js", ".py"]);

function runfileRoot(): string {
  const maybeRunfiles = process.env["RUNFILES_DIR"];
  return maybeRunfiles === undefined
    ? (process.env["BUILD_WORKSPACE_DIRECTORY"] ?? process.cwd())
    : path.join(maybeRunfiles, "_main");
}

function isAllowedTerminalArtifact(name: string): boolean {
  return (
    name === "bright-builds-auto-update.sh" ||
    name === "detect-ultra205.sh" ||
    name === "espflash-tool.sh" ||
    name === "phase13-monitor-capture.sh" ||
    name === "phase13-uart-native-reader.py" ||
    name === "process-group.sh" ||
    name === "serial-session-trace.sh" ||
    name === "serial-session-trace-test.sh" ||
    name.startsWith("phase28.1.1") ||
    name.startsWith("diagnose-ultra205-late-attach") ||
    name.startsWith("diagnose-ultra205-uart-capture") ||
    name.startsWith("ultra205-late-attach") ||
    name.startsWith("ultra205-uart-capture") ||
    name.startsWith("ultra205-transport-qualification")
  );
}

test("active automation contains no untyped script entrypoints", async () => {
  // Arrange
  const root = runfileRoot();
  const entries = await readdir(path.join(root, "scripts"), { withFileTypes: true });

  // Act
  const forbidden = entries
    .filter((entry) => entry.isFile() && automationExtensions.has(path.extname(entry.name)))
    .map((entry) => entry.name)
    .filter((name) => !isAllowedTerminalArtifact(name));

  // Assert
  assert.deepEqual(forbidden, []);
});

test("child process access is isolated to the process adapter", async () => {
  // Arrange
  const root = runfileRoot();
  const sourceRoot = path.join(root, "tools/automation/src");
  const entries = await readdir(sourceRoot, { withFileTypes: true });

  // Act
  const offenders: string[] = [];
  for (const entry of entries) {
    if (!entry.isFile() || !entry.name.endsWith(".ts") || entry.name === "process.ts") continue;
    const source = await readFile(path.join(sourceRoot, entry.name), "utf8");
    if (source.includes("node:child_process")) offenders.push(entry.name);
  }

  // Assert
  assert.deepEqual(offenders, []);
});
