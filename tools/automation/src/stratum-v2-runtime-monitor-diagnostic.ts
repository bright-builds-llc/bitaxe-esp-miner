import { chmod, mkdir, readFile, stat } from "node:fs/promises";
import path from "node:path";

import { runCampaignProcess } from "./stratum-v2-campaign.js";
import { sha256 } from "./stratum-v2-restore-model.js";
import { monitorRuntimeOrigin } from "./stratum-v2-runtime-admission.js";

export type RuntimeMonitorDiagnosticArgs = {
  readonly board: "205";
  readonly port: string;
  readonly privateRoot: string;
  readonly redactEvidence: true;
};

const taskId = "task-parity-str005-autonomous-continuation";
const planRelative =
  "docs/parity/work-plans/20260825T123346Z-STR-005-AUTONOMOUS-CONTINUATION/PLAN.md";
const expectedPrivateRoot = "scratch/str005-runtime-monitor-diagnostic/diagnostic-002";

export class RuntimeMonitorDiagnosticError extends Error {
  public constructor(
    public readonly category: "invalid_invocation" | "evidence_invalid" | "hardware_blocked",
    public readonly checkpoint: string,
  ) {
    super("runtime monitor diagnostic failed");
    this.name = "RuntimeMonitorDiagnosticError";
  }
}

function fail(
  category: RuntimeMonitorDiagnosticError["category"],
  checkpoint: string,
): never {
  throw new RuntimeMonitorDiagnosticError(category, checkpoint);
}

export function parseRuntimeMonitorDiagnosticArgs(
  values: readonly string[],
): RuntimeMonitorDiagnosticArgs {
  const options = new Map<string, string | true>();
  for (let index = 0; index < values.length; index += 1) {
    const key = values[index];
    if (key === "--redact-evidence") {
      if (options.has(key)) fail("invalid_invocation", "invocation");
      options.set(key, true);
      continue;
    }
    const value = values[index + 1];
    if (key === undefined || value === undefined || !key.startsWith("--")
      || value.startsWith("--") || options.has(key)) {
      fail("invalid_invocation", "invocation");
    }
    options.set(key, value);
    index += 1;
  }
  const allowed = new Set(["--board", "--port", "--private-root", "--redact-evidence"]);
  if ([...options.keys()].some(key => !allowed.has(key))) {
    fail("invalid_invocation", "invocation");
  }
  const required = (key: string): string => {
    const maybeValue = options.get(key);
    if (typeof maybeValue !== "string" || maybeValue.length === 0) {
      fail("invalid_invocation", "invocation");
    }
    return maybeValue;
  };
  const board = required("--board");
  if (board !== "205" || options.get("--redact-evidence") !== true) {
    fail("invalid_invocation", "invocation");
  }
  return {
    board,
    port: required("--port"),
    privateRoot: required("--private-root"),
    redactEvidence: true,
  };
}

async function gitText(workspace: string, args: readonly string[]): Promise<string> {
  const outcome = await runCampaignProcess(workspace, "git", args, 10_000);
  if (outcome.exitCode !== 0) fail("evidence_invalid", "source_identity");
  return outcome.stdout.trim();
}

async function requireAbsent(candidate: string): Promise<void> {
  try {
    await stat(candidate);
    fail("evidence_invalid", "outputs_absent");
  } catch (error) {
    if (error instanceof RuntimeMonitorDiagnosticError) throw error;
    if ((error as NodeJS.ErrnoException).code !== "ENOENT") {
      fail("evidence_invalid", "outputs_absent");
    }
  }
}

export async function runRuntimeMonitorDiagnostic(
  workspace: string,
  args: RuntimeMonitorDiagnosticArgs,
): Promise<void> {
  if (args.board !== "205"
    || args.privateRoot !== expectedPrivateRoot
    || !args.redactEvidence
    || args.port.length === 0) {
    fail("invalid_invocation", "invocation");
  }
  const sourceCommit = await gitText(workspace, ["rev-parse", "HEAD"]);
  const status = await gitText(workspace, ["status", "--porcelain"]);
  const sync = await gitText(workspace, ["rev-list", "--left-right", "--count", "HEAD...@{u}"]);
  if (!/^[0-9a-f]{40}$/u.test(sourceCommit) || status !== "" || sync !== "0\t0") {
    fail("evidence_invalid", "source_identity");
  }
  const plan = await readFile(path.join(workspace, planRelative), "utf8");
  const tasks = await readFile(path.join(workspace, "TASKS.md"), "utf8");
  if (!plan.includes(`- Active task: \`${taskId}\``)
    || !tasks.includes(`### ${taskId}`)
    || !tasks.includes(planRelative)
    || !tasks.includes(expectedPrivateRoot)) {
    fail("evidence_invalid", "plan_binding");
  }
  const ignored = await runCampaignProcess(
    workspace,
    "git",
    ["check-ignore", "-q", args.privateRoot],
    5_000,
  );
  if (ignored.exitCode !== 0) fail("evidence_invalid", "private_root_ignored");
  const privateRoot = path.resolve(workspace, args.privateRoot);
  await requireAbsent(privateRoot);
  await mkdir(path.dirname(privateRoot), { recursive: true, mode: 0o700 });
  await chmod(path.dirname(privateRoot), 0o700);
  await mkdir(privateRoot, { mode: 0o700 });
  await chmod(privateRoot, 0o700);
  const diagnosticFail = (
    category: string,
    _message: string,
    checkpoint: string,
  ): never => {
    fail(category === "evidence_invalid" ? "evidence_invalid" : "hardware_blocked", checkpoint);
  };
  await monitorRuntimeOrigin(
    workspace,
    path.join(workspace, "bazel-bin/tools/flash/flash"),
    args.port,
    runCampaignProcess,
    diagnosticFail,
    {
      receiptPath: path.join(privateRoot, "runtime-monitor-receipt.private.json"),
      sourceCommit,
      planSha256: sha256(plan),
    },
  );
}
