import { spawn } from "node:child_process";
import { chmod, lstat, readFile, writeFile } from "node:fs/promises";
import path from "node:path";

import { allowedEnvironment } from "./process.js";
import { sha256 } from "./stratum-v2-restore-model.js";

export const runtimeMonitorReceiptSchema =
  "bitaxe-stratum-v2-runtime-monitor-receipt-v1" as const;

const maximumOutputBytes = 1_048_576;
const digestPattern = /^[0-9a-f]{64}$/u;
const commitPattern = /^[0-9a-f]{40}$/u;
const originPattern = /\bhttps?:\/\/[A-Za-z0-9.-]+(?::[0-9]+)?\b/gu;

export type RuntimeMonitorTerminalCategory =
  | "ready"
  | "concurrent_repo_session"
  | "foreign_holder"
  | "transport_absent"
  | "identity_drift"
  | "cleanup_failed"
  | "timeout"
  | "output_limit"
  | "launch_failed"
  | "monitor_failed";

export type RuntimeMonitorReceipt = {
  readonly schema_version: typeof runtimeMonitorReceiptSchema;
  readonly launcher: "bounded_spawn";
  readonly working_directory: "workspace_bound";
  readonly environment_policy: "allowlisted";
  readonly exit_code: number | null;
  readonly timed_out: boolean;
  readonly output_limit_exceeded: boolean;
  readonly launch_failed: boolean;
  readonly stdout_bytes: number;
  readonly stderr_bytes: number;
  readonly stdout_sha256: string;
  readonly stderr_sha256: string;
  readonly invocation_sha256: string;
  readonly origin_count: 0 | 1 | 2;
  readonly usb_cleanup_ready: boolean;
  readonly monitor_command_emitted: boolean;
  readonly terminal_category: RuntimeMonitorTerminalCategory;
  readonly source_commit: string;
  readonly plan_sha256: string;
};

type RuntimeMonitorInput = {
  readonly workspace: string;
  readonly program: string;
  readonly args: readonly string[];
  readonly receiptPath: string;
  readonly sourceCommit: string;
  readonly planSha256: string;
  readonly timeoutMillis: number;
};

type CapturedOutput = {
  readonly buffers: Buffer[];
  bytes: number;
  retainedBytes: number;
};

function capture(output: CapturedOutput, chunk: Buffer): boolean {
  output.bytes = Math.min(maximumOutputBytes + 1, output.bytes + chunk.length);
  const remaining = maximumOutputBytes - output.retainedBytes;
  if (remaining > 0) {
    const retained = chunk.subarray(0, remaining);
    output.buffers.push(retained);
    output.retainedBytes += retained.length;
  }
  return output.bytes > maximumOutputBytes;
}

function countOrigins(stdout: string): 0 | 1 | 2 {
  const unique = new Set(stdout.match(originPattern) ?? []);
  return unique.size === 0 ? 0 : unique.size === 1 ? 1 : 2;
}

function terminalCategory(input: {
  readonly exitCode: number | null;
  readonly timedOut: boolean;
  readonly outputLimitExceeded: boolean;
  readonly launchFailed: boolean;
  readonly stdout: string;
  readonly stderr: string;
}): RuntimeMonitorTerminalCategory {
  if (input.launchFailed) return "launch_failed";
  if (input.timedOut) return "timeout";
  if (input.outputLimitExceeded) return "output_limit";
  const output = `${input.stdout}\n${input.stderr}`;
  if (output.includes("concurrent_repo_session")) return "concurrent_repo_session";
  if (output.includes("foreign_holder")) return "foreign_holder";
  if (output.includes("transport_absent")) return "transport_absent";
  if (output.includes("identity_drift")) return "identity_drift";
  if (output.includes("cleanup_failed")) return "cleanup_failed";
  if (input.exitCode === 0
    && input.stdout.includes("usb_session: ready")
    && countOrigins(input.stdout) === 1) {
    return "ready";
  }
  return "monitor_failed";
}

export async function runRuntimeMonitorChild(input: RuntimeMonitorInput): Promise<{
  readonly receipt: RuntimeMonitorReceipt;
  readonly stdout: string;
}> {
  if (!path.isAbsolute(input.workspace)
    || !path.isAbsolute(input.program)
    || input.args.some(value => !path.isAbsolute(value) && value.includes(path.sep))) {
    throw new Error("runtime monitor invocation is not absolute");
  }
  const stdout: CapturedOutput = { buffers: [], bytes: 0, retainedBytes: 0 };
  const stderr: CapturedOutput = { buffers: [], bytes: 0, retainedBytes: 0 };
  let timedOut = false;
  let outputLimitExceeded = false;
  let launchFailed = false;
  const child = spawn(input.program, [...input.args], {
    cwd: input.workspace,
    env: allowedEnvironment(process.env),
    stdio: ["ignore", "pipe", "pipe"],
  });
  child.stdout.on("data", (chunk: Buffer) => {
    if (capture(stdout, chunk)) {
      outputLimitExceeded = true;
      child.kill("SIGTERM");
    }
  });
  child.stderr.on("data", (chunk: Buffer) => {
    if (capture(stderr, chunk)) {
      outputLimitExceeded = true;
      child.kill("SIGTERM");
    }
  });
  let killTimer: NodeJS.Timeout | undefined;
  const timer = setTimeout(() => {
    timedOut = true;
    child.kill("SIGTERM");
    killTimer = setTimeout(() => child.kill("SIGKILL"), 5_000);
  }, input.timeoutMillis);
  const exitCode = await new Promise<number | null>((resolve) => {
    let settled = false;
    const settle = (value: number | null) => {
      if (settled) return;
      settled = true;
      clearTimeout(timer);
      if (killTimer !== undefined) clearTimeout(killTimer);
      resolve(value);
    };
    child.once("error", () => {
      launchFailed = true;
      settle(null);
    });
    child.once("close", value => settle(value));
  });
  const stdoutBytes = Buffer.concat(stdout.buffers);
  const stderrBytes = Buffer.concat(stderr.buffers);
  const stdoutValue = stdoutBytes.toString("utf8");
  const stderrValue = stderrBytes.toString("utf8");
  const receipt: RuntimeMonitorReceipt = {
    schema_version: runtimeMonitorReceiptSchema,
    launcher: "bounded_spawn",
    working_directory: "workspace_bound",
    environment_policy: "allowlisted",
    exit_code: exitCode,
    timed_out: timedOut,
    output_limit_exceeded: outputLimitExceeded,
    launch_failed: launchFailed,
    stdout_bytes: stdout.bytes,
    stderr_bytes: stderr.bytes,
    stdout_sha256: sha256(stdoutBytes),
    stderr_sha256: sha256(stderrBytes),
    invocation_sha256: sha256(JSON.stringify(input.args)),
    origin_count: countOrigins(stdoutValue),
    usb_cleanup_ready: stdoutValue.includes("usb_session: ready"),
    monitor_command_emitted: stdoutValue.includes("monitor_command:"),
    terminal_category: terminalCategory({
      exitCode,
      timedOut,
      outputLimitExceeded,
      launchFailed,
      stdout: stdoutValue,
      stderr: stderrValue,
    }),
    source_commit: input.sourceCommit,
    plan_sha256: input.planSha256,
  };
  await writeFile(input.receiptPath, `${JSON.stringify(receipt, null, 2)}\n`, {
    mode: 0o600,
    flag: "wx",
  });
  await chmod(input.receiptPath, 0o600);
  return { receipt, stdout: stdoutValue };
}

export async function validateRuntimeMonitorReceipt(
  candidate: string,
  expectedSourceCommit: string,
  expectedPlanSha256: string,
): Promise<RuntimeMonitorReceipt> {
  const metadata = await lstat(candidate);
  if (metadata.isSymbolicLink() || !metadata.isFile() || (metadata.mode & 0o777) !== 0o600) {
    throw new Error("runtime monitor receipt protection is invalid");
  }
  const value = JSON.parse(await readFile(candidate, "utf8")) as RuntimeMonitorReceipt;
  const categories: readonly RuntimeMonitorTerminalCategory[] = [
    "ready", "concurrent_repo_session", "foreign_holder", "transport_absent",
    "identity_drift", "cleanup_failed", "timeout", "output_limit", "launch_failed",
    "monitor_failed",
  ];
  if (value.schema_version !== runtimeMonitorReceiptSchema
    || value.launcher !== "bounded_spawn"
    || value.working_directory !== "workspace_bound"
    || value.environment_policy !== "allowlisted"
    || (value.exit_code !== null
      && (!Number.isSafeInteger(value.exit_code) || value.exit_code < 0 || value.exit_code > 255))
    || typeof value.timed_out !== "boolean"
    || typeof value.output_limit_exceeded !== "boolean"
    || typeof value.launch_failed !== "boolean"
    || !Number.isSafeInteger(value.stdout_bytes)
    || value.stdout_bytes < 0
    || value.stdout_bytes > maximumOutputBytes + 1
    || !Number.isSafeInteger(value.stderr_bytes)
    || value.stderr_bytes < 0
    || value.stderr_bytes > maximumOutputBytes + 1
    || !digestPattern.test(value.stdout_sha256)
    || !digestPattern.test(value.stderr_sha256)
    || !digestPattern.test(value.invocation_sha256)
    || ![0, 1, 2].includes(value.origin_count)
    || typeof value.usb_cleanup_ready !== "boolean"
    || typeof value.monitor_command_emitted !== "boolean"
    || !categories.includes(value.terminal_category)
    || !commitPattern.test(value.source_commit)
    || !digestPattern.test(value.plan_sha256)
    || value.source_commit !== expectedSourceCommit
    || value.plan_sha256 !== expectedPlanSha256
    || (value.launch_failed !== (value.terminal_category === "launch_failed"))
    || (!value.launch_failed
      && value.timed_out !== (value.terminal_category === "timeout"))
    || (!value.launch_failed
      && !value.timed_out
      && value.output_limit_exceeded !== (value.terminal_category === "output_limit"))
    || (value.terminal_category === "ready"
      && (value.exit_code !== 0
        || value.timed_out
        || value.output_limit_exceeded
        || value.launch_failed
        || value.origin_count !== 1
        || !value.usb_cleanup_ready))) {
    throw new Error("runtime monitor receipt is invalid");
  }
  return value;
}
