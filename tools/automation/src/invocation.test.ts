import assert from "node:assert/strict";
import test from "node:test";

import { parseInvocation } from "./invocation.js";

test("parser rejects legacy and unsupported monitor syntax", () => {
  // Arrange
  const cases = [
    ["observe-serial", "port=/dev/cu.test"],
    ["observe-serial", "--evidence_mode", "dual"],
    ["observe-serial", "--evidence-mode", "dual"],
  ];

  // Act / Assert
  for (const args of cases) assert.throws(() => parseInvocation(args));
});

test("parser rejects duplicate, unknown, invalid enum, and missing options", () => {
  // Arrange
  const cases = [
    ["observe-serial", "--port", "a", "--port", "b"],
    ["doctor", "--port", "a"],
    ["verify-hardware-surface", "--surface", "overclock", "--request", "request.json"],
    ["capture-version-evidence", "--private-root", "scratch/attempt"],
    ["verify-flash-durability", "--image", "firmware.bin"],
  ];

  // Act / Assert
  for (const args of cases) assert.throws(() => parseInvocation(args));
});

test("parser accepts a complete version evidence request", () => {
  // Act
  const invocation = parseInvocation([
    "capture-version-evidence",
    "--private-root", "scratch/attempt",
    "--package-manifest", "bazel-bin/package.json",
    "--wifi-credentials", "wifi-credentials.json",
    "--port", "/dev/cu.test",
    "--projection", "scratch/version.json",
    "--capture-timeout-seconds", "45",
  ]);

  // Assert
  assert.equal(invocation.command, "capture-version-evidence");
});

test("version evidence requires exactly one detector handoff", () => {
  // Arrange
  const common = [
    "capture-version-evidence",
    "--private-root", "scratch/attempt",
    "--package-manifest", "bazel-bin/package.json",
    "--wifi-credentials", "wifi-credentials.json",
    "--projection", "scratch/version.json",
    "--capture-timeout-seconds", "45",
  ];

  // Act
  const detector = parseInvocation([...common, "--detector-output", "scratch/detector.stdout"]);

  // Assert
  assert.equal(detector.values.get("--detector-output"), "scratch/detector.stdout");
  assert.throws(() => parseInvocation(common));
  assert.throws(() => parseInvocation([...common, "--port", "/dev/cu.test", "--detector-output", "scratch/detector.stdout"]));
});

test("parser accepts bare semantic redaction and rejects removed revision flags", () => {
  // Arrange
  const legacyCases = [
    ["verify-redaction", "--base", "base"],
    ["verify-redaction", "--head", "head"],
    ["verify-redaction", "--new-branch-base", "origin/main"],
  ];

  // Act
  const invocation = parseInvocation(["verify-redaction"]);

  // Assert
  assert.equal(invocation.command, "verify-redaction");
  for (const args of legacyCases) assert.throws(() => parseInvocation(args));
});

test("settings durability capture requires the complete capture surface", () => {
  // Arrange
  const complete = [
    "verify-settings-durability", "--mode", "capture",
    "--private-root", "scratch/settings",
    "--package-manifest", "bazel-bin/package.json",
    "--wifi-credentials", "wifi-credentials.json",
    "--detector-output", "scratch/detector.stdout",
    "--projection", "docs/evidence/settings.json",
    "--capture-timeout-seconds", "360",
  ];

  // Act
  const invocation = parseInvocation(complete);

  // Assert
  assert.equal(invocation.values.get("--mode"), "capture");
  assert.throws(() => parseInvocation(complete.slice(0, -2)));
  assert.throws(() => parseInvocation([...complete, "--trace", "legacy.log"]));
  assert.throws(() => parseInvocation(["verify-settings-durability", "--mode", "baseline"]));
});

test("theme durability requires the complete detector-gated capture surface", () => {
  const invocation = parseInvocation([
    "verify-theme-durability",
    "--private-root", "scratch/theme",
    "--package-manifest", "package.json",
    "--wifi-credentials", "wifi.json",
    "--detector-output", "scratch/detector.stdout",
    "--projection", "docs/evidence/theme.json",
    "--capture-timeout-seconds", "360",
  ]);
  assert.equal(invocation.command, "verify-theme-durability");
  assert.throws(() => parseInvocation(["verify-theme-durability", "--private-root", "scratch/theme"]));
});

test("settings PATCH evidence requires the complete detector-gated surface", () => {
  // Arrange
  const complete = [
    "capture-settings-patch-evidence",
    "--private-root", "scratch/settings-patch",
    "--package-manifest", "package.json",
    "--wifi-credentials", "wifi.json",
    "--detector-output", "scratch/detector.stdout",
    "--projection", "docs/evidence/settings-patch.json",
    "--capture-timeout-seconds", "240",
  ];

  // Act
  const invocation = parseInvocation(complete);

  // Assert
  assert.equal(invocation.command, "capture-settings-patch-evidence");
  assert.throws(() => parseInvocation(complete.slice(0, -2)));
  assert.throws(() => parseInvocation([...complete, "--port", "/dev/cu.private"]));
});

test("log buffer evidence requires the complete detector-gated surface", () => {
  // Arrange
  const complete = [
    "capture-log-buffer-evidence",
    "--private-root", "scratch/log-buffer",
    "--package-manifest", "package.json",
    "--wifi-credentials", "wifi.json",
    "--detector-output", "scratch/detector.stdout",
    "--projection", "docs/evidence/log-buffer.json",
    "--capture-timeout-seconds", "240",
  ];

  // Act
  const invocation = parseInvocation(complete);

  // Assert
  assert.equal(invocation.command, "capture-log-buffer-evidence");
  assert.throws(() => parseInvocation(complete.slice(0, -2)));
  assert.throws(() => parseInvocation([...complete, "--port", "/dev/cu.private"]));
});

test("partition layout evidence requires the complete detector-gated surface", () => {
  // Arrange
  const complete = [
    "capture-partition-layout-evidence",
    "--private-root", "scratch/partition-layout",
    "--package-manifest", "package.json",
    "--wifi-credentials", "wifi.json",
    "--detector-output", "scratch/detector.stdout",
    "--projection", "docs/evidence/partition-layout.json",
    "--capture-timeout-seconds", "360",
  ];

  // Act
  const invocation = parseInvocation(complete);

  // Assert
  assert.equal(invocation.command, "capture-partition-layout-evidence");
  assert.throws(() => parseInvocation(complete.slice(0, -2)));
  assert.throws(() => parseInvocation([...complete, "--port", "/dev/cu.private"]));
});

test("SDK config rollback evidence requires the complete detector-gated surface", () => {
  // Arrange
  const complete = [
    "capture-sdkconfig-rollback-evidence",
    "--private-root", "scratch/sdkconfig-rollback",
    "--package-manifest", "package.json",
    "--rollback-probe-image", "probe.bin",
    "--rollback-probe-metadata", "probe.json",
    "--wifi-credentials", "wifi.json",
    "--detector-output", "scratch/detector.stdout",
    "--projection", "docs/evidence/sdkconfig-rollback.json",
    "--capture-timeout-seconds", "600",
  ];

  // Act
  const invocation = parseInvocation(complete);

  // Assert
  assert.equal(invocation.command, "capture-sdkconfig-rollback-evidence");
  assert.throws(() => parseInvocation(complete.slice(0, -2)));
  assert.throws(() => parseInvocation([...complete, "--port", "/dev/cu.private"]));
});

test("operator snapshot capture requires the detector-gated closed surface", () => {
  // Arrange
  const complete = [
    "capture-operator-snapshot-evidence",
    "--private-root", "scratch/snapshot",
    "--package-manifest", "bazel-bin/package.json",
    "--wifi-credentials", "wifi-credentials.json",
    "--detector-output", "scratch/detector.stdout",
    "--projection", "docs/evidence/snapshot.json",
    "--capture-timeout-seconds", "360",
  ];

  // Act
  const invocation = parseInvocation(complete);

  // Assert
  assert.equal(invocation.command, "capture-operator-snapshot-evidence");
  assert.throws(() => parseInvocation(complete.slice(0, -2)));
  assert.throws(() => parseInvocation([...complete, "--port", "/dev/cu.private"]));
});

test("runtime health capture requires the detector-gated closed surface", () => {
  // Arrange
  const complete = [
    "capture-runtime-health-evidence",
    "--private-root", "scratch/health",
    "--package-manifest", "bazel-bin/package.json",
    "--wifi-credentials", "wifi-credentials.json",
    "--detector-output", "scratch/detector.stdout",
    "--projection", "docs/evidence/health.json",
    "--capture-timeout-seconds", "360",
  ];

  // Act
  const invocation = parseInvocation(complete);

  // Assert
  assert.equal(invocation.command, "capture-runtime-health-evidence");
  assert.throws(() => parseInvocation(complete.slice(0, -2)));
  assert.throws(() => parseInvocation([...complete, "--port", "/dev/cu.private"]));
});

test("system info capture requires the detector-gated closed surface", () => {
  // Arrange
  const complete = [
    "capture-system-info-evidence",
    "--private-root", "scratch/system-info",
    "--package-manifest", "bazel-bin/package.json",
    "--wifi-credentials", "wifi-credentials.json",
    "--detector-output", "scratch/detector.stdout",
    "--projection", "docs/evidence/system-info.json",
    "--capture-timeout-seconds", "360",
  ];

  // Act
  const invocation = parseInvocation(complete);

  // Assert
  assert.equal(invocation.command, "capture-system-info-evidence");
  assert.throws(() => parseInvocation(complete.slice(0, -2)));
  assert.throws(() => parseInvocation([...complete, "--port", "/dev/cu.private"]));
});
