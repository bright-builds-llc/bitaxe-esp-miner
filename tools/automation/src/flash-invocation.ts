import { flashProgram, stringNumber } from "./cli-tools.js";
import { flashMonitorCommand, monitorCommand } from "./contracts.generated.js";
import { hasFlag, InvocationError, maybeOptionValue, type ParsedInvocation } from "./invocation.js";

export function monitorSpec(root: string, invocation: ParsedInvocation) {
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
export function flashDurabilitySpec(root: string, invocation: ParsedInvocation) {
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
