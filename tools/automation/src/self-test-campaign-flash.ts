import { internalCommandSpec } from "./contracts.generated.js";
import type { SelfTestCampaignBoard } from "./self-test-campaign-profile.js";

const monitorSeconds = 360;

/** The board a command names. */
export type FlashTarget = {
  readonly board: SelfTestCampaignBoard;
};

type FlashInputs = {
  readonly flashProgram: string;
  readonly port: string;
  readonly manifest: string;
  readonly wifiCredentials: string;
};

export function selfTestFlashMonitorSpec(
  target: FlashTarget,
  flashProgram: string,
  port: string,
  manifest: string,
  wifiCredentials: string,
  intentPath: string,
  evidenceDir: string,
) {
  return internalCommandSpec(flashProgram, [
    "flash-monitor",
    "--board", String(target.board),
    "--port", port,
    "--manifest", manifest,
    // Seeding NVS is a factory install; the flash tool refuses Wi-Fi without it.
    "--factory-reset",
    "--wifi-credentials", wifiCredentials,
    "--self-test-intent", intentPath,
    "--capture-timeout-seconds", String(monitorSeconds),
    "--evidence-mode", "dual",
    "--evidence-dir", evidenceDir,
  ], value => value);
}

export function selfTestFlashMonitorDryRunSpec(
  target: FlashTarget,
  flashProgram: string,
  port: string,
  manifest: string,
  wifiCredentials: string,
  intentPath: string,
) {
  return internalCommandSpec(flashProgram, [
    "flash-monitor",
    "--board", String(target.board),
    "--port", port,
    "--dry-run",
    "--manifest", manifest,
    // Seeding NVS is a factory install; the flash tool refuses Wi-Fi without it.
    "--factory-reset",
    "--wifi-credentials", wifiCredentials,
    "--self-test-intent", intentPath,
    "--capture-timeout-seconds", String(monitorSeconds),
  ], value => value);
}

export function ordinaryFlashMonitorSpec(
  target: FlashTarget,
  inputs: FlashInputs & { readonly evidenceDir: string },
) {
  return internalCommandSpec(inputs.flashProgram, [
    "flash-monitor",
    "--board", String(target.board),
    "--port", inputs.port,
    "--manifest", inputs.manifest,
    // Seeding NVS is a factory install; the flash tool refuses Wi-Fi without it.
    "--factory-reset",
    "--wifi-credentials", inputs.wifiCredentials,
    "--capture-timeout-seconds", String(monitorSeconds),
    "--evidence-mode", "dual",
    "--evidence-dir", inputs.evidenceDir,
  ], value => value);
}

/** The passive monitor on the campaign's board. */
export function selfTestMonitorSpec(target: FlashTarget, flashProgram: string, port: string) {
  return internalCommandSpec(flashProgram, [
    "monitor",
    "--board", String(target.board),
    "--port", port,
    "--capture-timeout-seconds", String(monitorSeconds),
  ], value => value);
}
