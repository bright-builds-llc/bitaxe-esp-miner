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

test("network scan capture requires the detector-gated closed surface", () => {
  // Arrange
  const complete = [
    "capture-network-scan-evidence",
    "--private-root", "scratch/network-scan",
    "--package-manifest", "bazel-bin/package.json",
    "--wifi-credentials", "wifi-credentials.json",
    "--detector-output", "scratch/detector.stdout",
    "--projection", "docs/evidence/network-scan.json",
    "--capture-timeout-seconds", "240",
  ];

  // Act
  const invocation = parseInvocation(complete);

  // Assert
  assert.equal(invocation.command, "capture-network-scan-evidence");
  assert.throws(() => parseInvocation(complete.slice(0, -2)));
  assert.throws(() => parseInvocation([...complete, "--port", "/dev/cu.private"]));
});

test("ASIC initialization projection accepts only sealed source inputs", () => {
  // Arrange
  const complete = [
    "project-asic-initialization-evidence",
    "--attempt-root", "scratch/accepted/attempt-007",
    "--attempt-source-commit", "a".repeat(40),
    "--projection", "docs/evidence/asic-init.json",
  ];

  // Act
  const invocation = parseInvocation(complete);

  // Assert
  assert.equal(invocation.command, "project-asic-initialization-evidence");
  assert.throws(() => parseInvocation(complete.slice(0, -2)));
  assert.throws(() => parseInvocation([...complete, "--port", "/dev/cu.private"]));
});

test("ASIC power initialization projection accepts only the committed source proof", () => {
  // Arrange
  const complete = [
    "project-asic-power-initialization-evidence",
    "--source-projection", "docs/parity/evidence/asic002-initialization/asic-initialization-projection.json",
    "--attempt-source-commit", "a".repeat(40),
    "--projection", "docs/evidence/asic-power-initialization.json",
  ];

  // Act
  const invocation = parseInvocation(complete);

  // Assert
  assert.equal(invocation.command, "project-asic-power-initialization-evidence");
  assert.throws(() => parseInvocation(complete.slice(0, -2)));
  assert.throws(() => parseInvocation([...complete, "--port", "/dev/cu.private"]));
});

test("ASIC work-send projection accepts only the committed source proof", () => {
  // Arrange
  const complete = [
    "project-asic-work-send-evidence",
    "--source-projection", "docs/parity/evidence/asic002-initialization/asic-initialization-projection.json",
    "--attempt-source-commit", "a".repeat(40),
    "--projection", "docs/evidence/asic-work-send.json",
  ];

  // Act
  const invocation = parseInvocation(complete);

  // Assert
  assert.equal(invocation.command, "project-asic-work-send-evidence");
  assert.throws(() => parseInvocation(complete.slice(0, -2)));
  assert.throws(() => parseInvocation([...complete, "--port", "/dev/cu.private"]));
});

test("ASIC result-parsing projection accepts only the committed work-send proof", () => {
  // Arrange
  const complete = [
    "project-asic-result-parsing-evidence",
    "--source-projection", "docs/parity/evidence/asic003-work-send/asic-work-send-projection.json",
    "--attempt-source-commit", "a".repeat(40),
    "--projection", "docs/evidence/asic-result-parsing.json",
  ];

  // Act
  const invocation = parseInvocation(complete);

  // Assert
  assert.equal(invocation.command, "project-asic-result-parsing-evidence");
  assert.throws(() => parseInvocation(complete.slice(0, -2)));
  assert.throws(() => parseInvocation([...complete, "--port", "/dev/cu.private"]));
});

test("ASIC serial-transport projection accepts only both committed source proofs", () => {
  // Arrange
  const complete = [
    "project-asic-serial-transport-evidence",
    "--work-send-projection", "docs/parity/evidence/asic003-work-send/asic-work-send-projection.json",
    "--result-parsing-projection", "docs/parity/evidence/asic004-result-parsing/asic-result-parsing-projection.json",
    "--attempt-source-commit", "a".repeat(40),
    "--projection", "docs/evidence/asic-serial-transport.json",
  ];

  // Act
  const invocation = parseInvocation(complete);

  // Assert
  assert.equal(invocation.command, "project-asic-serial-transport-evidence");
  assert.throws(() => parseInvocation(complete.slice(0, -2)));
  assert.throws(() => parseInvocation([...complete, "--port", "/dev/cu.private"]));
});

test("ASIC frequency-transition projection accepts only the initialization proof", () => {
  // Arrange
  const complete = [
    "project-asic-frequency-transition-evidence",
    "--source-projection", "docs/parity/evidence/asic002-initialization/asic-initialization-projection.json",
    "--attempt-source-commit", "a".repeat(40),
    "--projection", "docs/evidence/asic-frequency-transition.json",
  ];

  // Act
  const invocation = parseInvocation(complete);

  // Assert
  assert.equal(invocation.command, "project-asic-frequency-transition-evidence");
  assert.throws(() => parseInvocation(complete.slice(0, -2)));
  assert.throws(() => parseInvocation([...complete, "--port", "/dev/cu.private"]));
});

test("Stratum socket projection accepts only the initialization proof", () => {
  // Arrange
  const complete = [
    "project-stratum-socket-evidence",
    "--source-projection", "docs/parity/evidence/asic002-initialization/asic-initialization-projection.json",
    "--attempt-source-commit", "a".repeat(40),
    "--projection", "docs/evidence/stratum-socket.json",
  ];

  // Act
  const invocation = parseInvocation(complete);

  // Assert
  assert.equal(invocation.command, "project-stratum-socket-evidence");
  assert.throws(() => parseInvocation(complete.slice(0, -2)));
  assert.throws(() => parseInvocation([...complete, "--port", "/dev/cu.private"]));
});

test("protocol coordinator projection requires the closed four-proof join", () => {
  // Arrange
  const complete = [
    "project-protocol-coordinator-evidence",
    "--initialization-projection", "docs/evidence/initialization.json",
    "--work-send-projection", "docs/evidence/work-send.json",
    "--result-parsing-projection", "docs/evidence/result-parsing.json",
    "--socket-projection", "docs/evidence/socket.json",
    "--attempt-source-commit", "a".repeat(40),
    "--projection", "docs/evidence/protocol-coordinator.json",
  ];

  // Act
  const invocation = parseInvocation(complete);

  // Assert
  assert.equal(invocation.command, "project-protocol-coordinator-evidence");
  assert.throws(() => parseInvocation(complete.slice(0, -2)));
  assert.throws(() => parseInvocation([...complete, "--port", "/dev/cu.private"]));
});

test("mining criteria Bazel wrapper injects its command exactly once", () => {
  // Arrange
  const command = "project-mining-criteria-evidence";
  const callerFlags = [
    "--summary", "docs/evidence/summary.md",
    "--smoke", "docs/evidence/smoke.md",
    "--soak", "docs/evidence/soak.md",
    "--coordinator-projection", "docs/evidence/coordinator.json",
    "--projection", "docs/evidence/mining-criteria.json",
  ];
  const wrapperArgs = [command, ...callerFlags];

  // Act
  const invocation = parseInvocation(wrapperArgs);

  // Assert
  assert.equal(invocation.command, command);
  assert.throws(() => parseInvocation([command, ...wrapperArgs]));
  assert.throws(() => parseInvocation(wrapperArgs.slice(0, -2)));
  assert.throws(() => parseInvocation([...wrapperArgs, "--port", "/dev/cu.private"]));
});

test("Ultra 205 defaults capture requires the detector-gated closed surface", () => {
  // Arrange
  const complete = [
    "capture-ultra205-defaults-evidence",
    "--private-root", "scratch/defaults",
    "--package-manifest", "bazel-bin/package.json",
    "--wifi-credentials", "wifi-credentials.json",
    "--detector-output", "scratch/detector.stdout",
    "--projection", "docs/evidence/defaults.json",
    "--capture-timeout-seconds", "360",
  ];

  // Act
  const invocation = parseInvocation(complete);

  // Assert
  assert.equal(invocation.command, "capture-ultra205-defaults-evidence");
  assert.throws(() => parseInvocation(complete.slice(0, -2)));
  assert.throws(() => parseInvocation([...complete, "--port", "/dev/cu.private"]));
});

test("network reconnect capture requires the detector-gated closed surface", () => {
  // Arrange
  const complete = [
    "capture-network-reconnect-evidence",
    "--private-root", "scratch/net001/attempt-001",
    "--package-manifest", "bazel-bin/package.json",
    "--wifi-credentials", "wifi-credentials.json",
    "--detector-output", "scratch/net001/detector.stdout",
    "--projection", "docs/evidence/network-reconnect.json",
    "--capture-timeout-seconds", "90",
  ];

  // Act
  const invocation = parseInvocation(complete);

  // Assert
  assert.equal(invocation.command, "capture-network-reconnect-evidence");
  assert.throws(() => parseInvocation(complete.slice(0, -2)));
  assert.throws(() => parseInvocation([...complete, "--port", "/dev/cu.private"]));
});

test("provisioning network capture requires the detector-gated closed surface", () => {
  // Arrange
  const complete = [
    "capture-provisioning-network-evidence",
    "--private-root", "scratch/net002/attempt-001",
    "--package-manifest", "bazel-bin/package.json",
    "--wifi-credentials", "wifi-credentials.json",
    "--detector-output", "scratch/net002/detector.stdout",
    "--projection", "docs/evidence/provisioning-network.json",
    "--capture-timeout-seconds", "120",
  ];

  // Act
  const invocation = parseInvocation(complete);

  // Assert
  assert.equal(invocation.command, "capture-provisioning-network-evidence");
  assert.throws(() => parseInvocation(complete.slice(0, -2)));
  assert.throws(() => parseInvocation([...complete, "--port", "/dev/cu.private"]));
});

test("API command effects requires the closed 600-second attempt shape", () => {
  // Arrange
  const complete = [
    "api-command-effects-campaign",
    "--private-root", "scratch/api009/attempt-001",
    "--package-manifest", "bazel-bin/package.json",
    "--wifi-credentials", "wifi-credentials.json",
    "--detector-output", "scratch/api009/detector.stdout",
    "--projection", "docs/evidence/api-command-effects.json",
    "--duration-seconds", "600",
  ];

  // Act
  const invocation = parseInvocation(complete);

  // Assert
  assert.equal(invocation.command, "api-command-effects-campaign");
  assert.equal(invocation.values.get("--duration-seconds"), "600");
  assert.throws(() => parseInvocation(complete.slice(0, -2)));
  assert.throws(() => parseInvocation([...complete.slice(0, -1), "599"]));
  assert.throws(() => parseInvocation([...complete, "--port", "/dev/cu.private"]));
});
