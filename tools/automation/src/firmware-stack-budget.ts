const operatorSensorSymbol = "bitaxe_firmware::operator_sensor_runtime::run";
const platformReadinessSymbol = "bitaxe_firmware::runtime_snapshot::collect_platform_readiness_snapshot";
const screenCollectorSymbol = "bitaxe_firmware::runtime_snapshot::screen::collect_screen_snapshot";
const maxIndividualFrameBytes = 3 * 1024;
const maxCombinedFrameBytes = 4 * 1024;
const maxPlatformReadinessFrameBytes = 1024;

export type FirmwareStackBudget = {
  readonly operatorSensorFrameBytes: number;
  readonly platformReadinessFrameBytes: number;
  readonly screenCollectorFrameBytes: number;
  readonly combinedFrameBytes: number;
};

function entryFrameBytes(disassembly: string, symbol: string): number {
  const lines = disassembly.split(/\r?\n/u);
  const headers = lines
    .map((line, index) => ({ line, index }))
    .filter(({ line }) => line.trimEnd().endsWith(`<${symbol}>:`));
  if (headers.length !== 1) throw new Error(`firmware stack audit requires exactly one ${symbol} symbol`);

  const header = headers[0];
  if (header === undefined) throw new Error("firmware stack audit symbol disappeared");
  const nextHeaderOffset = lines
    .slice(header.index + 1)
    .findIndex((line) => /^\s*[0-9a-f]+\s+<.+>:\s*$/iu.test(line));
  const bodyEnd = nextHeaderOffset === -1 ? lines.length : header.index + 1 + nextHeaderOffset;
  const body = lines.slice(header.index + 1, bodyEnd);
  const entries = body
    .map((line) => line.match(/\bentry\s+a1,\s+(0x[0-9a-f]+|[0-9]+)/iu)?.[1])
    .filter((value): value is string => value !== undefined);
  if (entries.length !== 1) throw new Error(`firmware stack audit requires one entry frame for ${symbol}`);

  const encoded = entries[0];
  if (encoded === undefined) throw new Error("firmware stack audit frame disappeared");
  const frameBytes = Number.parseInt(encoded, encoded.startsWith("0x") ? 16 : 10);
  if (!Number.isSafeInteger(frameBytes) || frameBytes <= 0 || frameBytes % 16 !== 0) {
    throw new Error(`firmware stack audit frame is invalid for ${symbol}`);
  }
  return frameBytes;
}

export function verifyFirmwareStackBudget(disassembly: string): FirmwareStackBudget {
  const operatorSensorFrameBytes = entryFrameBytes(disassembly, operatorSensorSymbol);
  const platformReadinessFrameBytes = entryFrameBytes(disassembly, platformReadinessSymbol);
  const screenCollectorFrameBytes = entryFrameBytes(disassembly, screenCollectorSymbol);
  if (operatorSensorFrameBytes > maxIndividualFrameBytes || screenCollectorFrameBytes > maxIndividualFrameBytes) {
    throw new Error("firmware stack audit found an oversized individual frame");
  }
  const combinedFrameBytes = operatorSensorFrameBytes + screenCollectorFrameBytes;
  if (combinedFrameBytes > maxCombinedFrameBytes) {
    throw new Error("firmware stack audit found an oversized operator screen path");
  }
  if (platformReadinessFrameBytes > maxPlatformReadinessFrameBytes) {
    throw new Error("firmware stack audit found an oversized platform readiness frame");
  }
  return {
    operatorSensorFrameBytes,
    platformReadinessFrameBytes,
    screenCollectorFrameBytes,
    combinedFrameBytes,
  };
}
