#!/usr/bin/env node
// Stand-in soak observer: reads the handoff line, emits idle journal lines, and stops on `{"op":"stop"}`.
import { createInterface } from "node:readline";
const write = (event, extra = {}) => process.stdout.write(JSON.stringify({ schema: "soak-observer-v1", hostUnixMs: Date.now(), transport: null, event, sample: null, detail: null, ...extra }) + "\n");
let first = true;
for await (const line of createInterface({ input: process.stdin })) {
  if (first) {
    first = false;
    const handoff = JSON.parse(line);
    if (handoff.schema !== "soak-observer-input-v1") process.exit(3);
    write("connected", { transport: "websocket", detail: { reconnect: false } });
    for (let index = 0; index < 25; index++) write("sample", { transport: "http", sample: { miningActive: false } });
    for (let index = 0; index < 12; index++) write("sample", { transport: "websocket", sample: { miningActive: false } });
    continue;
  }
  if (JSON.parse(line).op === "stop") { write("stopped", { detail: { reason: "requested" } }); process.exit(0); }
}
