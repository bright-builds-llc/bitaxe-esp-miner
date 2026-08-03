import { writeFile } from "node:fs/promises";

const MAX_FRAME_BYTES = 256 * 1024;

type WebSocketMessageEvent = {
  readonly data: unknown;
};

export type WebSocketClient = {
  addEventListener(
    type: "message" | "error" | "close",
    listener: (event: WebSocketMessageEvent) => void,
    options?: { once: true },
  ): void;
  close(): void;
};

export type WebSocketFactory = (target: string) => WebSocketClient;

function defaultWebSocketFactory(target: string): WebSocketClient {
  return new globalThis.WebSocket(target) as unknown as WebSocketClient;
}

function websocketTarget(origin: URL, route: string): URL {
  if (
    origin.username !== ""
    || origin.password !== ""
    || origin.pathname !== "/"
    || origin.search !== ""
    || origin.hash !== ""
  ) {
    throw new Error("device origin must be origin-only");
  }
  if (!route.startsWith("/") || route.startsWith("//")) {
    throw new Error("WebSocket route must be same-origin relative");
  }
  const target = new URL(route, origin);
  if (target.origin !== origin.origin) throw new Error("WebSocket target escaped the admitted origin");
  if (target.protocol === "http:") {
    target.protocol = "ws:";
  } else if (target.protocol === "https:") {
    target.protocol = "wss:";
  } else {
    throw new Error("WebSocket origin protocol is invalid");
  }
  return target;
}

function textFrame(data: unknown): string {
  if (typeof data === "string") return data;
  if (Buffer.isBuffer(data)) return data.toString("utf8");
  if (data instanceof ArrayBuffer) return Buffer.from(data).toString("utf8");
  throw new Error("WebSocket frame must be text");
}

export async function captureJsonWebSocketFrame(
  origin: URL,
  route: string,
  privateOutput: string,
  maybeFactory: WebSocketFactory | undefined = undefined,
): Promise<unknown> {
  const target = websocketTarget(origin, route);
  const factory = maybeFactory ?? defaultWebSocketFactory;
  const value = await new Promise<unknown>((resolve, reject) => {
    const client = factory(target.href);
    let settled = false;
    const timeout = setTimeout(() => finish(new Error("same-origin WebSocket frame timed out")), 10_000);

    function finish(result: unknown): void {
      if (settled) return;
      settled = true;
      clearTimeout(timeout);
      try {
        client.close();
      } catch (error) {
        if (!(result instanceof Error)) {
          result = error instanceof Error ? error : new Error("WebSocket cleanup failed");
        }
      }
      if (result instanceof Error) {
        reject(result);
      } else {
        resolve(result);
      }
    }

    client.addEventListener("message", (event) => {
      try {
        const text = textFrame(event.data);
        if (Buffer.byteLength(text) > MAX_FRAME_BYTES) {
          throw new Error("WebSocket frame exceeds the private evidence limit");
        }
        const parsed: unknown = JSON.parse(text);
        if (typeof parsed !== "object" || parsed === null || Array.isArray(parsed)) {
          throw new Error("WebSocket frame must be a JSON object");
        }
        finish(parsed);
      } catch (error) {
        finish(error instanceof Error ? error : new Error("WebSocket frame is invalid"));
      }
    });
    client.addEventListener("error", () => finish(new Error("same-origin WebSocket failed")), { once: true });
    client.addEventListener("close", () => finish(new Error("same-origin WebSocket closed before a frame")), { once: true });
  });
  await writeFile(privateOutput, `${JSON.stringify(value)}\n`, { encoding: "utf8", mode: 0o600, flag: "wx" });
  return value;
}
