// The repository's active lessons: an always-loaded core (`.codex/tasks/lessons.md`) and area files
// under `.codex/tasks/lessons/`, loaded when a task touches their scope (AGENTS.md, Deterministic
// Active-Lesson Loading). The managed Bright Builds check validates the core only, so this check
// validates every active file's blocks, keeps lesson IDs unique across core, areas and archive, and
// holds the repository's active set to the local budget (standards-overrides.md).
import { readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

export const CORE_PATH = ".codex/tasks/lessons.md";
export const AREA_DIR = ".codex/tasks/lessons";
export const ARCHIVE_PATH = ".codex/tasks/lessons.archive.md";
/** The local override of the managed 24,000-byte / 8,000-token budget, over the full active set. */
export const BYTE_BUDGET = 40_000;
export const TOKEN_BUDGET = 13_500;

const HEADING = /^## (lesson-[a-z0-9]+(?:-[a-z0-9]+)*) \| (\d{4}-\d{2}-\d{2}(?: \d{2}:\d{2}(?: [A-Z]{2,5})?)?)$/u;
const FIELD = /^(?:-\s+|\d+\.\s+)(Date|What went wrong|Preventive rule|Trigger signal(?: to catch it earlier)?):\s*(.*)$/u;
const FIELDS = ["Date", "What went wrong", "Preventive rule", "Trigger signal"];

/** The blocks of one active file, and the format findings, as the managed check judges the core. */
export function parseBlocks(path, text) {
  const findings = [];
  const blocks = [];
  let maybeBlock = null;
  text.split(/\r?\n/u).forEach((line, index) => {
    if (!line.startsWith("## ")) { maybeBlock?.lines.push(line); return; }
    const maybeHeading = HEADING.exec(line);
    if (!maybeHeading) { findings.push(`${path}:${index + 1}: invalid lesson heading`); maybeBlock = null; return; }
    maybeBlock = { id: maybeHeading[1], line: index + 1, lines: [] };
    blocks.push(maybeBlock);
  });
  for (const block of blocks) {
    const counts = new Map(FIELDS.map((name) => [name, 0]));
    for (const line of block.lines) {
      const maybeField = FIELD.exec(line);
      if (!maybeField) continue;
      const name = maybeField[1].startsWith("Trigger signal") ? "Trigger signal" : maybeField[1];
      counts.set(name, counts.get(name) + 1);
      if (maybeField[2].trim() === "") findings.push(`${path}:${block.line}: ${block.id} has an empty ${name} field`);
    }
    for (const [name, count] of counts) {
      if (count !== 1) findings.push(`${path}:${block.line}: ${block.id} needs exactly one ${name} field; found ${count}`);
    }
  }
  return { blocks, findings };
}

/** An area file starts with `# Lessons: <title>` and states a non-empty `Scope:` before its first block. */
export function areaHeaderFindings(path, text) {
  const lines = text.split(/\r?\n/u);
  const findings = [];
  if (!/^# Lessons: \S/u.test(lines[0] ?? "")) findings.push(`${path}:1: an area file starts with "# Lessons: <title>"`);
  const firstBlock = lines.findIndex((line) => line.startsWith("## "));
  const preamble = firstBlock === -1 ? lines : lines.slice(0, firstBlock);
  if (!preamble.some((line) => /^Scope: \S/u.test(line))) findings.push(`${path}: an area file states "Scope: ..." before its blocks`);
  return findings;
}

/**
 * Judges the active files (`core`, then `areas`, each `{ path, text }`) and the archive text.
 * Returns the findings and the active set's size.
 */
export function checkLessonAreas({ core, areas, archiveText }) {
  const findings = [];
  const seen = new Map();
  const claim = (id, where) => {
    if (seen.has(id)) findings.push(`${where}: duplicate ${id}; first in ${seen.get(id)}`);
    else seen.set(id, where);
  };
  let bytes = 0, tokens = 0, lessons = 0;
  for (const file of [core, ...areas]) {
    if (file !== core) findings.push(...areaHeaderFindings(file.path, file.text));
    const parsed = parseBlocks(file.path, file.text);
    findings.push(...parsed.findings);
    parsed.blocks.forEach((block) => claim(block.id, `${file.path}:${block.line}`));
    lessons += parsed.blocks.length;
    const size = Buffer.byteLength(file.text);
    bytes += size;
    tokens += Math.ceil(size / 3);
  }
  // Archived IDs are immutable and never reused.
  for (const match of archiveText.matchAll(/^## (lesson-[a-z0-9-]+) \|/gmu)) claim(match[1], ARCHIVE_PATH);
  if (bytes > BYTE_BUDGET || tokens > TOKEN_BUDGET) {
    findings.push(`active lessons exceed the budget (${bytes} bytes, ${tokens} tokens; ${BYTE_BUDGET} / ${TOKEN_BUDGET}): audit before appending`);
  }
  return { findings, summary: { sources: 1 + areas.length, lessons, bytes, estimated_tokens: tokens } };
}

/** Reads the repository's lesson files relative to `root`. */
export function readLessonFiles(root) {
  const read = (path) => ({ path, text: readFileSync(join(root, path), "utf8") });
  const areas = readdirSync(join(root, AREA_DIR)).filter((name) => name.endsWith(".md")).sort()
    .map((name) => read(`${AREA_DIR}/${name}`));
  return { core: read(CORE_PATH), areas, archiveText: read(ARCHIVE_PATH).text };
}

if (process.argv[1] && fileURLToPath(import.meta.url) === process.argv[1]) {
  const { findings, summary } = checkLessonAreas(readLessonFiles(process.cwd()));
  for (const finding of findings) process.stdout.write(`FAIL lessons ${finding}\n`);
  process.stdout.write(`SUMMARY lesson-areas ${Object.entries(summary).map(([key, value]) => `${key}=${value}`).join(" ")} findings=${findings.length}\n`);
  process.exitCode = findings.length === 0 ? 0 : 1;
}
