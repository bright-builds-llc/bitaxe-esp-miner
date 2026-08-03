import { constants, existsSync, realpathSync } from "node:fs";
import { access, readFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { buildFirmware } from "./build.js";
import {
  flashMonitorCommand,
  internalCommandSpec,
  monitorCommand,
  type AutomationCategory,
  type AutomationCommand,
  type AutomationResult,
} from "./contracts.generated.js";
import { portFromDetectorOutput } from "./detector.js";
import { fetchJsonFromSameOrigin } from "./http.js";
import {
  hasFlag,
  InvocationError,
  maybeOptionValue,
  optionValue,
  parseInvocation,
  type ParsedInvocation,
} from "./invocation.js";
import { packageFirmware } from "./package.js";
import { createLocalProcessPort, type ProcessPort } from "./process.js";
import { verifySemanticEvidenceRedaction } from "./redaction.js";
import { captureVersionEvidence } from "./version-evidence.js";
import { executeCommandSpec } from "./workflow.js";
import { assertWithinWorkspace } from "./workspace.js";

class PolicyError extends Error {}

function workspaceRoot(): string {
  const maybeWorkspace = process.env["BUILD_WORKSPACE_DIRECTORY"];
  const starts = [
    ...(maybeWorkspace === undefined ? [] : [maybeWorkspace]),
    process.cwd(),
    path.dirname(fileURLToPath(import.meta.url)),
  ];
  for (const start of starts) {
    let candidate = path.resolve(start);
    while (true) {
      const moduleFile = path.join(candidate, "MODULE.bazel");
      if (existsSync(moduleFile)) return path.dirname(realpathSync(moduleFile));
      const parent = path.dirname(candidate);
      if (parent === candidate) break;
      candidate = parent;
    }
  }
  throw new Error("cannot locate the canonical workspace root");
}

function automationResult(
  command: AutomationCommand,
  status: "succeeded" | "failed" | "blocked",
  category: AutomationCategory,
  publicValue?: unknown,
): AutomationResult {
  const base = { schema_version: "bitaxe-automation-result-v1" as const, command, status, category };
  return publicValue === undefined ? base : { ...base, public: publicValue };
}

function safeErrorSummary(error: unknown): string | undefined {
  if (!(error instanceof Error)) return undefined;
  return /^[A-Za-z0-9 _.:()-]+$/u.test(error.message) ? error.message : undefined;
}

function toolProgram(root: string, relative: string): string {
  const maybeRunfiles = process.env["RUNFILES_DIR"];
  return maybeRunfiles === undefined
    ? path.join(root, "bazel-bin", relative)
    : path.join(maybeRunfiles, "_main", relative);
}

function flashProgram(root: string): string {
  return toolProgram(root, "tools/flash/flash");
}

function stringNumber(value: string | undefined): number | undefined {
  return value === undefined ? undefined : Number(value);
}

function monitorSpec(root: string, invocation: ParsedInvocation) {
  const maybePort = maybeOptionValue(invocation, "--port");
  const maybeEvidenceDir = maybeOptionValue(invocation, "--evidence-dir");
  const maybeCaptureTimeout = stringNumber(maybeOptionValue(invocation, "--capture-timeout-seconds"));
  return monitorCommand(flashProgram(root), {
    board: 205,
    ...(maybePort === undefined ? {} : { port: maybePort }),
    ...(maybeEvidenceDir === undefined ? {} : { evidenceDir: maybeEvidenceDir }),
    ...(maybeCaptureTimeout === undefined ? {} : { captureTimeoutSeconds: maybeCaptureTimeout }),
    dryRun: hasFlag(invocation, "--dry-run"),
    redactEvidence: hasFlag(invocation, "--redact-evidence"),
  });
}

function flashDurabilitySpec(root: string, invocation: ParsedInvocation) {
  const maybePort = maybeOptionValue(invocation, "--port");
  const maybeImage = maybeOptionValue(invocation, "--image");
  const maybeManifest = maybeOptionValue(invocation, "--manifest");
  const maybeWifiCredentials = maybeOptionValue(invocation, "--wifi-credentials");
  const maybeEvidenceDir = maybeOptionValue(invocation, "--evidence-dir");
  const maybeCaptureTimeout = stringNumber(maybeOptionValue(invocation, "--capture-timeout-seconds"));
  const common = {
    board: 205,
    ...(maybePort === undefined ? {} : { port: maybePort }),
    ...(maybeWifiCredentials === undefined ? {} : { wifiCredentials: maybeWifiCredentials }),
    ...(maybeEvidenceDir === undefined ? {} : { evidenceDir: maybeEvidenceDir }),
    ...(maybeCaptureTimeout === undefined ? {} : { captureTimeoutSeconds: maybeCaptureTimeout }),
    dryRun: hasFlag(invocation, "--dry-run"),
    redactEvidence: hasFlag(invocation, "--redact-evidence"),
  } as const;
  if (maybeImage === undefined) {
    return flashMonitorCommand(flashProgram(root), {
      ...common,
      ...(maybeManifest === undefined ? {} : { manifest: maybeManifest }),
    });
  }
  if (maybeManifest === undefined) throw new InvocationError("--image requires --manifest");
  return flashMonitorCommand(flashProgram(root), {
    ...common,
    image: maybeImage,
    manifest: maybeManifest,
  });
}

async function typedRequestArguments(root: string, invocation: ParsedInvocation): Promise<string[]> {
  const request = assertWithinWorkspace(root, optionValue(invocation, "--request"));
  const value: unknown = JSON.parse(await readFile(request, "utf8"));
  if (typeof value !== "object" || value === null) throw new InvocationError("typed request must be a JSON object");
  const candidate = value as Record<string, unknown>;
  const workflow = candidate["workflow"];
  if (typeof workflow !== "object" || workflow === null) throw new InvocationError("typed request workflow is missing");
  const identity = workflow as Record<string, unknown>;
  if (identity["schema_version"] !== "bitaxe-workflow-identity-v1") throw new InvocationError("typed request workflow schema is invalid");
  if (typeof identity["command"] !== "string" || typeof identity["request_sha256"] !== "string") {
    throw new InvocationError("typed request workflow identity is incomplete");
  }
  return ["--manifest", request, "--workflow", identity["command"], "--request-sha256", identity["request_sha256"]];
}

async function dispatchProcess(
  root: string,
  invocation: ParsedInvocation,
  processPort: ProcessPort,
): Promise<unknown> {
  let spec;
  switch (invocation.command) {
    case "doctor": {
      const checks = ["cargo", "rustup", "ldproxy", "bazel", "espflash"];
      for (const program of checks) {
        const outcome = await processPort.run(internalCommandSpec(program, ["--version"], (value) => value));
        if (outcome.exitCode !== 0) throw new Error(`required dependency unavailable: ${program}`);
      }
      return { dependencies: "available" };
    }
    case "bootstrap-esp":
      spec = internalCommandSpec("espup", ["install", "--targets", "esp32s3", "--std"], (value) => value);
      break;
    case "verify-reference":
      spec = internalCommandSpec("cargo", ["run", "--quiet", "-p", "xtask", "--", "verify-reference"], (value) => value);
      break;
    case "verify-production-session":
      spec = internalCommandSpec("bazel", ["test", "//crates/bitaxe-stratum:tests", "//crates/bitaxe-api:tests", "//crates/bitaxe-config:tests"], (value) => value);
      break;
    case "observe-serial":
      spec = monitorSpec(root, invocation);
      break;
    case "verify-flash-durability":
      spec = flashDurabilitySpec(root, invocation);
      break;
    case "capture-operator-evidence":
      spec = internalCommandSpec(toolProgram(root, "tools/parity/report"), [
        "operator-evidence", "--profile", "release", "--evidence-root", optionValue(invocation, "--evidence-root"),
        ...(hasFlag(invocation, "--require-redaction-passed") ? ["--require-redaction-passed"] : []),
        ...(hasFlag(invocation, "--require-operator-snapshot-coherence") ? ["--require-operator-snapshot-coherence"] : []),
      ], (value) => value);
      break;
    case "verify-settings-durability":
      spec = internalCommandSpec(toolProgram(root, "tools/parity/report"), ["verify-settings-durability", ...invocation.args], (value) => value);
      break;
    case "capture-correlated-runtime-evidence":
      spec = internalCommandSpec(toolProgram(root, "tools/parity/report"), ["admit-correlated-runtime-evidence", ...invocation.args], (value) => value);
      break;
    case "verify-hardware-surface": {
      const typed = await typedRequestArguments(root, invocation);
      spec = internalCommandSpec(toolProgram(root, "tools/parity/report"), ["safety-allow", ...typed, "--surface", optionValue(invocation, "--surface")], (value) => value);
      break;
    }
    case "verify-mining": {
      const typed = await typedRequestArguments(root, invocation);
      spec = internalCommandSpec(toolProgram(root, "tools/parity/report"), ["mining-allow", ...typed], (value) => value);
      break;
    }
    case "verify-firmware-ota":
    case "verify-web-assets-ota":
    case "verify-recovery":
      await typedRequestArguments(root, invocation);
      throw new PolicyError("the request is typed but no authorized effect adapter is active");
    case "build-firmware":
    case "package-firmware":
    case "verify-redaction":
    case "verify-http-api":
    case "capture-version-evidence":
      throw new Error("specialized workflow reached generic dispatch");
  }
  if (spec.program.includes(path.sep)) await access(spec.program, constants.X_OK);
  await executeCommandSpec(spec, processPort);
  return undefined;
}

async function main(): Promise<number> {
  let invocation: ParsedInvocation;
  try {
    invocation = parseInvocation(process.argv.slice(2));
  } catch (error) {
    const message = error instanceof Error ? error.message : "invalid invocation";
    process.stderr.write(`bitaxe-automation: ${message}\n`);
    process.stdout.write(`${JSON.stringify(automationResult("doctor", "failed", "invalid_invocation"))}\n`);
    return 2;
  }

  const root = workspaceRoot();
  try {
    await access(path.join(root, "MODULE.bazel"), constants.R_OK);
    const processPort = createLocalProcessPort({ cwd: root, timeoutMs: 900_000 });
    let publicValue: unknown;
    if (invocation.command === "build-firmware") {
      await buildFirmware(root, {
        outputDir: optionValue(invocation, "--output-dir"),
        buildProvenanceStamp: optionValue(invocation, "--build-provenance-stamp"),
        identitySdkconfigDefaults: optionValue(invocation, "--identity-sdkconfig-defaults"),
        buildTimestampUtc: optionValue(invocation, "--build-timestamp-utc"),
      }, processPort);
    } else if (invocation.command === "package-firmware") {
      await packageFirmware(root, {
        firmwareElf: optionValue(invocation, "--firmware-elf"),
        buildProvenanceStamp: optionValue(invocation, "--build-provenance-stamp"),
        espIdfSdkconfig: optionValue(invocation, "--esp-idf-sdkconfig"),
        bootloaderBin: optionValue(invocation, "--bootloader-bin"),
        partitionTableBin: optionValue(invocation, "--partition-table-bin"),
        otadataInitialBin: optionValue(invocation, "--otadata-initial-bin"),
        outDir: optionValue(invocation, "--out-dir"),
        manifest: optionValue(invocation, "--manifest"),
      }, processPort, toolProgram(root, "tools/xtask/xtask"));
    } else if (invocation.command === "verify-http-api") {
      const origin = new URL(optionValue(invocation, "--device-url"));
      const output = assertWithinWorkspace(root, optionValue(invocation, "--output"));
      await fetchJsonFromSameOrigin(origin, maybeOptionValue(invocation, "--route") ?? "/api/system/info", output);
    } else if (invocation.command === "capture-version-evidence") {
      const maybeDetectorOutput = maybeOptionValue(invocation, "--detector-output");
      const port = maybeDetectorOutput === undefined
        ? optionValue(invocation, "--port")
        : await portFromDetectorOutput(root, maybeDetectorOutput);
      publicValue = await captureVersionEvidence(root, {
        privateRoot: optionValue(invocation, "--private-root"),
        packageManifest: optionValue(invocation, "--package-manifest"),
        wifiCredentials: optionValue(invocation, "--wifi-credentials"),
        port,
        projection: optionValue(invocation, "--projection"),
        captureTimeoutSeconds: Number(optionValue(invocation, "--capture-timeout-seconds")),
      }, processPort, flashProgram(root), toolProgram(root, "crates/bitaxe-automation-contracts/validate_version_evidence"));
    } else if (invocation.command === "verify-redaction") {
      const evidenceRoot = assertWithinWorkspace(root, maybeOptionValue(invocation, "--evidence-root") ?? "docs/parity/evidence");
      publicValue = await verifySemanticEvidenceRedaction(evidenceRoot);
    } else {
      publicValue = await dispatchProcess(root, invocation, processPort);
    }
    process.stderr.write(`bitaxe-automation: ${invocation.command} completed\n`);
    process.stdout.write(`${JSON.stringify(automationResult(invocation.command, "succeeded", "complete", publicValue))}\n`);
    return 0;
  } catch (error) {
    const policyBlocked = error instanceof PolicyError;
    const invalid = error instanceof InvocationError;
    const category: AutomationCategory = policyBlocked
      ? "policy_blocked"
      : invalid
        ? "invalid_invocation"
        : "process_failed";
    const status = policyBlocked ? "blocked" : "failed";
    const exitCode = policyBlocked ? 3 : invalid ? 2 : 1;
    const maybeSummary = safeErrorSummary(error);
    process.stderr.write(
      `bitaxe-automation: ${invocation.command} ${policyBlocked ? "blocked by policy" : "failed"}${maybeSummary === undefined ? "" : `: ${maybeSummary}`}\n`,
    );
    process.stdout.write(`${JSON.stringify(automationResult(invocation.command, status, category))}\n`);
    return exitCode;
  }
}

process.exitCode = await main();
