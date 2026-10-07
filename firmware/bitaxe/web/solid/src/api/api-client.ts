// Device API client for the SolidJS variant.
//
// Request shapes (paths, methods, headers, bodies and the 15-second timeout)
// match `firmware/bitaxe/static/www/assets/api-client.js` exactly; the shared
// contract test in `test/api-client-contract.test.ts` runs both clients against
// the same fake `fetch`. Browser services are injected so the shell stays
// testable without a DOM.

import type { CommandName } from "../core/workflows.js";

export type ApiFailureCategory = "http" | "timeout" | "unavailable" | "invalid-response" | "invalid-command";

/** Error thrown by the client; `category` drives the public message. */
export type ApiError = Error & { readonly category: ApiFailureCategory; readonly status: number };

const REQUEST_TIMEOUT_MS = 15000;
const COMMANDS: ReadonlySet<string> = new Set(["pause", "resume", "restart"]);

export function apiError(category: ApiFailureCategory, status: number): ApiError {
  return Object.assign(new Error(category), { name: "ApiError", category, status });
}

export function isApiError(error: unknown): error is ApiError {
  return error instanceof Error && error.name === "ApiError" && "category" in error;
}

type MinimalSocket = {
  addEventListener(type: string, listener: (event: { data?: unknown }) => void): void;
  close(): void;
};

/** Browser services the client needs; `browserServices()` binds the real ones. */
export type ApiServices = Readonly<{
  fetch: (path: string, init: RequestInit) => Promise<Response>;
  setTimeout: (callback: () => void, delayMs: number) => unknown;
  clearTimeout: (handle: unknown) => void;
  location: Readonly<{ protocol: string; host: string }>;
  maybeWebSocket: (new (url: string) => MinimalSocket) | null;
  clickDownload: (href: string, fileName: string) => void;
}>;

export type SocketState = "connected" | "closed" | "unavailable";

export type ApiClient = ReturnType<typeof createApiClient>;

export function createApiClient(services: ApiServices) {
  async function request(path: string, options: RequestInit = {}): Promise<Response> {
    const controller = new AbortController();
    const timeout = services.setTimeout(() => controller.abort(), REQUEST_TIMEOUT_MS);
    try {
      const response = await services.fetch(path, {
        ...options,
        credentials: "same-origin",
        headers: { Accept: "application/json", ...((options.headers as Record<string, string> | undefined) ?? {}) },
        signal: controller.signal,
      });
      if (!response.ok) throw apiError("http", response.status);
      return response;
    } catch (error) {
      if (isApiError(error)) throw error;
      if ((error as { name?: unknown } | null)?.name === "AbortError") throw apiError("timeout", 0);
      throw apiError("unavailable", 0);
    } finally {
      services.clearTimeout(timeout);
    }
  }

  async function json(path: string, options?: RequestInit): Promise<unknown> {
    const response = await request(path, options);
    try {
      return await response.json();
    } catch {
      throw apiError("invalid-response", response.status);
    }
  }

  function jsonBody(method: string, body: unknown): RequestInit {
    return { method, headers: { "Content-Type": "application/json" }, body: JSON.stringify(body) };
  }

  function octetStream(file: Blob): RequestInit {
    return { method: "POST", headers: { "Content-Type": "application/octet-stream" }, body: file };
  }

  function openLogStream(onText: (text: string) => void, onState: (state: SocketState) => void): () => void {
    const maybeSocketType = services.maybeWebSocket;
    if (maybeSocketType === null) {
      onState("unavailable");
      return () => {};
    }
    const protocol = services.location.protocol === "https:" ? "wss:" : "ws:";
    const socket = new maybeSocketType(`${protocol}//${services.location.host}/api/ws`);
    socket.addEventListener("open", () => onState("connected"));
    socket.addEventListener("message", (event) => {
      if (typeof event.data === "string") onText(event.data);
    });
    socket.addEventListener("close", () => onState("closed"));
    socket.addEventListener("error", () => onState("unavailable"));
    return () => socket.close();
  }

  return Object.freeze({
    getInfo: () => json("/api/system/info"),
    getScoreboard: () => json("/api/system/scoreboard"),
    patchSettings: (patch: unknown) => json("/api/system", jsonBody("PATCH", patch)),
    getTheme: () => json("/api/theme"),
    saveTheme: (theme: unknown) => json("/api/theme", jsonBody("POST", theme)),
    command: async (name: CommandName): Promise<unknown> => {
      if (!COMMANDS.has(name)) throw apiError("invalid-command", 0);
      return json(`/api/system/${name}`, { method: "POST" });
    },
    retainedLogs: async (): Promise<string> =>
      (await request("/api/system/logs", { headers: { Accept: "text/plain" } })).text(),
    downloadLogs: () => services.clickDownload("/api/system/logs", "bitaxe.log"),
    uploadFirmware: (file: Blob) => request("/api/system/OTA", octetStream(file)),
    uploadWww: (file: Blob) => request("/api/system/OTAWWW", octetStream(file)),
    openLogStream,
  });
}

/** Real browser bindings for `createApiClient`. */
export function browserServices(): ApiServices {
  return Object.freeze({
    fetch: (path: string, init: RequestInit) => globalThis.fetch(path, init),
    setTimeout: (callback: () => void, delayMs: number) => globalThis.setTimeout(callback, delayMs),
    clearTimeout: (handle: unknown) => globalThis.clearTimeout(handle as ReturnType<typeof setTimeout>),
    location: globalThis.location,
    maybeWebSocket: typeof globalThis.WebSocket === "function" ? globalThis.WebSocket : null,
    clickDownload: (href: string, fileName: string) => {
      const anchor = globalThis.document.createElement("a");
      anchor.href = href;
      anchor.download = fileName;
      anchor.rel = "noopener";
      anchor.click();
    },
  });
}
