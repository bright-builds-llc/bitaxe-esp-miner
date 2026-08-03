import { readFile, stat } from "node:fs/promises";

import { assertWithinWorkspace } from "./workspace.js";

const UNIX_PORT = /^\/dev\/(?:cu\.usbmodem|cu\.usbserial|ttyUSB|ttyACM)[A-Za-z0-9._-]*$/u;
const WINDOWS_PORT = /^COM[0-9]+$/u;

export async function portFromDetectorOutput(workspaceRoot: string, detectorOutput: string): Promise<string> {
  const detectorPath = assertWithinWorkspace(workspaceRoot, detectorOutput);
  const metadata = await stat(detectorPath);
  if (!metadata.isFile()) throw new Error("detector output must be a regular file");
  if ((metadata.mode & 0o777) !== 0o600) throw new Error("detector output must have mode 0600");
  const document = await readFile(detectorPath, "utf8");
  const ports = document
    .split(/\r?\n/u)
    .flatMap((line) => line.startsWith("port: ") ? [line.slice("port: ".length)] : []);
  if (ports.length !== 1) throw new Error("detector output must contain exactly one admitted port");
  const [port] = ports;
  if (port === undefined || (!UNIX_PORT.test(port) && !WINDOWS_PORT.test(port))) {
    throw new Error("detector output admitted port is invalid");
  }
  return port;
}
