(function installBitaxeApiClient(global) {
  "use strict";

  class ApiError extends Error {
    constructor(category, status) {
      super(category);
      this.name = "ApiError";
      this.category = category;
      this.status = status;
    }
  }

  async function request(path, options = {}) {
    const controller = new AbortController();
    const timeout = global.setTimeout(() => controller.abort(), 15000);
    try {
      const response = await global.fetch(path, {
        ...options,
        credentials: "same-origin",
        headers: {
          Accept: "application/json",
          ...(options.headers ?? {}),
        },
        signal: controller.signal,
      });
      if (!response.ok) {
        throw new ApiError("http", response.status);
      }
      return response;
    } catch (error) {
      if (error instanceof ApiError) {
        throw error;
      }
      if (error?.name === "AbortError") {
        throw new ApiError("timeout", 0);
      }
      throw new ApiError("unavailable", 0);
    } finally {
      global.clearTimeout(timeout);
    }
  }

  async function json(path, options) {
    const response = await request(path, options);
    try {
      return await response.json();
    } catch {
      throw new ApiError("invalid-response", response.status);
    }
  }

  function jsonBody(method, body) {
    return {
      method,
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(body),
    };
  }

  async function getInfo() {
    return json("/api/system/info");
  }

  async function getScoreboard() {
    return json("/api/system/scoreboard");
  }

  async function patchSettings(patch) {
    return json("/api/system", jsonBody("PATCH", patch));
  }

  async function getTheme() {
    return json("/api/theme");
  }

  async function saveTheme(theme) {
    return json("/api/theme", jsonBody("POST", theme));
  }

  async function command(name) {
    const allowed = new Set(["pause", "resume", "restart"]);
    if (!allowed.has(name)) {
      throw new ApiError("invalid-command", 0);
    }
    return json(`/api/system/${name}`, { method: "POST" });
  }

  async function retainedLogs() {
    const response = await request("/api/system/logs", {
      headers: { Accept: "text/plain" },
    });
    return response.text();
  }

  function downloadLogs() {
    const anchor = global.document.createElement("a");
    anchor.href = "/api/system/logs";
    anchor.download = "bitaxe.log";
    anchor.rel = "noopener";
    anchor.click();
  }

  async function uploadFirmware(file) {
    return request("/api/system/OTA", {
      method: "POST",
      headers: { "Content-Type": "application/octet-stream" },
      body: file,
    });
  }

  function openLogStream(onText, onState) {
    if (typeof global.WebSocket !== "function") {
      onState("unavailable");
      return () => {};
    }
    const protocol = global.location.protocol === "https:" ? "wss:" : "ws:";
    const socket = new global.WebSocket(`${protocol}//${global.location.host}/api/ws`);
    socket.addEventListener("open", () => onState("connected"));
    socket.addEventListener("message", (event) => {
      if (typeof event.data === "string") {
        onText(event.data);
      }
    });
    socket.addEventListener("close", () => onState("closed"));
    socket.addEventListener("error", () => onState("unavailable"));
    return () => socket.close();
  }

  global.BitaxeApi = Object.freeze({
    ApiError,
    command,
    downloadLogs,
    getInfo,
    getScoreboard,
    getTheme,
    openLogStream,
    patchSettings,
    retainedLogs,
    saveTheme,
    uploadFirmware,
  });
})(globalThis);
