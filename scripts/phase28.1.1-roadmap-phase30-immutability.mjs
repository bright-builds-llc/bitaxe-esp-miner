#!/usr/bin/env node
import { createHash } from "node:crypto";
import { chmodSync, readFileSync, writeFileSync } from "node:fs";
import { basename, join, resolve } from "node:path";
import { pathToFileURL } from "node:url";

export const PROTECTED_SET_ID = "phase28.1.1-prior-evidence-v1";
export const PROTECTED_EVIDENCE_FILES = Object.freeze([
  "28.1.1-accepted-state-baseline-redacted.md",
  "28.1.1-accepted-state-disposition-redacted.md",
  "28.1.1-accepted-state-lifecycle-redacted.md",
  "28.1.1-final-share-evidence-redacted.md",
  "28.1.1-init-parser-fixes-redacted.md",
  "28.1.1-job-hypothesis-runs-redacted.md",
  "28.1.1-live-share-outcome-redacted.md",
  "28.1.1-run-postfix-rust-redacted.md",
  "28.1.1-run-wire-parity-redacted.md",
  "28.1.1-upstream-wire-capture-redacted.md",
  "28.1.1-wire-diff-summary.md",
]);

function fail(message) {
  throw new Error(`phase30_immutability_error: ${message}`);
}

function sha256(contents) {
  return createHash("sha256").update(contents).digest("hex");
}

function read(path) {
  try {
    return readFileSync(path, "utf8");
  } catch {
    fail(`required file is unavailable: ${basename(path)}`);
  }
}

export function extractPhase30Projection(roadmap) {
  const lines = roadmap.split(/\n/);
  const start = lines.findIndex((line) => line.startsWith("### Phase 30:"));
  if (start < 0) fail("Phase 30 section is missing");
  let end = lines.length;
  for (let index = start + 1; index < lines.length; index += 1) {
    if (lines[index].startsWith("### Phase ") || lines[index].startsWith("## ")) {
      end = index;
      break;
    }
  }
  const section = lines.slice(start, end).join("\n");
  const requirements = lines
    .slice(start, end)
    .find((line) => line.startsWith("**Requirements**:"));
  const plans = lines
    .slice(start, end)
    .find((line) => line.startsWith("**Plans**:"));
  const progressRows = lines.filter((line) => /^\| 30\. /.test(line));
  if (!requirements || !plans || progressRows.length !== 1) {
    fail("Phase 30 requirements, plans, or progress projection is ambiguous");
  }
  return {
    section_sha256: sha256(section),
    requirements,
    plans,
    progress_row: progressRows[0],
  };
}

export function checkFrontmatter(paths) {
  if (paths.length === 0) fail("check-frontmatter requires explicit files");
  for (const path of paths) {
    const contents = read(path);
    const delimiters = contents.split(/\n/).filter((line) => line === "---");
    if (!contents.startsWith("---\n") || delimiters.length !== 2) {
      fail(`${basename(path)} must start with exactly one frontmatter block`);
    }
  }
}

export function checkZeroFrontmatter(paths) {
  if (paths.length === 0) fail("check-zero-frontmatter requires explicit files");
  for (const path of paths) {
    const delimiters = read(path).split(/\n/).filter((line) => line === "---");
    if (delimiters.length !== 0) {
      fail(`${basename(path)} must remain frontmatterless`);
    }
  }
}

export function snapshotProtected(phaseDir, outputPath) {
  const normalizedOutput = resolve(outputPath);
  if (!normalizedOutput.includes("/hardware-runs/phase28.1.1/attempt-control/")) {
    fail("protected manifest must stay in the ignored private attempt root");
  }
  const files = Object.fromEntries(
    PROTECTED_EVIDENCE_FILES.map((name) => [name, sha256(read(join(phaseDir, name)))]),
  );
  writeFileSync(
    normalizedOutput,
    `${JSON.stringify({ schema_version: PROTECTED_SET_ID, files }, null, 2)}\n`,
    { mode: 0o600 },
  );
  chmodSync(normalizedOutput, 0o600);
}

export function compareProtected(phaseDir, manifestPath) {
  const manifest = JSON.parse(read(manifestPath));
  const expectedKeys = ["files", "schema_version"].sort();
  if (
    !manifest ||
    typeof manifest !== "object" ||
    Object.keys(manifest).sort().join("\0") !== expectedKeys.join("\0") ||
    manifest.schema_version !== PROTECTED_SET_ID ||
    !manifest.files ||
    Object.keys(manifest.files).sort().join("\0") !==
      [...PROTECTED_EVIDENCE_FILES].sort().join("\0")
  ) {
    fail("protected manifest schema or member set changed");
  }
  for (const name of PROTECTED_EVIDENCE_FILES) {
    if (sha256(read(join(phaseDir, name))) !== manifest.files[name]) {
      fail(`protected evidence changed: ${name}`);
    }
  }
}

function parseFiles(args) {
  const marker = args.indexOf("--files");
  if (marker < 0 || marker === args.length - 1) fail("explicit --files are required");
  return args.slice(marker + 1);
}

function option(args, name) {
  const index = args.indexOf(name);
  if (index < 0 || index === args.length - 1) fail(`${name} is required`);
  return args[index + 1];
}

function maybeOption(args, name) {
  const index = args.indexOf(name);
  if (index < 0) return null;
  if (index === args.length - 1) fail(`${name} is required`);
  return args[index + 1];
}

function protectedOptions(args, action) {
  const setId = maybeOption(args, "--set") ?? PROTECTED_SET_ID;
  if (setId !== PROTECTED_SET_ID) fail("protected set ID changed");
  const phaseDir =
    maybeOption(args, "--phase-dir") ??
    resolve(
      ".planning/phases/28.1.1-bm1366-nonce-production-wire-parity",
    );
  const path =
    action === "snapshot"
      ? (maybeOption(args, "--output") ?? maybeOption(args, "--out"))
      : (maybeOption(args, "--baseline") ?? maybeOption(args, "--manifest"));
  if (!path) {
    fail(
      action === "snapshot"
        ? "--output is required"
        : "--baseline is required",
    );
  }
  return { phaseDir, path };
}

function main(args) {
  const [command, ...rest] = args;
  switch (command) {
    case "snapshot-phase30": {
      const output = option(rest, "--out");
      writeFileSync(output, `${JSON.stringify(extractPhase30Projection(read(option(rest, "--roadmap"))), null, 2)}\n`, { mode: 0o600 });
      chmodSync(output, 0o600);
      break;
    }
    case "compare-phase30": {
      const current = extractPhase30Projection(read(option(rest, "--roadmap")));
      const baseline = JSON.parse(read(option(rest, "--baseline")));
      if (JSON.stringify(current) !== JSON.stringify(baseline)) fail("Phase 30 projection changed");
      break;
    }
    case "check-frontmatter": checkFrontmatter(parseFiles(rest)); break;
    case "check-zero-frontmatter": checkZeroFrontmatter(parseFiles(rest)); break;
    case "snapshot-protected": {
      const { phaseDir, path } = protectedOptions(rest, "snapshot");
      snapshotProtected(phaseDir, path);
      break;
    }
    case "compare-protected": {
      const { phaseDir, path } = protectedOptions(rest, "compare");
      compareProtected(phaseDir, path);
      break;
    }
    default: fail("unknown command");
  }
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  try {
    main(process.argv.slice(2));
  } catch (error) {
    console.error(error.message);
    process.exit(1);
  }
}
