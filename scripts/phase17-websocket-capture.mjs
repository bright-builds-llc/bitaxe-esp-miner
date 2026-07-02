#!/usr/bin/env node
import { writeFileSync } from "node:fs";

const allowedWebSocketPaths = new Set(["/api/ws", "/api/ws/live"]);
const maxDurationMs = 30_000;
const maxAllowedFrames = 10;

function parseArgs(argv) {
  const args = {
    maybeDeviceUrl: "",
    path: "/api/ws/live",
    out: "",
    durationMs: 5_000,
    maxFrames: 3,
  };

  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    const next = argv[index + 1];

    switch (arg) {
      case "--device-url":
        if (!next) {
          throw new Error("--device-url requires a value");
        }
        args.maybeDeviceUrl = next;
        index += 1;
        break;
      case "--path":
        if (!next) {
          throw new Error("--path requires a value");
        }
        args.path = next;
        index += 1;
        break;
      case "--out":
        if (!next) {
          throw new Error("--out requires a value");
        }
        args.out = next;
        index += 1;
        break;
      case "--duration-ms":
        if (!next || !/^[0-9]+$/.test(next)) {
          throw new Error("--duration-ms requires a numeric value");
        }
        args.durationMs = Number(next);
        index += 1;
        break;
      case "--max-frames":
        if (!next || !/^[0-9]+$/.test(next)) {
          throw new Error("--max-frames requires a numeric value");
        }
        args.maxFrames = Number(next);
        index += 1;
        break;
      case "-h":
      case "--help":
        console.log(
          "usage: phase17-websocket-capture.mjs --device-url URL --path /api/ws|/api/ws/live --out PATH [--duration-ms N] [--max-frames N]",
        );
        process.exit(0);
        break;
      default:
        throw new Error(`unknown argument: ${arg}`);
    }
  }

  return args;
}

function redactText(value) {
  return String(value)
    .replace(
      /"(ssid|wifiPass|wifiPassword|stratumUser|stratumPassword|stratumCert|poolUrl|fallbackPoolUrl|hostname|ip|ipAddress|gateway|netmask|dns|token|apiToken|apiKey|password|nvsSecret|secret)"\s*:\s*"[^"]*"/gi,
      '"$1":"[redacted]"',
    )
    .replace(
      /"(stratumPort|fallbackStratumPort)"\s*:\s*[0-9]+/gi,
      '"$1":[redacted]',
    )
    .replace(/https?:\/\/[^\s"<>]+/gi, "[redacted-url]")
    .replace(/wss?:\/\/[^\s"<>]+/gi, "[redacted-url]")
    .replace(/\b(?:\d{1,3}\.){3}\d{1,3}\b/g, "[redacted-ip]")
    .replace(/\b(?:[a-f0-9]{2}:){5}[a-f0-9]{2}\b/gi, "[redacted-mac]");
}

function maybeParseOriginDeviceUrl(value) {
  if (!value) {
    return { ok: false, reason: "missing DEVICE_URL" };
  }

  let parsed;
  try {
    parsed = new URL(value);
  } catch {
    return { ok: false, reason: "invalid origin-only DEVICE_URL" };
  }

  const protocolAllowed = parsed.protocol === "http:" || parsed.protocol === "https:";
  const originOnlyPath = parsed.pathname === "" || parsed.pathname === "/";
  if (
    !protocolAllowed ||
    parsed.username ||
    parsed.password ||
    parsed.search ||
    parsed.hash ||
    !originOnlyPath
  ) {
    return { ok: false, reason: "invalid origin-only DEVICE_URL" };
  }

  return { ok: true, url: parsed };
}

function websocketUrlFromDeviceUrl(deviceUrl, path) {
  const parsed = new URL(deviceUrl.toString());
  parsed.protocol = parsed.protocol === "https:" ? "wss:" : "ws:";
  parsed.username = "";
  parsed.password = "";
  parsed.pathname = path;
  parsed.search = "";
  parsed.hash = "";
  return parsed;
}

function redactUrlForStatus(url) {
  const protocol = url.protocol === "wss:" ? "wss:" : "ws:";
  return `${protocol}//[redacted]${url.pathname}`;
}

function baseLines(path, args) {
  return [
    "phase17_websocket_capture",
    `path=${path}`,
    "network_scan=disabled - DEVICE_URL must be explicit",
    "websocket_route_claim=frame-capture-required",
    `duration_ms=${args.durationMs}`,
    `max_frames=${args.maxFrames}`,
  ];
}

function writeOutput(path, lines) {
  const content = `${lines.join("\n")}\n`;
  if (path) {
    writeFileSync(path, content, "utf8");
    return;
  }

  process.stdout.write(content);
}

function writeBlocked(args, reason, maybeCaptureUrl = "") {
  const lines = baseLines(args.path, args);
  if (maybeCaptureUrl) {
    lines.splice(2, 0, `websocket_capture_url=${maybeCaptureUrl}`);
  }
  lines.push(`websocket_target_status=blocked - ${reason}`);
  lines.push("websocket_open_status=blocked");
  lines.push("websocket_frame_status=not-run");
  writeOutput(args.out, lines);
}

function validateArgs(args) {
  if (!allowedWebSocketPaths.has(args.path)) {
    return { ok: false, reason: "unsupported WebSocket path" };
  }
  if (args.durationMs <= 0) {
    return { ok: false, reason: "duration-ms must be positive" };
  }
  if (args.durationMs > maxDurationMs) {
    return { ok: false, reason: "duration-ms exceeds 30000" };
  }
  if (args.maxFrames <= 0) {
    return { ok: false, reason: "max-frames must be positive" };
  }
  if (args.maxFrames > maxAllowedFrames) {
    return { ok: false, reason: "max-frames exceeds 10" };
  }

  return { ok: true };
}

function statusForTimeout(path, frames, opened) {
  if (frames > 0) {
    return `websocket_frame_status=passed frames=${frames}`;
  }
  if (path === "/api/ws") {
    return opened
      ? "websocket_frame_status=pending - open timeout without raw log frame"
      : "websocket_frame_status=pending - no open before timeout";
  }
  return "websocket_frame_status=pending - no live frame before timeout";
}

async function captureFake(args, wsUrl, mode) {
  const lines = baseLines(args.path, args);
  lines.splice(2, 0, `websocket_capture_url=${redactUrlForStatus(wsUrl)}`);
  lines.push("websocket_target_status=passed");

  if (mode === "open-frame") {
    const payload = process.env.PHASE17_FAKE_WEBSOCKET_PAYLOAD || "{}";
    lines.push("websocket_open_status=opened");
    lines.push(`websocket_frame_1=${redactText(payload)}`);
    lines.push("websocket_frame_status=passed frames=1");
    writeOutput(args.out, lines);
    return;
  }

  if (mode === "open-timeout") {
    lines.push("websocket_open_status=opened");
    lines.push(statusForTimeout(args.path, 0, true));
    writeOutput(args.out, lines);
    return;
  }

  if (mode === "error") {
    lines.push("websocket_open_status=not-run");
    lines.push("websocket_error=connection error");
    lines.push("websocket_frame_status=pending - connection error");
    writeOutput(args.out, lines);
    return;
  }

  throw new Error(`unsupported PHASE17_FAKE_WEBSOCKET_MODE: ${mode}`);
}

async function captureReal(args, wsUrl) {
  const lines = baseLines(args.path, args);
  lines.splice(2, 0, `websocket_capture_url=${redactUrlForStatus(wsUrl)}`);
  lines.push("websocket_target_status=passed");

  if (typeof globalThis.WebSocket !== "function") {
    lines.push("websocket_open_status=not-run");
    lines.push("websocket_frame_status=pending - Node global WebSocket unavailable");
    writeOutput(args.out, lines);
    return;
  }

  await new Promise((resolve) => {
    let settled = false;
    let opened = false;
    let frames = 0;
    const socket = new globalThis.WebSocket(wsUrl);
    const timer = setTimeout(() => {
      if (settled) {
        return;
      }
      settled = true;
      lines.push(opened ? "websocket_open_status=opened" : "websocket_open_status=not-run");
      lines.push(statusForTimeout(args.path, frames, opened));
      socket.close();
      resolve();
    }, args.durationMs);

    socket.addEventListener("open", () => {
      opened = true;
    });

    socket.addEventListener("message", (event) => {
      frames += 1;
      lines.push(`websocket_frame_${frames}=${redactText(event.data)}`);
      if (frames >= args.maxFrames && !settled) {
        settled = true;
        clearTimeout(timer);
        lines.push("websocket_open_status=opened");
        lines.push(`websocket_frame_status=passed frames=${frames}`);
        socket.close();
        resolve();
      }
    });

    socket.addEventListener("error", (event) => {
      if (settled) {
        return;
      }
      settled = true;
      clearTimeout(timer);
      lines.push(opened ? "websocket_open_status=opened" : "websocket_open_status=not-run");
      lines.push(`websocket_error=${redactText(event.message || "connection error")}`);
      lines.push("websocket_frame_status=pending - connection error");
      resolve();
    });
  });

  writeOutput(args.out, lines);
}

async function main() {
  const args = parseArgs(process.argv.slice(2));
  const argValidation = validateArgs(args);
  if (!argValidation.ok) {
    writeBlocked(args, argValidation.reason);
    return;
  }

  const maybeParsed = maybeParseOriginDeviceUrl(args.maybeDeviceUrl);
  if (!maybeParsed.ok) {
    writeBlocked(args, maybeParsed.reason);
    return;
  }

  const wsUrl = websocketUrlFromDeviceUrl(maybeParsed.url, args.path);
  const maybeFakeMode = process.env.PHASE17_FAKE_WEBSOCKET_MODE || "";
  if (maybeFakeMode) {
    await captureFake(args, wsUrl, maybeFakeMode);
    return;
  }

  await captureReal(args, wsUrl);
}

try {
  await main();
} catch (error) {
  const message = error instanceof Error ? error.message : String(error);
  process.stderr.write(`${redactText(message)}\n`);
  process.exit(2);
}
