import { spawn } from "node:child_process";
import { chmod, lstat, readFile, writeFile } from "node:fs/promises";
import path from "node:path";

import { allowedEnvironment } from "./process.js";
import { sha256 } from "./stratum-v2-restore-model.js";

export const validatorChildReceiptSchema =
  "bitaxe-stratum-v2-validator-child-receipt-v1" as const;

const maximumOutputBytes = 65_536;
const acceptedOutput = "restore_readiness=accepted\n";
const digestPattern = /^[0-9a-f]{64}$/u;
const commitPattern = /^[0-9a-f]{40}$/u;

export type ValidatorChildReceipt = {
  readonly schema_version: typeof validatorChildReceiptSchema;
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
  readonly validation_accepted: boolean;
  readonly source_commit: string;
  readonly plan_sha256: string;
};

type ValidatorChildInput = {
  readonly workspace: string;
  readonly program: string;
  readonly args: readonly string[];
  readonly receiptPath: string;
  readonly sourceCommit: string;
  readonly planSha256: string;
  readonly timeoutMillis?: number;
};

type CapturedOutput = {
  readonly buffers: Buffer[];
  bytes: number;
};

function capture(output: CapturedOutput, chunk: Buffer): boolean {
  output.bytes = Math.min(maximumOutputBytes + 1, output.bytes + chunk.length);
  const retained = output.buffers.reduce((total, value) => total + value.length, 0);
  const remaining = maximumOutputBytes - retained;
  if (remaining > 0) output.buffers.push(chunk.subarray(0, remaining));
  return output.bytes > maximumOutputBytes;
}

function outputBytes(output: CapturedOutput): Buffer {
  return Buffer.concat(output.buffers);
}

function receiptIsAccepted(receipt: ValidatorChildReceipt): boolean {
  return receipt.exit_code === 0
    && !receipt.timed_out
    && !receipt.output_limit_exceeded
    && !receipt.launch_failed
    && receipt.stdout_bytes === Buffer.byteLength(acceptedOutput)
    && receipt.stderr_bytes === 0
    && receipt.stdout_sha256 === sha256(acceptedOutput)
    && receipt.stderr_sha256 === sha256("");
}

export async function runValidatorChild(input: ValidatorChildInput): Promise<ValidatorChildReceipt> {
  if (!path.isAbsolute(input.workspace)
    || !path.isAbsolute(input.program)
    || input.args.some(value => !path.isAbsolute(value) && value.includes(path.sep))) {
    throw new Error("validator child invocation is not absolute");
  }
  const stdout: CapturedOutput = { buffers: [], bytes: 0 };
  const stderr: CapturedOutput = { buffers: [], bytes: 0 };
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
  }, input.timeoutMillis ?? 30_000);
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
  const stdoutValue = outputBytes(stdout);
  const stderrValue = outputBytes(stderr);
  const base = {
    schema_version: validatorChildReceiptSchema,
    launcher: "bounded_spawn",
    working_directory: "workspace_bound",
    environment_policy: "allowlisted",
    exit_code: exitCode,
    timed_out: timedOut,
    output_limit_exceeded: outputLimitExceeded,
    launch_failed: launchFailed,
    stdout_bytes: stdout.bytes,
    stderr_bytes: stderr.bytes,
    stdout_sha256: sha256(stdoutValue),
    stderr_sha256: sha256(stderrValue),
    invocation_sha256: sha256(JSON.stringify(input.args)),
    validation_accepted: false,
    source_commit: input.sourceCommit,
    plan_sha256: input.planSha256,
  } as const;
  const receipt: ValidatorChildReceipt = {
    ...base,
    validation_accepted: receiptIsAccepted(base),
  };
  await writeFile(input.receiptPath, `${JSON.stringify(receipt, null, 2)}\n`, {
    mode: 0o600,
    flag: "wx",
  });
  await chmod(input.receiptPath, 0o600);
  return receipt;
}

export async function validateValidatorChildReceipt(
  candidate: string,
  expectedSourceCommit: string,
  expectedPlanSha256: string,
): Promise<ValidatorChildReceipt> {
  const metadata = await lstat(candidate);
  if (metadata.isSymbolicLink() || !metadata.isFile() || (metadata.mode & 0o777) !== 0o600) {
    throw new Error("validator receipt protection is invalid");
  }
  const value = JSON.parse(await readFile(candidate, "utf8")) as ValidatorChildReceipt;
  if (value.schema_version !== validatorChildReceiptSchema
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
    || !commitPattern.test(value.source_commit)
    || !digestPattern.test(value.plan_sha256)
    || value.source_commit !== expectedSourceCommit
    || value.plan_sha256 !== expectedPlanSha256
    || value.validation_accepted !== receiptIsAccepted(value)) {
    throw new Error("validator receipt is invalid");
  }
  return value;
}
