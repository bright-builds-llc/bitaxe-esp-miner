import { readFile, stat } from "node:fs/promises";

import { assertWithinWorkspace } from "./workspace.js";

const UNIX_PORT = /^\/dev\/(?:cu\.usbmodem|cu\.usbserial|ttyUSB|ttyACM)[A-Za-z0-9._-]*$/u;
const WINDOWS_PORT = /^COM[0-9]+$/u;

export class DetectorHandoffError extends Error {
  public readonly category = "evidence_invalid" as const;
  public readonly publicValue = { detector_admitted: false } as const;

  public constructor(message: string) {
    super(message);
    this.name = "DetectorHandoffError";
  }
}

export async function portFromDetectorOutput(workspaceRoot: string, detectorOutput: string): Promise<string> {
  return (await detectorHandoffFromOutput(workspaceRoot, detectorOutput)).port;
}

export type ProvisioningDetectorHandoff = {
  readonly port: string;
  readonly configurationCandidate: string;
};

export async function provisioningDetectorHandoffFromOutput(
  workspaceRoot: string,
  detectorOutput: string,
): Promise<ProvisioningDetectorHandoff> {
  const handoff = await detectorHandoffFromOutput(workspaceRoot, detectorOutput);
  const candidates = handoff.document
    .split(/\r?\n/u)
    .flatMap((line) => line.startsWith("configuration_candidate: ")
      ? [line.slice("configuration_candidate: ".length)]
      : []);
  if (candidates.length !== 1 || !/^Bitaxe_[0-9A-F]{4}$/u.test(candidates[0] ?? "")) {
    throw new DetectorHandoffError(
      "detector output must contain exactly one private configuration candidate",
    );
  }
  return { port: handoff.port, configurationCandidate: candidates[0] as string };
}

async function detectorHandoffFromOutput(
  workspaceRoot: string,
  detectorOutput: string,
): Promise<{ readonly port: string; readonly document: string }> {
  let document: string;
  try {
    const detectorPath = assertWithinWorkspace(workspaceRoot, detectorOutput);
    const metadata = await stat(detectorPath);
    if (!metadata.isFile()) throw new DetectorHandoffError("detector output must be a regular file");
    if ((metadata.mode & 0o777) !== 0o600) {
      throw new DetectorHandoffError("detector output must have mode 0600");
    }
    document = await readFile(detectorPath, "utf8");
  } catch (error) {
    if (error instanceof DetectorHandoffError) throw error;
    throw new DetectorHandoffError("detector output is unavailable or malformed");
  }
  const ports = document
    .split(/\r?\n/u)
    .flatMap((line) => line.startsWith("port: ") ? [line.slice("port: ".length)] : []);
  if (ports.length !== 1) {
    throw new DetectorHandoffError("detector output must contain exactly one admitted port");
  }
  const [port] = ports;
  if (port === undefined || (!UNIX_PORT.test(port) && !WINDOWS_PORT.test(port))) {
    throw new DetectorHandoffError("detector output admitted port is invalid");
  }
  return { port, document };
}
