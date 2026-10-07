import assert from "node:assert/strict";
import test from "node:test";

import {
  LOG_TEXT_LIMIT,
  appendBoundedLog,
  commandPrompt,
  isAdmittedUpload,
  isCommandName,
  maybeValidatedProvenance,
  visibleLogText,
} from "../src/core/workflows.js";

test("streamed logs keep only the newest characters", () => {
  // Arrange
  const existing = "a".repeat(LOG_TEXT_LIMIT);

  // Act
  const bounded = appendBoundedLog(existing, "tail");

  // Assert
  assert.equal(bounded.length, LOG_TEXT_LIMIT);
  assert.ok(bounded.endsWith("tail"));
});

test("log filters keep matching lines and report an empty view", () => {
  // Arrange
  const text = "I (1) wifi: up\nW (2) pool: retry\nI (3) wifi: rssi";

  // Act
  const filtered = visibleLogText(text, "wifi");
  const unmatched = visibleLogText(text, "asic");

  // Assert
  assert.equal(filtered, "I (1) wifi: up\nI (3) wifi: rssi");
  assert.equal(unmatched, "No matching logs.");
  assert.equal(visibleLogText("", ""), "No matching logs.");
});

test("uploads admit only the exact upstream file names", () => {
  // Arrange
  const candidates = ["esp-miner.bin", "www.bin", "esp-miner (1).bin", null];

  // Act
  const firmware = candidates.map((name) => isAdmittedUpload("firmware", name));
  const www = candidates.map((name) => isAdmittedUpload("www", name));

  // Assert
  assert.deepEqual(firmware, [true, false, false, false]);
  assert.deepEqual(www, [false, true, false, false]);
});

test("device commands are bounded and each has a confirmation prompt", () => {
  // Arrange
  const names = ["pause", "resume", "restart"] as const;

  // Act
  const prompts = names.map(commandPrompt);

  // Assert
  assert.deepEqual(prompts, ["Pause mining?", "Resume mining?", "Restart the device now?"]);
  assert.equal(isCommandName("erase"), false);
});

test("provenance links only a well-formed commit and otherwise stays unavailable", () => {
  // Arrange
  const payload = {
    semanticVersion: "0.1.0",
    sourceCommit: "0123456789abcdef0123456789abcdef01234567",
    buildTimestampUtc: "2026-10-07T12:00:00Z",
    sourceDirty: true,
  };

  // Act
  const provenance = maybeValidatedProvenance(payload);
  const rejected = maybeValidatedProvenance({ ...payload, sourceCommit: "javascript:alert(1)" });

  // Assert
  assert.deepEqual(provenance, {
    version: "0.1.0",
    commitLabel: "0123456789ab (dirty)",
    commitUrl: "https://github.com/bright-builds-llc/bitaxe-esp-miner/commit/0123456789abcdef0123456789abcdef01234567",
    built: "2026-10-07T12:00:00Z",
  });
  assert.equal(rejected, null);
  assert.equal(maybeValidatedProvenance(null), null);
});
