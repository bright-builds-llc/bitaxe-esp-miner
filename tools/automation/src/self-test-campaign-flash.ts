import { internalCommandSpec } from "./contracts.generated.js";

const monitorSeconds = 360;

type FlashInputs = {
  readonly flashProgram: string;
  readonly port: string;
  readonly manifest: string;
  readonly wifiCredentials: string;
};

export function selfTestFlashMonitorSpec(
  flashProgram: string,
  port: string,
  manifest: string,
  wifiCredentials: string,
  intentPath: string,
  evidenceDir: string,
) {
  return internalCommandSpec(flashProgram, [
    "flash-monitor",
    "--board", "205",
    "--port", port,
    "--manifest", manifest,
    "--wifi-credentials", wifiCredentials,
    "--self-test-intent", intentPath,
    "--capture-timeout-seconds", String(monitorSeconds),
    "--evidence-mode", "dual",
    "--evidence-dir", evidenceDir,
  ], value => value);
}

export function selfTestFlashMonitorDryRunSpec(
  flashProgram: string,
  port: string,
  manifest: string,
  wifiCredentials: string,
  intentPath: string,
) {
  return internalCommandSpec(flashProgram, [
    "flash-monitor",
    "--board", "205",
    "--port", port,
    "--dry-run",
    "--manifest", manifest,
    "--wifi-credentials", wifiCredentials,
    "--self-test-intent", intentPath,
    "--capture-timeout-seconds", String(monitorSeconds),
  ], value => value);
}

export function ordinaryFlashMonitorSpec(
  inputs: FlashInputs & { readonly evidenceDir: string },
) {
  return internalCommandSpec(inputs.flashProgram, [
    "flash-monitor",
    "--board", "205",
    "--port", inputs.port,
    "--manifest", inputs.manifest,
    "--wifi-credentials", inputs.wifiCredentials,
    "--capture-timeout-seconds", String(monitorSeconds),
    "--evidence-mode", "dual",
    "--evidence-dir", inputs.evidenceDir,
  ], value => value);
}
