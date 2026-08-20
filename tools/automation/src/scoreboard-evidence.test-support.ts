import { createHash } from "node:crypto";
import { chmod, mkdir, mkdtemp, writeFile } from "node:fs/promises";
import http, { type Server } from "node:http";
import os from "node:os";
import path from "node:path";

import type { ScoreboardEvidenceOptions } from "./scoreboard-evidence.js";
import {
  scoreboardReferenceFragments,
  scoreboardSourceFragments,
} from "./scoreboard-source-inventory.js";

export const sourceCommit = "a".repeat(40);
export const referenceCommit = "c1915b0a63bfabebdb95a515cedfee05146c1d50";
export const appElfSha256 = "e".repeat(64);
const nodeProgram = process.env["JS_BINARY__NODE_BINARY"] ?? process.execPath;

export type ScoreboardFixture = Readonly<{
  root: string;
  planSha256: string;
  options: ScoreboardEvidenceOptions;
}>;

export type ScoreboardServer = Readonly<{
  origin: string;
  close: () => Promise<void>;
}>;

function sha256(value: string): string {
  return createHash("sha256").update(value).digest("hex");
}

async function writeProtected(candidate: string, document: string): Promise<void> {
  await writeFile(candidate, document, { mode: 0o600 });
  await chmod(candidate, 0o600);
}

export async function scoreboardFixture(name: string): Promise<ScoreboardFixture> {
  const root = await mkdtemp(path.join(os.tmpdir(), `bitaxe-scoreboard-${name}-`));
  await writeFile(path.join(root, "MODULE.bazel"), 'module(name = "fixture")\n');
  for (const [relative, fragments] of [...scoreboardSourceFragments, ...scoreboardReferenceFragments]) {
    const candidate = path.join(root, relative);
    await mkdir(path.dirname(candidate), { recursive: true });
    await writeFile(candidate, `${fragments.join("\n")}\n`);
  }
  const planRelative = "docs/parity/work-plans/20260820T150151Z-STAT-003/PLAN.md";
  const plan = "- Parity row: `STAT-003`\n- Active task: `task-parity-stat003-scoreboard`\n";
  await mkdir(path.dirname(path.join(root, planRelative)), { recursive: true });
  await writeFile(path.join(root, planRelative), plan);
  await writeFile(path.join(root, "TASKS.md"), [
    "### task-parity-stat003-scoreboard | fixture",
    `Plan: \`${planRelative}\`.`,
    "Attempt: `attempt-005`.",
  ].join("\n"));
  const inputs = path.join(root, "inputs");
  await mkdir(inputs);
  await writeFile(path.join(inputs, "package.json"), JSON.stringify({
    source_commit: sourceCommit,
    reference_commit: referenceCommit,
    app_elf_sha256: appElfSha256,
  }));
  await writeProtected(path.join(inputs, "wifi.json"), "{}\n");
  await writeProtected(path.join(inputs, "pool.json"), "{}\n");
  const wrapper = path.join(root, "scratch/stat003-scoreboard/wrapper-005");
  await mkdir(wrapper, { recursive: true, mode: 0o700 });
  await chmod(wrapper, 0o700);
  for (const output of ["detector.stdout", "detector.stderr", "capture.stdout", "capture.stderr"]) {
    await writeProtected(path.join(wrapper, output), "");
  }
  return {
    root,
    planSha256: sha256(plan),
    options: {
      privateRoot: "scratch/stat003-scoreboard/attempt-005",
      packageManifest: "inputs/package.json",
      wifiCredentials: "inputs/wifi.json",
      poolCredentials: "inputs/pool.json",
      detectorOutput: "scratch/stat003-scoreboard/wrapper-005/detector.stdout",
      port: "/dev/private-port",
      projection: "docs/parity/evidence/stat003-scoreboard/scoreboard-projection.json",
      durationSeconds: 600,
      captureTimeoutSeconds: 1_800,
    },
  };
}

export async function scoreboardChild(
  fixture: ScoreboardFixture,
  origin: string,
  options: Readonly<{
    finalTerminalConsumed?: boolean;
    omitTerminalCloseRequested?: boolean;
    terminalCloseRequested?: boolean | string;
  }> = {},
): Promise<string> {
  const child = path.join(fixture.root, "child.mjs");
  const terminalCloseRequestedField = options.omitTerminalCloseRequested === true
    ? ""
    : `terminal_close_requested: ${JSON.stringify(options.terminalCloseRequested ?? true)},`;
  await writeFile(child, `#!${nodeProgram}
import { createHash } from "node:crypto";
import { chmod, mkdir, writeFile } from "node:fs/promises";
import path from "node:path";
const args = process.argv.slice(2);
const digest = (value) => createHash("sha256").update(value).digest("hex");
if (args[0] === "mining-campaign") {
  const root = args[args.indexOf("--evidence-dir") + 1];
  await mkdir(root, { recursive: true, mode: 0o700 });
  await chmod(root, 0o700);
  const diagnostics = JSON.stringify({ schema: "mining-campaign-serial-diagnostics-v4", runtime_attestation_mixed_reset_reason: "none", panic_signature: "none", panic_signature_count: 0 }) + "\\n";
  const network = JSON.stringify({ schema: "mining-campaign-network-continuity-v12", status: "accepted", correlation_failure: "none", required_window_count: 20, covered_window_count: 20, work_renewal_valid: true, terminal_http_valid: true, terminal_websocket_valid: true, terminal_pool_persisted: true, terminal_settlement: "accepted_after_serial_close", ${terminalCloseRequestedField} terminal_consumed_observed: true, final_terminal_consumed: ${options.finalTerminalConsumed ?? true}, serial_finished_observed: true }) + "\\n";
  const flash = JSON.stringify({ schema: "mining-campaign-flash-diagnostics-v1", factory: {}, nvs: {}, raw_output_included: false }) + "\\n";
  const result = JSON.stringify({ schema: "mining-campaign-result-v16", status: "accepted", stage: "live-share", profile: "conservative", duration_seconds: 600, runtime_identity: "trusted", pool_config: "local_owner_supplied", safe_stop: "confirmed", usb_cleanup: "ready", redacted: true, qualified_candidate_count: 1, below_pool_target_count: 0, duplicate_candidate_count: 0, submit_outcome: "accepted", diagnostics_sha256: digest(diagnostics), network_continuity_sha256: digest(network), flash_diagnostics_sha256: digest(flash) }) + "\\n";
  const files = new Map([["campaign-diagnostics.private.json", diagnostics], ["campaign-network.private.json", network], ["campaign-flash.private.json", flash], ["campaign-mining-diagnostics.private.json", "{}\\n"], ["campaign-observations.private.json", "{}\\n"], ["campaign-result.json", result], ["campaign-result.sha256", digest(result) + "\\n"]]);
  for (const [name, document] of files) { const candidate = path.join(root, name); await writeFile(candidate, document, { mode: 0o600 }); await chmod(candidate, 0o600); }
} else if (args[0] === "monitor") {
  process.stdout.write("runtime_origin session=00112233445566778899aabbccddeeff boot_ordinal=7 device_url=${origin} redacted=true\\n");
} else if (args[0] === "-C" && args[2] === "rev-parse") {
  process.stdout.write("${referenceCommit}\\n");
} else if (args[0] === "-C" && args[2] === "status") {
  process.stdout.write("");
} else if (args[0] === "status") {
  process.stdout.write("");
} else if (args[0] === "rev-parse") {
  process.stdout.write("${sourceCommit}\\n");
}
`);
  await chmod(child, 0o700);
  return child;
}

export async function startScoreboardServer(
  options: Readonly<{
    changeAfterRestart?: boolean;
    postRestartMiningActivity?: string;
  }> = {},
): Promise<ScoreboardServer> {
  let restarted = false;
  const entries = [
    { difficulty: 42.5, job_id: "job-a", extranonce2: "0001", ntime: 1, nonce: "1234ABCD", version_bits: "20000000" },
    { difficulty: 10, job_id: "job-b", extranonce2: "0002", ntime: 2, nonce: "00000001", version_bits: "00000000" },
  ];
  const server: Server = http.createServer((request, response) => {
    if (request.method === "GET" && request.url === "/api/system/info") {
      response.setHeader("content-type", "application/json");
      response.end(JSON.stringify({
        sourceCommit,
        referenceCommit,
        appElfSha256,
        bootSession: restarted ? "ffeeddccbbaa99887766554433221100" : "00112233445566778899aabbccddeeff",
        bootOrdinal: restarted ? 8 : 7,
        resetReasonCategory: restarted ? "software_cpu" : "power_on",
        miningActivity: restarted
          ? options.postRestartMiningActivity ?? "safe_blocked"
          : "safe_blocked",
        startMiningOnBoot: false,
      }));
      return;
    }
    if (request.method === "GET" && request.url === "/api/system/scoreboard") {
      response.setHeader("content-type", "application/json");
      response.end(JSON.stringify(restarted && options.changeAfterRestart === true ? entries.slice(0, 1) : entries));
      return;
    }
    if (request.method === "GET" && request.url === "/scoreboard") {
      response.setHeader("content-type", "text/html; charset=utf-8");
      response.end('<section data-page="scoreboard"></section><script src="/assets/api-client.js"></script>');
      return;
    }
    if (request.method === "POST" && request.url === "/api/system/restart") {
      restarted = true;
      response.setHeader("content-type", "application/json");
      response.end("{}");
      return;
    }
    response.statusCode = 404;
    response.end();
  });
  await new Promise<void>((resolve) => server.listen(0, "127.0.0.1", resolve));
  const address = server.address();
  if (address === null || typeof address === "string") throw new Error("test server address invalid");
  return {
    origin: `http://127.0.0.1:${String(address.port)}`,
    close: () => new Promise<void>((resolve, reject) => server.close((error) => {
      if (error === undefined) resolve();
      else reject(error);
    })),
  };
}
