import net from "node:net";

export type InterruptedUploadObservation = {
  readonly declared_body_bytes: number;
  readonly transmitted_body_bytes: number;
  readonly connection_closed: true;
};

export async function sendInterruptedFirmwareUpload(
  origin: URL,
  image: Buffer,
  prefixBytes: number,
  timeoutMs = 10_000,
): Promise<InterruptedUploadObservation> {
  if (
    origin.protocol !== "http:"
    || origin.username !== ""
    || origin.password !== ""
    || origin.pathname !== "/"
    || origin.search !== ""
    || origin.hash !== ""
  ) {
    throw new Error("interrupted upload requires one origin-only HTTP target");
  }
  if (!Number.isSafeInteger(prefixBytes) || prefixBytes <= 0 || prefixBytes >= image.length) {
    throw new Error("interrupted upload prefix must be a strict image prefix");
  }
  const port = origin.port === "" ? 80 : Number(origin.port);
  if (!Number.isSafeInteger(port) || port <= 0 || port > 65_535) {
    throw new Error("interrupted upload origin port is invalid");
  }
  const body = image.subarray(0, prefixBytes);
  const authority = origin.port === "" ? origin.hostname : origin.host;
  const headers = Buffer.from(
    [
      "POST /api/system/OTA HTTP/1.1",
      `Host: ${authority}`,
      `Origin: ${origin.origin}`,
      "Content-Type: application/octet-stream",
      `Content-Length: ${String(image.length)}`,
      "Connection: close",
      "",
      "",
    ].join("\r\n"),
    "ascii",
  );
  await new Promise<void>((resolve, reject) => {
    const socket = net.createConnection({ host: origin.hostname, port });
    const timeout = setTimeout(() => {
      socket.destroy();
      reject(new Error("interrupted upload timed out"));
    }, timeoutMs);
    socket.once("error", (error) => {
      clearTimeout(timeout);
      reject(error);
    });
    socket.once("connect", () => {
      socket.end(Buffer.concat([headers, body]), () => {
        clearTimeout(timeout);
        resolve();
      });
    });
  });
  return {
    declared_body_bytes: image.length,
    transmitted_body_bytes: body.length,
    connection_closed: true,
  };
}
