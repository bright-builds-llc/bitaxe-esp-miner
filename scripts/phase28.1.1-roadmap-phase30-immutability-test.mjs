#!/usr/bin/env node
import assert from "node:assert/strict";
import { chmodSync, mkdirSync, mkdtempSync, readFileSync, rmSync, statSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import "./phase28.1.1-hardware-attempt-state-test.mjs";
import "./phase28.1.1-strict-production-evidence-test.mjs";
import {
  PROTECTED_EVIDENCE_FILES,
  checkFrontmatter,
  checkZeroFrontmatter,
  compareProtected,
  extractPhase30Projection,
  snapshotProtected,
} from "./phase28.1.1-roadmap-phase30-immutability.mjs";

const tempRoot = mkdtempSync(join(tmpdir(), "phase28-phase30-guard-"));
process.on("exit", () => rmSync(tempRoot, { recursive: true, force: true }));

function roadmapFixture(sectionSuffix = "") {
  return `# Roadmap

## Phase Details

### Phase 29: Earlier
**Plans**: 3 plans

### Phase 30: Live Share Outcome And Verified Promotion
**Goal**: exact goal${sectionSuffix}
**Depends on**: Phase 29
**Requirements**: STR-09, CFG-07, ASIC-11
**Plans**: 0 plans

## Progress

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 30. Live Share Outcome And Verified Promotion | v1.1 | 0/4 | Not started | — |
`;
}

function expectFailure(action, pattern) {
  assert.throws(action, pattern);
}

function testPhase30Boundaries() {
  // Arrange
  const fixture = roadmapFixture();

  // Act
  const projection = extractPhase30Projection(fixture);

  // Assert
  assert.equal(projection.requirements, "**Requirements**: STR-09, CFG-07, ASIC-11");
  assert.equal(projection.plans, "**Plans**: 0 plans");
  assert.match(projection.progress_row, /^\| 30\. /);
  assert.notEqual(
    projection.section_sha256,
    extractPhase30Projection(roadmapFixture(" changed")).section_sha256,
  );
  expectFailure(() => extractPhase30Projection(fixture.replace("### Phase 30:", "### Phase 31:")), /missing/);
  const adjacent = fixture.replace("\n## Progress", "\n### Phase 31: Adjacent\n**Plans**: 0 plans\n\n## Progress");
  assert.equal(extractPhase30Projection(adjacent).section_sha256, projection.section_sha256);
}

function testFrontmatterChecks() {
  // Arrange
  const valid = join(tempRoot, "valid.md");
  const missing = join(tempRoot, "missing.md");
  const extra = join(tempRoot, "extra.md");
  const misplaced = join(tempRoot, "misplaced.md");
  const roadmap = join(tempRoot, "ROADMAP.md");
  writeFileSync(valid, "---\nphase: test\n---\n# Body\n");
  writeFileSync(missing, "# Body\n");
  writeFileSync(extra, "---\nphase: test\n---\n# Body\n---\n");
  writeFileSync(misplaced, "# Before\n---\nphase: test\n---\n");
  writeFileSync(roadmap, roadmapFixture());

  // Act / Assert
  checkFrontmatter([valid]);
  checkZeroFrontmatter([roadmap]);
  expectFailure(() => checkFrontmatter([missing]), /frontmatter/);
  expectFailure(() => checkFrontmatter([extra]), /frontmatter/);
  expectFailure(() => checkFrontmatter([misplaced]), /frontmatter/);
  writeFileSync(roadmap, `${roadmapFixture()}\n---\n`);
  expectFailure(() => checkZeroFrontmatter([roadmap]), /frontmatterless/);
}

function testProtectedSet() {
  // Arrange
  const phaseDir = join(tempRoot, "phase");
  const privateRoot = join(
    tempRoot,
    "hardware-runs/phase28.1.1/attempt-control/run",
  );
  const manifest = join(privateRoot, "protected.json");
  mkdirSync(phaseDir);
  mkdirSync(privateRoot, { recursive: true });
  for (const name of PROTECTED_EVIDENCE_FILES) {
    writeFileSync(join(phaseDir, name), `${name}\n`);
  }

  // Act
  snapshotProtected(phaseDir, manifest);

  // Assert
  assert.equal(statSync(manifest).mode & 0o777, 0o600);
  compareProtected(phaseDir, manifest);
  const changed = PROTECTED_EVIDENCE_FILES[0];
  writeFileSync(join(phaseDir, changed), "changed\n");
  expectFailure(() => compareProtected(phaseDir, manifest), /changed/);
  writeFileSync(join(phaseDir, changed), `${changed}\n`);
  const parsed = JSON.parse(readFileSync(manifest, "utf8"));
  parsed.files.extra = "0".repeat(64);
  const extraManifest = join(privateRoot, "extra-protected.json");
  writeFileSync(extraManifest, JSON.stringify(parsed));
  chmodSync(extraManifest, 0o600);
  expectFailure(() => compareProtected(phaseDir, extraManifest), /member set/);
  rmSync(join(phaseDir, PROTECTED_EVIDENCE_FILES[1]));
  expectFailure(() => compareProtected(phaseDir, manifest), /unavailable/);
}

testPhase30Boundaries();
testFrontmatterChecks();
testProtectedSet();

console.log("phase28.1.1 roadmap Phase 30 immutability tests: passed");
