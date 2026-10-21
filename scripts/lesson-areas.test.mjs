import assert from "node:assert/strict";
import test from "node:test";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { BYTE_BUDGET, checkLessonAreas, readLessonFiles } from "./lesson-areas.mjs";

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "..");

const block = (id) => `## ${id} | 2026-10-21

1. Date: 2026-10-21
2. What went wrong: Something.
3. Preventive rule: Do better.
4. Trigger signal: A sign.
`;
const area = (path, ...ids) => ({ path, text: `# Lessons: Area\n\nScope: some work.\n\n${ids.map(block).join("\n")}` });
const core = (...ids) => ({ path: "core.md", text: ids.map(block).join("\n") });

test("the repository's lesson files pass", () => {
  // Arrange
  const files = readLessonFiles(ROOT);
  // Act
  const { findings, summary } = checkLessonAreas(files);
  // Assert
  assert.deepEqual(findings, []);
  assert.ok(summary.sources >= 2);
});

test("a valid core and area pass", () => {
  // Arrange
  const files = { core: core("lesson-a"), areas: [area("a.md", "lesson-b")], archiveText: "" };
  // Act
  const { findings, summary } = checkLessonAreas(files);
  // Assert
  assert.deepEqual(findings, []);
  assert.equal(summary.lessons, 2);
});

test("an ID repeated across files fails", () => {
  // Arrange
  const files = { core: core("lesson-a"), areas: [area("a.md", "lesson-a")], archiveText: "" };
  // Act
  const { findings } = checkLessonAreas(files);
  // Assert
  assert.equal(findings.length, 1);
  assert.match(findings[0], /duplicate lesson-a/u);
});

test("an archived ID cannot be reused", () => {
  // Arrange
  const files = { core: core("lesson-a"), areas: [], archiveText: "# Archived lessons\n\n## lesson-a | 2026-01-01\n" };
  // Act
  const { findings } = checkLessonAreas(files);
  // Assert
  assert.match(findings[0], /duplicate lesson-a/u);
});

test("an area file without a scope fails", () => {
  // Arrange
  const files = { core: core("lesson-a"), areas: [{ path: "a.md", text: `# Lessons: Area\n\n${block("lesson-b")}` }], archiveText: "" };
  // Act
  const { findings } = checkLessonAreas(files);
  // Assert
  assert.deepEqual(findings, ["a.md: an area file states \"Scope: ...\" before its blocks"]);
});

test("a block missing a field fails", () => {
  // Arrange
  const files = { core: { path: "core.md", text: block("lesson-a").replace(/^3\. Preventive rule.*\n/mu, "") }, areas: [], archiveText: "" };
  // Act
  const { findings } = checkLessonAreas(files);
  // Assert
  assert.deepEqual(findings, ["core.md:1: lesson-a needs exactly one Preventive rule field; found 0"]);
});

test("an active set over the budget fails", () => {
  // Arrange
  const padding = "x".repeat(BYTE_BUDGET);
  const files = { core: { path: "core.md", text: `${block("lesson-a")}${padding}\n` }, areas: [], archiveText: "" };
  // Act
  const { findings } = checkLessonAreas(files);
  // Assert
  assert.equal(findings.length, 1);
  assert.match(findings[0], /exceed the budget/u);
});
