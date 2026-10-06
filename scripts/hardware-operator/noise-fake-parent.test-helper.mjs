// Stand-in operator parent for real-process tests: same argv, stdin FIFO and one-reply-per-line protocol.
import { readFileSync } from "node:fs";
import { createInterface } from "node:readline";

const root = process.argv[2].slice("--private-root=".length);
const mode = readFileSync(`${root}/fake-parent-mode`, "utf8").trim();
const output = (value) => process.stdout.write(`${JSON.stringify(value)}\n`);

async function serveCommands() {
  output({ event: "supervisor_ready", context_sha256: "0".repeat(64) });
  for await (const line of createInterface({ input: process.stdin, terminal: false })) {
    const command = JSON.parse(line);
    if (command.action === "exit") { output({ event: "parent_exiting" }); process.stdin.destroy(); break; }
    if (mode === "slow") await new Promise((done) => setTimeout(done, 1500));
    output({ event: "echo", action: command.action, ...(command.index === undefined ? {} : { index: command.index }) });
  }
}

if (mode === "fail" || mode === "fail-linger") {
  output({ event: "operator_error", code: "supervisor_start_failed" });
  // `fail-linger` keeps running briefly after its error line, as a parent still reaping its supervisor does.
  setTimeout(() => process.exit(1), mode === "fail-linger" ? 1500 : 0);
} else {
  await serveCommands();
}
