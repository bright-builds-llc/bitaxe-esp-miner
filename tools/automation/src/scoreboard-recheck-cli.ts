import { existsSync, realpathSync } from "node:fs";
import path from "node:path";

import { toolProgram } from "./cli-tools.js";
import { createLocalProcessPort } from "./process.js";
import {
  recheckScoreboardEvidence,
  type ScoreboardRecheckOptions,
} from "./scoreboard-recheck.js";

function workspaceRoot(): string {
  const configured = process.env["BUILD_WORKSPACE_DIRECTORY"];
  const starts = configured === undefined ? [process.cwd()] : [configured, process.cwd()];
  for (const start of starts) {
    let candidate = path.resolve(start);
    while (true) {
      const moduleFile = path.join(candidate, "MODULE.bazel");
      if (existsSync(moduleFile)) return path.dirname(realpathSync(moduleFile));
      const parent = path.dirname(candidate);
      if (parent === candidate) break;
      candidate = parent;
    }
  }
  throw new Error("workspace unavailable");
}

function parseOptions(args: readonly string[]): ScoreboardRecheckOptions {
  const expected = new Set([
    "--private-root",
    "--wrapper-root",
    "--capture-plan",
    "--capture-closure",
    "--evaluation-plan",
    "--projection",
  ]);
  const values = new Map<string, string>();
  for (let index = 0; index < args.length; index += 2) {
    const flag = args[index];
    const value = args[index + 1];
    if (flag === undefined || value === undefined || !expected.has(flag)
      || values.has(flag) || value.length === 0) {
      throw new Error("invalid invocation");
    }
    values.set(flag, value);
  }
  if (values.size !== expected.size) throw new Error("invalid invocation");
  const required = (flag: string): string => {
    const value = values.get(flag);
    if (value === undefined) throw new Error("invalid invocation");
    return value;
  };
  return {
    privateRoot: required("--private-root"),
    wrapperRoot: required("--wrapper-root"),
    capturePlan: required("--capture-plan"),
    captureClosure: required("--capture-closure"),
    evaluationPlan: required("--evaluation-plan"),
    projection: required("--projection"),
  };
}

async function main(): Promise<number> {
  try {
    const root = workspaceRoot();
    const evidence = await recheckScoreboardEvidence(
      root,
      parseOptions(process.argv.slice(2)),
      createLocalProcessPort({ cwd: root, timeoutMs: 30_000 }),
      "git",
      toolProgram(root, "crates/bitaxe-automation-contracts/validate_scoreboard_evidence"),
    );
    process.stdout.write(`${JSON.stringify({
      schema_version: "bitaxe-scoreboard-recheck-result-v2",
      status: "succeeded",
      projection_published: true,
      entry_count: evidence.scoreboard.entry_count,
      hardware_rerun_used: evidence.hardware_rerun_used,
    })}\n`);
    process.stderr.write("bitaxe-scoreboard-recheck-v2: completed\n");
    return 0;
  } catch (error) {
    process.stdout.write(`${JSON.stringify({
      schema_version: "bitaxe-scoreboard-recheck-result-v2",
      status: "failed",
      projection_published: false,
      stage: error instanceof Error ? scoreboardRecheckFailureStage(error.message) : "unexpected",
    })}\n`);
    process.stderr.write("bitaxe-scoreboard-recheck-v2: failed\n");
    return 1;
  }
}

process.exitCode = await main();

function scoreboardRecheckFailureStage(message: string): string {
  if (/path contract|absent before recheck/u.test(message)) return "preflight";
  if (/plan binding/u.test(message)) return "plan_binding";
  if (/mode|inventory|detector handoff/u.test(message)) return "protected_admission";
  if (/capture terminal|expected boundary/u.test(message)) return "capture_boundary";
  if (/source identity|source inventory/u.test(message)) return "evaluation_identity";
  if (/campaign/u.test(message)) return "campaign_quorum";
  if (/package identity|safe stop|post-restart identity/u.test(message)) return "runtime_identity";
  if (/retained observation/u.test(message)) return "scoreboard_quorum";
  if (/publication|child validation/u.test(message)) return "publication";
  return "unexpected";
}
