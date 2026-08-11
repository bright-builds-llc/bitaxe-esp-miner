import net from "node:net";

const RESET_DELIVERY_GRACE_MS = 100;

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
    let settled = false;
    let prefixFlushed = false;
    let resetIssued = false;
    let deliveryGrace: NodeJS.Timeout | undefined;
    const cleanup = (): void => {
      clearTimeout(timeout);
      if (deliveryGrace !== undefined) clearTimeout(deliveryGrace);
    };
    const closeOwnedSocket = (): void => {
      if (socket.destroyed) return;
      if (socket.connecting) {
        socket.destroy();
        return;
      }
      try {
        socket.resetAndDestroy();
      } catch {
        socket.destroy();
      }
    };
    const fail = (error: Error): void => {
      if (settled) return;
      settled = true;
      cleanup();
      closeOwnedSocket();
      reject(error);
    };
    const timeout = setTimeout(() => fail(new Error("interrupted upload timed out")), timeoutMs);
    socket.once("error", (error) => {
      if (!resetIssued) fail(error);
    });
    socket.once("close", () => {
      if (settled) return;
      if (!prefixFlushed || !resetIssued) {
        fail(new Error("interrupted upload connection closed before forced reset"));
        return;
      }
      settled = true;
      cleanup();
      resolve();
    });
    socket.once("connect", () => {
      socket.write(Buffer.concat([headers, body]), () => {
        if (settled) return;
        prefixFlushed = true;
        deliveryGrace = setTimeout(() => {
          if (settled) return;
          if (socket.destroyed) {
            fail(new Error("interrupted upload connection closed before forced reset"));
            return;
          }
          resetIssued = true;
          socket.resetAndDestroy();
        }, RESET_DELIVERY_GRACE_MS);
      });
    });
  });
  return {
    declared_body_bytes: image.length,
    transmitted_body_bytes: body.length,
    connection_closed: true,
  };
}
