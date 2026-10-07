// Stand-in `just` for hardware-free tests. It records each call, then acts out the recipe from
// `$FAKE_JUST_DIR/scenario.json`. Like real `just`, it removes the single quotes from recipe arguments.
import { appendFileSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";
import { processSnapshot } from "../host-stalls/capture.mjs";

const directory = process.env.FAKE_JUST_DIR;
const unquote = (value) => /^'.*'$/su.test(value) ? value.slice(1, -1).replaceAll("'\\''", "'") : value;
const [recipe, ...args] = process.argv.slice(2).map((value, index) => index === 0 ? value : unquote(value));
const scenario = JSON.parse(readFileSync(resolve(directory, "scenario.json"), "utf8"));
const option = (name) => args[args.indexOf(name) + 1];
const callsPath = resolve(directory, "calls.jsonl");
const previous = (() => { try { return readFileSync(callsPath, "utf8").trim().split("\n").filter(Boolean).map((line) => JSON.parse(line)); } catch { return []; } })();
appendFileSync(callsPath, `${JSON.stringify([recipe, ...args])}\n`);
appendFileSync(resolve(directory, "raw-calls.jsonl"), `${JSON.stringify(process.argv.slice(2))}\n`);
const write = (path, value) => writeFileSync(path, `${JSON.stringify(value)}\n`, { mode: 0o600, flag: "wx" });

if (recipe === "detect-ultra205") {
  const detections = previous.filter(([name]) => name === "detect-ultra205").length;
  const physicals = scenario.physicals ?? ["a".repeat(64)];
  process.stdout.write(`port: /dev/cu.usbmodemFAKE1\nusb_profile: serial_jtag_runtime\nphysical_identity_sha256: ${physicals[Math.min(detections, physicals.length - 1)]}\n`);
  process.exit(scenario.detectExit ?? 0);
}
if (recipe === "monitor" && scenario.monitorExit) process.exit(scenario.monitorExit);
if (recipe === "monitor") {
  for (const uptime of [60000, 120000]) process.stdout.write(`internal_heap_sample schema=v1 uptime_ms=${uptime} free_bytes=${scenario.freeBytes ?? 40000} ` +
    `allocated_bytes=400000 largest_block_bytes=20000 minimum_free_bytes=1295 allocated_blocks=1000 free_blocks=40 revoked=false redacted=true\n`);
  process.exit(0);
}
if (recipe === "core-dump-read") process.exit(scenario.readExit ?? 0);
if (args[0] === "finish") {
  const root = option("--private-root"), stage = args.includes("--stage") ? option("--stage") : null;
  if (stage === "recovery") write(resolve(root, "recovery", "result.json"), { current_recovery_complete: scenario.recoveryComplete ?? true });
  if (stage === null && scenario.proof) {
    write(resolve(root, "current-recovery.json"), { firmware_commit: "b".repeat(40), app_elf_sha256: "c".repeat(64),
      physical_identity_sha256: scenario.proofPhysical ?? "a".repeat(64), observed_at_unix_ms: Date.now() - (scenario.proofAgeMs ?? 1000) });
  }
  process.stdout.write('{"finished":true}\n');
  process.exit(scenario.finishExit ?? 0);
}
if (args[0] === "serve") {
  const stageRoot = resolve(option("--private-root"), option("--stage"));
  mkdirSync(stageRoot, { mode: 0o700 });
  const owner = (await processSnapshot()).find((row) => row.pid === process.pid);
  if (scenario.serveFails) process.exit(1);
  if (scenario.serveHangs) writeFileSync(resolve(directory, "serve.pid"), String(process.pid));
  else write(resolve(stageRoot, "server-owner.json"), { owner, port: 48765 });
  process.once("SIGTERM", () => process.exit(0));
  setInterval(() => {}, 1000);
} else if (!["detect-ultra205", "monitor", "core-dump-read"].includes(recipe)) {
  process.exit(97);
}
