import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { chmod, mkdtemp, mkdir, readFile, stat, unlink, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import { createLocalProcessPort } from "./process.js";
import {
  projectUiWorkflowEvidence,
  UiWorkflowEvidenceError,
  type UiWorkflowValidators,
} from "./ui-workflow-evidence.js";

const sourceCommit = "a".repeat(40);
const referenceCommit = "c1915b0a63bfabebdb95a515cedfee05146c1d50";
const appElfSha256 = "b".repeat(64);
const wwwSpiffsSha256 = "c".repeat(64);
const routes = ["dashboard", "network", "pool", "settings", "logs", "update", "theme"] as const;
const browserArtifactKinds = [
  ...routes.map((route) => `desktop-${route}`),
  ...routes.map((route) => `mobile-${route}`),
  "mobile-navigation-open",
  "mobile-navigation-closed",
  "write-only-secrets",
  "update-guard",
  "console",
  "network",
] as const;

const copiedFiles = [
  "TASKS.md",
  "docs/parity/work-plans/20260813T045300Z-UI-004/PLAN.md",
  "docs/parity/work-plans/20260804T190000Z-UI-004/RESULT.md",
  "tools/automation/src/static-ui.test.ts",
  "docs/parity/evidence/api010-theme-durability/theme-durability-projection.json",
  "docs/parity/evidence/api003-settings-patch/settings-patch-projection.json",
  "docs/parity/evidence/log001-retained-stream/log-buffer-projection.json",
  "docs/parity/evidence/rel001-ota-slot/partition-layout-projection.json",
  "docs/parity/evidence/rel002-sdkconfig-rollback/sdkconfig-rollback-projection.json",
] as const;

function runfileRoot(): string {
  const maybeRunfiles = process.env["RUNFILES_DIR"];
  return maybeRunfiles === undefined
    ? (process.env["BUILD_WORKSPACE_DIRECTORY"] ?? process.cwd())
    : path.join(maybeRunfiles, "_main");
}

async function copyFixtureFile(root: string, relative: string): Promise<void> {
  const destination = path.join(root, relative);
  await mkdir(path.dirname(destination), { recursive: true });
  await writeFile(destination, await readFile(path.join(runfileRoot(), relative)));
}

function packageManifest() {
  return {
    schema_version: 3,
    source_commit: sourceCommit,
    reference_commit: referenceCommit,
    app_elf_sha256: appElfSha256,
    image_metadata: { board: "205" },
    artifacts: [{ kind: "www_spiffs_image", sha256: wwwSpiffsSha256 }],
  };
}

function operatorSnapshot(packageManifestSha256: string) {
  return {
    schema_version: "bitaxe-operator-snapshot-evidence-v1",
    board: 205,
    source_commit: sourceCommit,
    reference_commit: referenceCommit,
    package_manifest_sha256: packageManifestSha256,
    restart_session: {
      terminal_category: "ready",
      request_attempt_count: 1,
      software_reset_observed: true,
      cleanup_complete: true,
    },
    mining_state: "disabled",
    hardware_control_state: "disabled",
    cleanup_complete: true,
    redaction_status: "passed",
  };
}

async function browserAttestation(browserRoot: string, mobileRouteCount = 7) {
  const artifacts = await Promise.all(browserArtifactKinds.map(async (kind) => {
    const relativePath = `artifacts/${kind}.txt`;
    const artifactPath = path.join(browserRoot, relativePath);
    const document = `${kind}\n`;
    await mkdir(path.dirname(artifactPath), { mode: 0o700, recursive: true });
    await chmod(path.dirname(artifactPath), 0o700);
    await writeFile(artifactPath, document, { mode: 0o600 });
    await chmod(artifactPath, 0o600);
    return { kind, relative_path: relativePath, sha256: createHash("sha256").update(document).digest("hex") };
  }));
  return {
    schema_version: "bitaxe-ui-browser-attestation-v1",
    source_commit: sourceCommit,
    reference_commit: referenceCommit,
    app_elf_sha256: appElfSha256,
    www_spiffs_sha256: wwwSpiffsSha256,
    routes,
    artifacts,
    browser: {
      desktop_route_count: 7,
      mobile_route_count: mobileRouteCount,
      same_origin_requests_observed: true,
      log_websocket_observed: true,
      mobile_navigation_opened: true,
      mobile_navigation_closed: true,
      write_only_secrets_blank: true,
      no_file_update_disabled: true,
      otawww_unavailable: true,
      console_error_count: 0,
      unexpected_request_failure_count: 0,
      desktop_viewport_observed: true,
      mobile_viewport_observed: true,
      browser_cleanup_complete: true,
    },
  };
}

async function fixture(name: string, mobileRouteCount = 7) {
  const root = await mkdtemp(path.join(os.tmpdir(), `bitaxe-ui-workflow-${name}-`));
  await writeFile(path.join(root, "MODULE.bazel"), "module(name = \"fixture\")\n");
  await Promise.all(copiedFiles.map(async (relative) => copyFixtureFile(root, relative)));
  const packagePath = path.join(root, "bazel-bin/firmware/bitaxe/package.json");
  await mkdir(path.dirname(packagePath), { recursive: true });
  const packageDocument = `${JSON.stringify(packageManifest(), null, 2)}\n`;
  await writeFile(packagePath, packageDocument);
  const packageDigest = (await import("node:crypto")).createHash("sha256")
    .update(packageDocument).digest("hex");
  const operatorPath = path.join(root, "scratch/ui004-live-workflows/wrapper-001/operator.private.json");
  await mkdir(path.dirname(operatorPath), { mode: 0o700, recursive: true });
  await chmod(path.dirname(operatorPath), 0o700);
  await writeFile(operatorPath, `${JSON.stringify(operatorSnapshot(packageDigest), null, 2)}\n`, { mode: 0o600 });
  await chmod(operatorPath, 0o600);
  const privateRoot = path.join(root, "scratch/ui004-live-workflows/attempt-001");
  await mkdir(privateRoot, { mode: 0o700, recursive: true });
  await chmod(privateRoot, 0o700);
  await writeFile(path.join(privateRoot, "capture.private.json"), "{}\n", { mode: 0o600 });
  const browserRoot = path.join(root, "output/playwright/ui004-attempt-001");
  await mkdir(browserRoot, { mode: 0o700, recursive: true });
  await chmod(browserRoot, 0o700);
  const browserPath = path.join(browserRoot, "browser-attestation.private.json");
  await writeFile(browserPath, `${JSON.stringify(await browserAttestation(browserRoot, mobileRouteCount), null, 2)}\n`, { mode: 0o600 });
  await chmod(browserPath, 0o600);
  const child = path.join(root, "child.mjs");
  await writeFile(child, [
    "#!/bin/sh",
    `if [ "$1" = "rev-parse" ] && [ "$2" = "HEAD" ]; then printf '%s\\n' '${sourceCommit}'`,
    `elif [ "$1" = "-C" ]; then printf '%s\\n' '${referenceCommit}'`,
    "fi",
    "",
  ].join("\n"));
  await chmod(child, 0o700);
  const projection = path.join(root, "docs/parity/evidence/ui004-live-workflows/ui-workflow-projection.json");
  const validators: UiWorkflowValidators = {
    operatorSnapshot: child,
    settings: child,
    log: child,
    partition: child,
    rollback: child,
    evidence: child,
  };
  return {
    root,
    packagePath,
    operatorPath,
    privateRoot,
    browserPath,
    child,
    projection,
    validators,
    firstBrowserArtifact: path.join(browserRoot, "artifacts", `${browserArtifactKinds[0]}.txt`),
  };
}

test("real child projection joins exact package browser and prior evidence", async () => {
  // Arrange
  const value = await fixture("complete");

  // Act
  const evidence = await projectUiWorkflowEvidence(value.root, {
    privateRoot: value.privateRoot,
    packageManifest: value.packagePath,
    operatorSnapshotProjection: value.operatorPath,
    browserAttestation: value.browserPath,
    projection: value.projection,
  }, createLocalProcessPort({ cwd: value.root, timeoutMs: 5_000 }), value.child, value.validators);

  // Assert
  assert.equal(evidence.schema_version, "bitaxe-ui-workflow-evidence-v1");
  assert.equal(evidence.browser.desktop_route_count, 7);
  assert.equal(evidence.browser.mobile_route_count, 7);
  assert.equal((await stat(value.projection)).mode & 0o777, 0o644);
  const publicDocument = await readFile(value.projection, "utf8");
  assert.doesNotMatch(publicDocument, /https?:|hostname|ssid|usb|password|\/Users\//iu);
});

test("incomplete mobile route quorum withholds public evidence", async () => {
  // Arrange
  const value = await fixture("route-missing", 6);

  // Act
  const outcome = projectUiWorkflowEvidence(value.root, {
    privateRoot: value.privateRoot,
    packageManifest: value.packagePath,
    operatorSnapshotProjection: value.operatorPath,
    browserAttestation: value.browserPath,
    projection: value.projection,
  }, createLocalProcessPort({ cwd: value.root, timeoutMs: 5_000 }), value.child, value.validators);

  // Assert
  await assert.rejects(outcome, (error: unknown) =>
    error instanceof UiWorkflowEvidenceError && error.category === "evidence_invalid");
  await assert.rejects(stat(value.projection), { code: "ENOENT" });
});

test("missing private browser artifact withholds public evidence", async () => {
  // Arrange
  const value = await fixture("artifact-missing");
  await unlink(value.firstBrowserArtifact);

  // Act
  const outcome = projectUiWorkflowEvidence(value.root, {
    privateRoot: value.privateRoot,
    packageManifest: value.packagePath,
    operatorSnapshotProjection: value.operatorPath,
    browserAttestation: value.browserPath,
    projection: value.projection,
  }, createLocalProcessPort({ cwd: value.root, timeoutMs: 5_000 }), value.child, value.validators);

  // Assert
  await assert.rejects(outcome, (error: unknown) =>
    error instanceof UiWorkflowEvidenceError && error.category === "evidence_invalid");
  await assert.rejects(stat(value.projection), { code: "ENOENT" });
});
