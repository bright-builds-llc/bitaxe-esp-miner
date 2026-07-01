#!/usr/bin/env node
import { writeFileSync } from "node:fs";

function parseArgs(argv) {
  const args = {
    maybeDeviceUrl: process.env.DEVICE_URL || "",
    out: "",
    durationMs: 5_000,
    maxFrames: 5,
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
          "usage: phase15-websocket-capture.mjs --device-url URL --out PATH [--duration-ms N] [--max-frames N]",
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
    .replace(/https?:\/\/[^\s"<>]+/gi, "[redacted-url]")
    .replace(/wss?:\/\/[^\s"<>]+/gi, "[redacted-url]")
    .replace(/\b(?:\d{1,3}\.){3}\d{1,3}\b/g, "[redacted-ip]")
    .replace(/\b(?:[a-f0-9]{2}:){5}[a-f0-9]{2}\b/gi, "[redacted-mac]")
    .replace(
      /"(ssid|wifiPass|wifiPassword|stratumUser|stratumPassword|stratumCert|poolUrl|fallbackPoolUrl|hostname|ip|ipAddress|gateway|netmask|dns|apiToken|apiKey|token|password|user|worker)"\s*:\s*"[^"]*"/gi,
      '"$1":"[redacted]"',
    )
    .replace(
      /"(stratumPort|fallbackStratumPort)"\s*:\s*[0-9]+/gi,
      '"$1":[redacted]',
    );
}

function redactUrlForStatus(url) {
  const protocol = url.protocol === "https:" || url.protocol === "wss:" ? url.protocol : "http:";
  return `${protocol}//[redacted]${url.pathname}`;
}

function websocketUrlFromDeviceUrl(deviceUrl) {
  const parsed = new URL(deviceUrl);
  if (parsed.protocol !== "http:" && parsed.protocol !== "https:") {
    throw new Error("DEVICE_URL must use http or https");
  }

  parsed.protocol = parsed.protocol === "https:" ? "wss:" : "ws:";
  parsed.pathname = "/api/ws/live";
  parsed.search = "";
  parsed.hash = "";
  return parsed;
}

function writeOutput(path, lines) {
  const content = `${lines.join("\n")}\n`;
  if (path) {
    writeFileSync(path, content, "utf8");
    return;
  }

  process.stdout.write(content);
}

async function captureWebSocket(args) {
  if (!args.maybeDeviceUrl) {
    throw new Error("DEVICE_URL is required");
  }

  const wsUrl = websocketUrlFromDeviceUrl(args.maybeDeviceUrl);
  const lines = [
    `websocket_capture_url=${redactUrlForStatus(wsUrl)}`,
    "network_scan=disabled - DEVICE_URL must be explicit",
  ];

  if (args.maxFrames === 0) {
    lines.push("websocket_frame_status=pending - max frames zero");
    writeOutput(args.out, lines);
    return;
  }

  if (typeof globalThis.WebSocket !== "function") {
    lines.push("websocket_frame_status=pending - Node global WebSocket unavailable");
    writeOutput(args.out, lines);
    return;
  }

  await new Promise((resolve) => {
    let settled = false;
    let frames = 0;
    const socket = new globalThis.WebSocket(wsUrl);
    const timer = setTimeout(() => {
      if (settled) {
        return;
      }
      settled = true;
      lines.push(
        frames > 0
          ? `websocket_frame_status=passed frames=${frames}`
          : "websocket_frame_status=pending - no frame before timeout",
      );
      socket.close();
      resolve();
    }, args.durationMs);

    socket.addEventListener("message", (event) => {
      frames += 1;
      lines.push(`websocket_frame_${frames}=${redactText(event.data)}`);
      if (frames >= args.maxFrames && !settled) {
        settled = true;
        clearTimeout(timer);
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
      lines.push(`websocket_error=${redactText(event.message || "connection error")}`);
      lines.push("websocket_frame_status=pending - connection error");
      resolve();
    });
  });

  writeOutput(args.out, lines);
}

try {
  const args = parseArgs(process.argv.slice(2));
  await captureWebSocket(args);
} catch (error) {
  const message = error instanceof Error ? error.message : String(error);
  process.stderr.write(`${redactText(message)}\n`);
  process.exit(2);
}
