const operatorSensorSymbol = "bitaxe_firmware::operator_sensor_runtime::run";
const gamma601SafeBootSymbol = "bitaxe_firmware::gamma601::run";
const gamma601PublishSymbol = "bitaxe_firmware::gamma601::observations::Producer::publish";
const gamma601VoutCheckSymbol = "bitaxe_firmware::gamma601::vout_check::run_check";
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
  if (entries.length === 0) {
    const rawWord = body[0]?.match(/^\s*[0-9a-f]+:\s+([0-9a-f]{8})\s*$/iu)?.[1];
    if (rawWord !== undefined) {
      const instruction = Number.parseInt(rawWord.slice(-6), 16);
      if ((instruction & 0xfff) === 0x136) {
        entries.push(String((instruction >>> 12) * 8));
      }
    }
  }
  if (entries.length !== 1) {
    throw new Error(`firmware stack audit requires one entry frame for ${symbol}`);
  }

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

export type Gamma601StackBudget = {
  readonly safeBootFrameBytes: number;
  /** The observation publish, nested under the safe boot's frame on every sweep. */
  readonly publishFrameBytes: number;
  readonly platformReadinessFrameBytes: number;
  /** The core-voltage check's frame, nested under the safe boot's; only in that variant. */
  readonly maybeVoutCheckFrameBytes?: number;
};

/**
 * The Gamma 601 safe boot (ADR-0041) runs its own thread in place of the 205's sensor owner.
 * The core-voltage check variant runs its check on that thread, so its frame is bounded too.
 */
export function verifyGamma601StackBudget(
  disassembly: string,
  options: { readonly voutCheck: boolean } = { voutCheck: false },
): Gamma601StackBudget {
  const safeBootFrameBytes = entryFrameBytes(disassembly, gamma601SafeBootSymbol);
  const publishFrameBytes = entryFrameBytes(disassembly, gamma601PublishSymbol);
  const platformReadinessFrameBytes = entryFrameBytes(disassembly, platformReadinessSymbol);
  if (safeBootFrameBytes > maxIndividualFrameBytes) {
    throw new Error("firmware stack audit found an oversized safe-boot frame");
  }
  if (publishFrameBytes > maxIndividualFrameBytes) {
    throw new Error("firmware stack audit found an oversized observation publish frame");
  }
  if (safeBootFrameBytes + publishFrameBytes > maxCombinedFrameBytes) {
    throw new Error("firmware stack audit found an oversized observation publish path");
  }
  if (platformReadinessFrameBytes > maxPlatformReadinessFrameBytes) {
    throw new Error("firmware stack audit found an oversized platform readiness frame");
  }
  if (!options.voutCheck) return { safeBootFrameBytes, publishFrameBytes, platformReadinessFrameBytes };
  const voutCheckFrameBytes = entryFrameBytes(disassembly, gamma601VoutCheckSymbol);
  if (voutCheckFrameBytes > maxIndividualFrameBytes) {
    throw new Error("firmware stack audit found an oversized core-voltage check frame");
  }
  if (safeBootFrameBytes + voutCheckFrameBytes > maxCombinedFrameBytes) {
    throw new Error("firmware stack audit found an oversized core-voltage check path");
  }
  return {
    safeBootFrameBytes,
    publishFrameBytes,
    platformReadinessFrameBytes,
    maybeVoutCheckFrameBytes: voutCheckFrameBytes,
  };
}
