#!/usr/bin/env node
// Stand-in for `flash usb-presence-watch`: argv is [controlFile, scriptJson, ...watcherArgs]. Step `i` of the
// script is emitted once the control file holds a number >= its `gate`; stdin EOF emits `stopped` and exits 0.
import { readFileSync } from "node:fs";
const [controlPath, scriptJson, ...args] = process.argv.slice(2);
if (!args.includes("usb-presence-watch") || !args.some((arg) => /^--physical-identity=[0-9a-f]{64}$/u.test(arg))) process.exit(2);
let sequence = 0;
const write = (event) => process.stdout.write(`${JSON.stringify({ schema: "bwg-usb-presence-watch-v1", sequence: ++sequence, ...event })}\n`);
write({ elapsed_ms: 0, event: "started" });
const steps = JSON.parse(scriptJson);
let next = 0;
const gate = () => { try { return Number(readFileSync(controlPath, "utf8").trim() || "0"); } catch { return 0; } };
const timer = setInterval(() => {
  while (next < steps.length && gate() >= (steps[next].gate ?? 0)) {
    const { gate: omitted, raw, ...event } = steps[next++];
    if (raw) process.stdout.write(`${raw}\n`); else write(event);
  }
}, 10);
process.stdin.on("data", () => undefined);
process.stdin.on("end", () => { clearInterval(timer); write({ elapsed_ms: 99999, event: "stopped" }); process.exit(0); });
