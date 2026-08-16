import { constants, existsSync, promises, realpathSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { buildFirmware } from "./build.js";
import { captureAdcObservationEvidence } from "./adc-observation-evidence.js";
import { captureApiCommandEffects } from "./api-command-effects.js";
import { emitOperatorCheckpointSignal } from "./api-command-effects-checkpoint.js";
import { deviceSessionProgram, flashProgram, stringNumber, toolProgram } from "./cli-tools.js";
import { captureHashrateMonitorEvidenceFromInvocation } from "./hashrate-monitor-command.js";
import { projectAsicFrequencyTransitionEvidence } from "./asic-frequency-transition-evidence.js";
import { projectAsicInitializationEvidence } from "./asic-initialization-evidence.js";
import { projectAsicPowerInitializationEvidence } from "./asic-power-initialization-evidence.js";
import { captureEmc2101ThermalEvidence } from "./emc2101-thermal-evidence.js";
import { captureEmc2101ThermalFaultFromInvocation } from "./emc2101-thermal-fault-command.js";
import { projectIna260Evidence } from "./ina260-evidence.js";
import { projectAsicResetEvidence } from "./asic-reset-evidence.js";
import { projectAsicResultParsingEvidence } from "./asic-result-parsing-evidence.js";
import { projectAsicSerialTransportEvidence } from "./asic-serial-transport-evidence.js";
import { projectAsicWorkSendEvidence } from "./asic-work-send-evidence.js";
import { projectStratumSocketEvidence } from "./stratum-socket-evidence.js";
import { projectProtocolCoordinatorEvidence } from "./protocol-coordinator-evidence.js";
import { projectMiningCriteriaEvidenceFromInvocation } from "./mining-criteria-evidence.js";
import {
  flashMonitorCommand,
  internalCommandSpec,
  monitorCommand,
  type AutomationCategory,
  type AutomationCommand,
  type AutomationResult,
} from "./contracts.generated.js";
import { portFromDetectorOutput, provisioningDetectorHandoffFromOutput } from "./detector.js";
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
import { packageRollbackProbe } from "./rollback-probe.js";
import { captureLogBufferEvidence } from "./log-buffer-evidence.js";
import { captureNetworkReconnectEvidence } from "./network-reconnect-evidence.js";
import { captureNetworkScanEvidence } from "./network-scan-evidence.js";
import { captureProvisioningNetworkEvidence } from "./provisioning-network-evidence.js";
import { captureOperatorSnapshotEvidence } from "./operator-snapshot-evidence.js";
import { capturePartitionLayoutEvidence } from "./partition-layout-evidence.js";
import { captureSdkconfigRollbackEvidence } from "./sdkconfig-rollback-evidence.js";
import { createLocalProcessPort, type ProcessPort } from "./process.js";
import { verifySemanticEvidenceRedaction } from "./redaction.js";
import { captureRuntimeHealthEvidence } from "./runtime-health-evidence.js";
import * as sealedEvidence from "./sealed-evidence-cli.js";
import { captureSettingsPatchEvidence, captureStatisticsHistoryEvidenceFromInvocation } from "./settings-evidence-cli.js";
import { captureSystemInfoEvidence } from "./system-info-evidence.js";
import { typedRequestArguments } from "./typed-request.js";
import { captureSettingsDurability } from "./settings-durability.js";
import { captureThemeDurability } from "./theme-durability.js";
import { maybeTypedFailureCategory, maybeTypedFailurePublicValue } from "./typed-failure.js";
import { captureUltra205DefaultsEvidence } from "./ultra205-defaults-evidence.js";
import { projectUiWorkflowEvidenceFromInvocation } from "./ui-workflow-cli.js";
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
    case "package-rollback-probe":
    case "verify-redaction":
    case "verify-http-api":
    case "capture-version-evidence":
    case "capture-operator-snapshot-evidence":
    case "capture-runtime-health-evidence":
    case "capture-system-info-evidence":
    case "capture-adc-observation-evidence":
    case "capture-hashrate-monitor-evidence":
    case "capture-emc2101-thermal-evidence":
    case "capture-emc2101-thermal-fault-evidence":
    case "capture-ultra205-defaults-evidence":
    case "capture-settings-patch-evidence":
    case "capture-statistics-history-evidence":
    case "capture-log-buffer-evidence":
    case "capture-partition-layout-evidence":
    case "capture-sdkconfig-rollback-evidence":
    case "capture-network-reconnect-evidence":
    case "capture-network-scan-evidence":
    case "project-asic-initialization-evidence":
    case "project-asic-power-initialization-evidence":
    case "project-core-voltage-control-evidence":
    case "project-display-behavior-evidence":
    case "project-screen-flow-evidence":
    case "project-ina260-evidence":
    case "project-asic-reset-evidence":
    case "project-asic-frequency-transition-evidence":
    case "project-stratum-socket-evidence":
    case "project-protocol-coordinator-evidence":
    case "project-mining-criteria-evidence":
    case "project-asic-work-send-evidence":
    case "project-asic-result-parsing-evidence":
    case "project-asic-serial-transport-evidence":
    case "capture-provisioning-network-evidence":
    case "project-ui-workflow-evidence":
    case "api-command-effects-campaign":
    case "verify-theme-durability":
      throw new Error("specialized workflow reached generic dispatch");
  }
  if (spec.program.includes(path.sep)) await promises.access(spec.program, constants.X_OK);
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
    await promises.access(path.join(root, "MODULE.bazel"), constants.R_OK);
    const processPort = createLocalProcessPort({ cwd: root, timeoutMs: 900_000 });
    let publicValue: unknown;
    if (invocation.command === "build-firmware") {
      await buildFirmware(root, {
        outputDir: optionValue(invocation, "--output-dir"),
        buildProvenanceStamp: optionValue(invocation, "--build-provenance-stamp"),
        identitySdkconfigDefaults: optionValue(invocation, "--identity-sdkconfig-defaults"),
        buildTimestampUtc: optionValue(invocation, "--build-timestamp-utc"),
        buildMode: optionValue(invocation, "--build-mode") as "normal" | "rollback-probe",
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
    } else if (invocation.command === "package-rollback-probe") {
      publicValue = await packageRollbackProbe(root, {
        firmwareElf: optionValue(invocation, "--firmware-elf"),
        buildProvenanceStamp: optionValue(invocation, "--build-provenance-stamp"),
        outputImage: optionValue(invocation, "--output-image"),
        metadata: optionValue(invocation, "--metadata"),
      }, processPort);
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
    } else if (invocation.command === "capture-operator-snapshot-evidence") {
      const maybeDetectorOutput = maybeOptionValue(invocation, "--detector-output");
      const port = maybeDetectorOutput === undefined
        ? optionValue(invocation, "--port")
        : await portFromDetectorOutput(root, maybeDetectorOutput);
      publicValue = await captureOperatorSnapshotEvidence(root, {
        privateRoot: optionValue(invocation, "--private-root"),
        packageManifest: optionValue(invocation, "--package-manifest"),
        wifiCredentials: optionValue(invocation, "--wifi-credentials"),
        port,
        projection: optionValue(invocation, "--projection"),
        captureTimeoutSeconds: Number(optionValue(invocation, "--capture-timeout-seconds")),
      }, processPort, flashProgram(root), toolProgram(root, "tools/parity/report"), deviceSessionProgram(root),
      toolProgram(root, "crates/bitaxe-automation-contracts/validate_operator_snapshot_evidence"));
    } else if (invocation.command === "capture-runtime-health-evidence") {
      const port = await portFromDetectorOutput(root, optionValue(invocation, "--detector-output"));
      publicValue = await captureRuntimeHealthEvidence(root, {
        privateRoot: optionValue(invocation, "--private-root"),
        packageManifest: optionValue(invocation, "--package-manifest"),
        wifiCredentials: optionValue(invocation, "--wifi-credentials"),
        port,
        projection: optionValue(invocation, "--projection"),
        captureTimeoutSeconds: Number(optionValue(invocation, "--capture-timeout-seconds")),
      }, processPort, flashProgram(root), toolProgram(root, "crates/bitaxe-automation-contracts/validate_runtime_health_evidence"));
    } else if (invocation.command === "capture-system-info-evidence") {
      const port = await portFromDetectorOutput(root, optionValue(invocation, "--detector-output"));
      publicValue = await captureSystemInfoEvidence(root, {
        privateRoot: optionValue(invocation, "--private-root"),
        packageManifest: optionValue(invocation, "--package-manifest"),
        wifiCredentials: optionValue(invocation, "--wifi-credentials"),
        port,
        projection: optionValue(invocation, "--projection"),
        captureTimeoutSeconds: Number(optionValue(invocation, "--capture-timeout-seconds")),
      }, processPort, flashProgram(root), toolProgram(root, "crates/bitaxe-automation-contracts/validate_system_info_evidence"));
    } else if (invocation.command === "capture-adc-observation-evidence") {
      const detectorOutput = optionValue(invocation, "--detector-output");
      const port = await portFromDetectorOutput(root, detectorOutput);
      const options = {
        privateRoot: optionValue(invocation, "--private-root"), packageManifest: optionValue(invocation, "--package-manifest"),
        wifiCredentials: optionValue(invocation, "--wifi-credentials"), detectorOutput, port,
        projection: optionValue(invocation, "--projection"), captureTimeoutSeconds: Number(optionValue(invocation, "--capture-timeout-seconds")),
      };
      publicValue = await captureAdcObservationEvidence(root, options, processPort, flashProgram(root), "git",
        toolProgram(root, "crates/bitaxe-automation-contracts/validate_system_info_evidence"),
        toolProgram(root, "crates/bitaxe-automation-contracts/validate_adc_observation_inputs"),
        toolProgram(root, "crates/bitaxe-automation-contracts/validate_adc_observation_evidence"));
    } else if (invocation.command === "capture-hashrate-monitor-evidence") {
      publicValue = await captureHashrateMonitorEvidenceFromInvocation(root, invocation, processPort);
    } else if (invocation.command === "capture-emc2101-thermal-evidence") {
      const detectorOutput = optionValue(invocation, "--detector-output");
      const port = await portFromDetectorOutput(root, detectorOutput);
      publicValue = await captureEmc2101ThermalEvidence(root, {
        privateRoot: optionValue(invocation, "--private-root"),
        packageManifest: optionValue(invocation, "--package-manifest"),
        wifiCredentials: optionValue(invocation, "--wifi-credentials"),
        detectorOutput,
        port,
        projection: optionValue(invocation, "--projection"),
        captureTimeoutSeconds: Number(optionValue(invocation, "--capture-timeout-seconds")),
      }, processPort, flashProgram(root), "git",
      toolProgram(root, "crates/bitaxe-automation-contracts/validate_system_info_evidence"),
      toolProgram(root, "crates/bitaxe-automation-contracts/validate_emc2101_thermal_inputs"),
      toolProgram(root, "crates/bitaxe-automation-contracts/validate_emc2101_thermal_evidence"));
    } else if (invocation.command === "capture-emc2101-thermal-fault-evidence") {
      publicValue = await captureEmc2101ThermalFaultFromInvocation(root, invocation, processPort);
    } else if (invocation.command === "capture-ultra205-defaults-evidence") {
      const port = await portFromDetectorOutput(root, optionValue(invocation, "--detector-output"));
      publicValue = await captureUltra205DefaultsEvidence(root, {
        privateRoot: optionValue(invocation, "--private-root"),
        packageManifest: optionValue(invocation, "--package-manifest"),
        wifiCredentials: optionValue(invocation, "--wifi-credentials"),
        port,
        projection: optionValue(invocation, "--projection"),
        captureTimeoutSeconds: Number(optionValue(invocation, "--capture-timeout-seconds")),
      }, processPort, flashProgram(root),
      toolProgram(root, "crates/bitaxe-automation-contracts/validate_system_info_evidence"),
      toolProgram(root, "crates/bitaxe-automation-contracts/validate_ultra205_defaults_evidence"));
    } else if (invocation.command === "capture-settings-patch-evidence") {
      const port = await portFromDetectorOutput(root, optionValue(invocation, "--detector-output"));
      publicValue = await captureSettingsPatchEvidence(root, {
        privateRoot: optionValue(invocation, "--private-root"),
        packageManifest: optionValue(invocation, "--package-manifest"),
        wifiCredentials: optionValue(invocation, "--wifi-credentials"),
        port,
        projection: optionValue(invocation, "--projection"),
        captureTimeoutSeconds: Number(optionValue(invocation, "--capture-timeout-seconds")),
      }, processPort, flashProgram(root), toolProgram(root, "crates/bitaxe-automation-contracts/validate_settings_patch_evidence"));
    } else if (invocation.command === "capture-statistics-history-evidence") {
      publicValue = await captureStatisticsHistoryEvidenceFromInvocation(root, invocation, processPort);
    } else if (invocation.command === "capture-log-buffer-evidence") {
      const port = await portFromDetectorOutput(root, optionValue(invocation, "--detector-output"));
      publicValue = await captureLogBufferEvidence(root, {
        privateRoot: optionValue(invocation, "--private-root"),
        packageManifest: optionValue(invocation, "--package-manifest"),
        wifiCredentials: optionValue(invocation, "--wifi-credentials"),
        port,
        projection: optionValue(invocation, "--projection"),
        captureTimeoutSeconds: Number(optionValue(invocation, "--capture-timeout-seconds")),
      }, processPort, flashProgram(root), toolProgram(root, "crates/bitaxe-automation-contracts/validate_log_buffer_evidence"));
    } else if (invocation.command === "capture-partition-layout-evidence") {
      const port = await portFromDetectorOutput(root, optionValue(invocation, "--detector-output"));
      publicValue = await capturePartitionLayoutEvidence(root, {
        privateRoot: optionValue(invocation, "--private-root"),
        packageManifest: optionValue(invocation, "--package-manifest"),
        wifiCredentials: optionValue(invocation, "--wifi-credentials"),
        port,
        projection: optionValue(invocation, "--projection"),
        captureTimeoutSeconds: Number(optionValue(invocation, "--capture-timeout-seconds")),
      }, processPort, flashProgram(root), deviceSessionProgram(root),
      toolProgram(root, "crates/bitaxe-automation-contracts/validate_partition_layout_evidence"));
    } else if (invocation.command === "capture-sdkconfig-rollback-evidence") {
      const port = await portFromDetectorOutput(root, optionValue(invocation, "--detector-output"));
      publicValue = await captureSdkconfigRollbackEvidence(root, {
        privateRoot: optionValue(invocation, "--private-root"),
        packageManifest: optionValue(invocation, "--package-manifest"),
        rollbackProbeImage: optionValue(invocation, "--rollback-probe-image"),
        rollbackProbeMetadata: optionValue(invocation, "--rollback-probe-metadata"),
        wifiCredentials: optionValue(invocation, "--wifi-credentials"),
        port,
        projection: optionValue(invocation, "--projection"),
        captureTimeoutSeconds: Number(optionValue(invocation, "--capture-timeout-seconds")),
      }, processPort, flashProgram(root), deviceSessionProgram(root),
      toolProgram(root, "crates/bitaxe-automation-contracts/validate_sdkconfig_rollback_evidence"));
    } else if (invocation.command === "capture-network-reconnect-evidence") {
      const port = await portFromDetectorOutput(root, optionValue(invocation, "--detector-output"));
      publicValue = await captureNetworkReconnectEvidence(root, {
        privateRoot: optionValue(invocation, "--private-root"),
        packageManifest: optionValue(invocation, "--package-manifest"),
        wifiCredentials: optionValue(invocation, "--wifi-credentials"),
        port,
        projection: optionValue(invocation, "--projection"),
        captureTimeoutSeconds: Number(optionValue(invocation, "--capture-timeout-seconds")),
      }, processPort, flashProgram(root),
      toolProgram(root, "crates/bitaxe-automation-contracts/validate_network_reconnect_evidence"));
    } else if (invocation.command === "project-asic-initialization-evidence") {
      publicValue = await projectAsicInitializationEvidence(root, {
        attemptRoot: optionValue(invocation, "--attempt-root"),
        attemptSourceCommit: optionValue(invocation, "--attempt-source-commit"),
        projection: optionValue(invocation, "--projection"),
      }, processPort, "git", toolProgram(root,
        "crates/bitaxe-automation-contracts/validate_asic_initialization_evidence"));
    } else if (invocation.command === "project-ui-workflow-evidence") {
      publicValue = await projectUiWorkflowEvidenceFromInvocation(root, invocation, processPort);
    } else if (invocation.command === "project-asic-power-initialization-evidence") {
      publicValue = await projectAsicPowerInitializationEvidence(root, {
        sourceProjection: optionValue(invocation, "--source-projection"), attemptSourceCommit: optionValue(invocation, "--attempt-source-commit"), projection: optionValue(invocation, "--projection"),
      }, processPort, "git", toolProgram(root, "crates/bitaxe-automation-contracts/validate_asic_initialization_evidence"),
      toolProgram(root, "crates/bitaxe-automation-contracts/validate_asic_power_initialization_evidence"));
    } else if (invocation.command === "project-core-voltage-control-evidence") {
      publicValue = await sealedEvidence.projectCoreVoltageControlEvidenceFromInvocation(root, invocation, processPort);
    } else if (invocation.command === "project-display-behavior-evidence") {
      publicValue = await sealedEvidence.projectDisplayBehaviorEvidenceFromInvocation(root, invocation, processPort);
    } else if (invocation.command === "project-screen-flow-evidence") {
      publicValue = await sealedEvidence.projectScreenFlowEvidenceFromInvocation(root, invocation, processPort);
    } else if (invocation.command === "project-ina260-evidence") {
      publicValue = await projectIna260Evidence(root, {
        attemptRoot: optionValue(invocation, "--attempt-root"),
        sourceProjection: optionValue(invocation, "--source-projection"),
        attemptSourceCommit: optionValue(invocation, "--attempt-source-commit"),
        projection: optionValue(invocation, "--projection"),
      }, processPort, "git",
      toolProgram(root, "crates/bitaxe-automation-contracts/validate_system_info_evidence"),
      toolProgram(root, "crates/bitaxe-automation-contracts/validate_ina260_evidence"));
    } else if (invocation.command === "project-asic-reset-evidence") {
      publicValue = await projectAsicResetEvidence(root, {
        sourceProjection: optionValue(invocation, "--source-projection"), attemptSourceCommit:
          optionValue(invocation, "--attempt-source-commit"), projection: optionValue(invocation, "--projection"),
      }, processPort, "git", toolProgram(root,
        "crates/bitaxe-automation-contracts/validate_asic_initialization_evidence"), toolProgram(root,
        "crates/bitaxe-automation-contracts/validate_asic_reset_evidence"));
    } else if (invocation.command === "project-asic-frequency-transition-evidence") {
      publicValue = await projectAsicFrequencyTransitionEvidence(root, {
        sourceProjection: optionValue(invocation, "--source-projection"), attemptSourceCommit:
          optionValue(invocation, "--attempt-source-commit"), projection: optionValue(invocation, "--projection"),
      }, processPort, "git", toolProgram(root,
        "crates/bitaxe-automation-contracts/validate_asic_initialization_evidence"), toolProgram(root,
        "crates/bitaxe-automation-contracts/validate_asic_frequency_transition_evidence"));
    } else if (invocation.command === "project-stratum-socket-evidence") {
      publicValue = await projectStratumSocketEvidence(root, {
        sourceProjection: optionValue(invocation, "--source-projection"),
        attemptSourceCommit: optionValue(invocation, "--attempt-source-commit"),
        projection: optionValue(invocation, "--projection"),
      }, processPort, "git",
      toolProgram(root, "crates/bitaxe-automation-contracts/validate_asic_initialization_evidence"),
      toolProgram(root, "crates/bitaxe-automation-contracts/validate_stratum_socket_evidence"));
    } else if (invocation.command === "project-protocol-coordinator-evidence") {
      publicValue = await projectProtocolCoordinatorEvidence(root, {
        initializationProjection: optionValue(invocation, "--initialization-projection"),
        workSendProjection: optionValue(invocation, "--work-send-projection"),
        resultParsingProjection: optionValue(invocation, "--result-parsing-projection"),
        socketProjection: optionValue(invocation, "--socket-projection"),
        attemptSourceCommit: optionValue(invocation, "--attempt-source-commit"),
        projection: optionValue(invocation, "--projection"),
      }, processPort, "git", {
        initialization: toolProgram(root,
          "crates/bitaxe-automation-contracts/validate_asic_initialization_evidence"),
        workSend: toolProgram(root,
          "crates/bitaxe-automation-contracts/validate_asic_work_send_evidence"),
        resultParsing: toolProgram(root,
          "crates/bitaxe-automation-contracts/validate_asic_result_parsing_evidence"),
        socket: toolProgram(root,
          "crates/bitaxe-automation-contracts/validate_stratum_socket_evidence"),
        evidence: toolProgram(root,
          "crates/bitaxe-automation-contracts/validate_protocol_coordinator_evidence"),
      });
    } else if (invocation.command === "project-mining-criteria-evidence") {
      publicValue = await projectMiningCriteriaEvidenceFromInvocation(
        root, invocation, processPort, toolProgram,
      );
    } else if (invocation.command === "project-asic-work-send-evidence") {
      publicValue = await projectAsicWorkSendEvidence(root, {
        sourceProjection: optionValue(invocation, "--source-projection"),
        attemptSourceCommit: optionValue(invocation, "--attempt-source-commit"),
        projection: optionValue(invocation, "--projection"),
      }, processPort, "git",
      toolProgram(root, "crates/bitaxe-automation-contracts/validate_asic_initialization_evidence"),
      toolProgram(root, "crates/bitaxe-automation-contracts/validate_asic_work_send_evidence"));
    } else if (invocation.command === "project-asic-result-parsing-evidence") {
      publicValue = await projectAsicResultParsingEvidence(root, {
        sourceProjection: optionValue(invocation, "--source-projection"),
        attemptSourceCommit: optionValue(invocation, "--attempt-source-commit"),
        projection: optionValue(invocation, "--projection"),
      }, processPort, "git",
      toolProgram(root, "crates/bitaxe-automation-contracts/validate_asic_work_send_evidence"),
      toolProgram(root, "crates/bitaxe-automation-contracts/validate_asic_result_parsing_evidence"));
    } else if (invocation.command === "project-asic-serial-transport-evidence") {
      publicValue = await projectAsicSerialTransportEvidence(root, {
        workSendProjection: optionValue(invocation, "--work-send-projection"),
        resultParsingProjection: optionValue(invocation, "--result-parsing-projection"),
        attemptSourceCommit: optionValue(invocation, "--attempt-source-commit"),
        projection: optionValue(invocation, "--projection"),
      }, processPort, "git",
      toolProgram(root, "crates/bitaxe-automation-contracts/validate_asic_work_send_evidence"),
      toolProgram(root, "crates/bitaxe-automation-contracts/validate_asic_result_parsing_evidence"),
      toolProgram(root, "crates/bitaxe-automation-contracts/validate_asic_serial_transport_evidence"));
    } else if (invocation.command === "capture-network-scan-evidence") {
      const port = await portFromDetectorOutput(root, optionValue(invocation, "--detector-output"));
      publicValue = await captureNetworkScanEvidence(root, {
        privateRoot: optionValue(invocation, "--private-root"),
        packageManifest: optionValue(invocation, "--package-manifest"),
        wifiCredentials: optionValue(invocation, "--wifi-credentials"),
        port,
        projection: optionValue(invocation, "--projection"),
        captureTimeoutSeconds: Number(optionValue(invocation, "--capture-timeout-seconds")),
      }, processPort, flashProgram(root),
      toolProgram(root, "crates/bitaxe-automation-contracts/validate_network_scan_evidence"));
    } else if (invocation.command === "capture-provisioning-network-evidence") {
      const handoff = await provisioningDetectorHandoffFromOutput(
        root,
        optionValue(invocation, "--detector-output"),
      );
      publicValue = await captureProvisioningNetworkEvidence(root, {
        privateRoot: optionValue(invocation, "--private-root"),
        packageManifest: optionValue(invocation, "--package-manifest"),
        wifiCredentials: optionValue(invocation, "--wifi-credentials"),
        port: handoff.port,
        configurationCandidate: handoff.configurationCandidate,
        projection: optionValue(invocation, "--projection"),
        captureTimeoutSeconds: Number(optionValue(invocation, "--capture-timeout-seconds")),
      }, processPort, flashProgram(root),
      toolProgram(root, "crates/bitaxe-automation-contracts/validate_provisioning_network_evidence"));
    } else if (invocation.command === "api-command-effects-campaign") {
      const port = await portFromDetectorOutput(root, optionValue(invocation, "--detector-output"));
      publicValue = await captureApiCommandEffects(root, {
        privateRoot: optionValue(invocation, "--private-root"),
        packageManifest: optionValue(invocation, "--package-manifest"),
        wifiCredentials: optionValue(invocation, "--wifi-credentials"),
        port,
        projection: optionValue(invocation, "--projection"),
        durationSeconds: Number(optionValue(invocation, "--duration-seconds")),
      }, processPort, toolProgram(root, "scripts/api_command_effects_stratum_pool_/api_command_effects_stratum_pool"), flashProgram(root), deviceSessionProgram(root), emitOperatorCheckpointSignal);
    } else if (invocation.command === "verify-settings-durability" && optionValue(invocation, "--mode") === "capture") {
      const port = await portFromDetectorOutput(root, optionValue(invocation, "--detector-output"));
      publicValue = await captureSettingsDurability(root, {
        privateRoot: optionValue(invocation, "--private-root"),
        packageManifest: optionValue(invocation, "--package-manifest"),
        wifiCredentials: optionValue(invocation, "--wifi-credentials"),
        port,
        projection: optionValue(invocation, "--projection"),
        captureTimeoutSeconds: Number(optionValue(invocation, "--capture-timeout-seconds")),
      }, processPort, flashProgram(root), toolProgram(root, "tools/parity/report"), deviceSessionProgram(root));
    } else if (invocation.command === "verify-theme-durability") {
      const port = await portFromDetectorOutput(root, optionValue(invocation, "--detector-output"));
      publicValue = await captureThemeDurability(root, {
        privateRoot: optionValue(invocation, "--private-root"),
        packageManifest: optionValue(invocation, "--package-manifest"),
        wifiCredentials: optionValue(invocation, "--wifi-credentials"),
        port,
        projection: optionValue(invocation, "--projection"),
        captureTimeoutSeconds: Number(optionValue(invocation, "--capture-timeout-seconds")),
      }, processPort, flashProgram(root), toolProgram(root, "tools/parity/report"), deviceSessionProgram(root));
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
    const maybeTypedCategory = maybeTypedFailureCategory(error);
    const category: AutomationCategory = policyBlocked
      ? "policy_blocked"
      : invalid
        ? "invalid_invocation"
        : maybeTypedCategory ?? "process_failed";
    const blocked = policyBlocked || category === "hardware_blocked";
    const status = blocked ? "blocked" : "failed";
    const exitCode = blocked ? 3 : invalid ? 2 : 1;
    const maybeSummary = safeErrorSummary(error);
    process.stderr.write(
      `bitaxe-automation: ${invocation.command} ${blocked ? "blocked" : "failed"}${maybeSummary === undefined ? "" : `: ${maybeSummary}`}\n`,
    );
    process.stdout.write(`${JSON.stringify(automationResult(
      invocation.command,
      status,
      category,
      maybeTypedFailurePublicValue(error),
    ))}\n`);
    return exitCode;
  }
}

process.exitCode = await main();
