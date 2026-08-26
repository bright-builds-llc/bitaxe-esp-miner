import { readFile, readdir, stat } from "node:fs/promises";
import { createHash } from "node:crypto";
import path from "node:path";

export type JsonObject = Record<string, unknown>;

type PreflightArgs = {
  readonly packageManifest: string;
  readonly wifiCredentials: string;
  readonly privateRoot: string;
  readonly projection: string;
  readonly plan: string;
  readonly campaignOrdinal: 5;
};

type PreflightCheckpoint =
  | "outputs_absent"
  | "private_path_ignored"
  | "wifi_restore_input"
  | "pool_restore_input"
  | "source_identity";

type ProcessResult = {
  readonly exitCode: number;
  readonly stdout: string;
};

type PreflightDependencies = {
  readonly runProcess: (
    workspace: string,
    program: string,
    args: readonly string[],
    timeoutMillis: number,
  ) => Promise<ProcessResult>;
  readonly fail: (
    category: string,
    message: string,
    checkpoint: PreflightCheckpoint,
  ) => never;
};

export type PreparedStratumV2Campaign = {
  readonly privateRoot: string;
  readonly projection: string;
  readonly manifestPath: string;
  readonly wifiPath: string;
  readonly poolPath: string;
  readonly manifestDocument: string;
  readonly manifest: JsonObject;
  readonly head: string;
};

const restoreProjectionStatus =
  "?? docs/parity/evidence/str005-installed-package-recovery/restore-readiness-projection-006.json";
const planSha256 = "14c7676fb26b6291a24d08d229bc38717691835978d61ae24fd8cff91736470a";
const taskId = "task-str005-inactive-restoration-and-campaign-continuation";

async function requireMode(
  candidate: string,
  mode: number,
  checkpoint: PreflightCheckpoint,
  dependencies: PreflightDependencies,
): Promise<void> {
  let metadata;
  try {
    metadata = await stat(candidate);
  } catch {
    dependencies.fail("evidence_invalid", "protected input is unavailable", checkpoint);
  }
  if (!metadata.isFile() || (metadata.mode & 0o777) !== mode) {
    dependencies.fail("evidence_invalid", "protected input mode is invalid", checkpoint);
  }
}

async function requireAbsent(
  candidate: string,
  dependencies: PreflightDependencies,
): Promise<void> {
  try {
    await stat(candidate);
    dependencies.fail(
      "evidence_invalid",
      "fresh campaign output already exists",
      "outputs_absent",
    );
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code === "ENOENT") return;
    dependencies.fail("evidence_invalid", "campaign output state is unavailable", "outputs_absent");
  }
}

async function runAtCheckpoint(
  workspace: string,
  program: string,
  args: readonly string[],
  checkpoint: PreflightCheckpoint,
  dependencies: PreflightDependencies,
): Promise<ProcessResult> {
  try {
    return await dependencies.runProcess(workspace, program, args, 5_000);
  } catch {
    dependencies.fail("process_failed", "preflight child failed", checkpoint);
  }
}

async function poolRestoreInput(
  workspace: string,
  dependencies: PreflightDependencies,
): Promise<string> {
  let names: string[];
  try {
    names = await readdir(workspace);
  } catch {
    dependencies.fail(
      "hardware_blocked",
      "restoration pool inventory is unavailable",
      "pool_restore_input",
    );
  }
  const candidates = names
    .filter(name => /^pool-credentials(?:-[A-Za-z0-9_-]+)?\.json$/u.test(name))
    .map(name => path.join(workspace, name));
  if (candidates.length !== 1 || candidates[0] === undefined) {
    dependencies.fail(
      "hardware_blocked",
      "exactly one ignored restoration pool input is required",
      "pool_restore_input",
    );
  }
  await requireMode(candidates[0], 0o600, "pool_restore_input", dependencies);
  return candidates[0];
}

export async function prepareStratumV2Campaign(
  workspace: string,
  args: PreflightArgs,
  dependencies: PreflightDependencies,
): Promise<PreparedStratumV2Campaign> {
  const privateRoot = path.resolve(workspace, args.privateRoot);
  const projection = path.resolve(workspace, args.projection);
  const manifestPath = path.resolve(workspace, args.packageManifest);
  const wifiPath = path.resolve(workspace, args.wifiCredentials);
  const planPath = path.resolve(workspace, args.plan);
  await requireAbsent(privateRoot, dependencies);
  await requireAbsent(projection, dependencies);
  const ignored = await runAtCheckpoint(
    workspace,
    "git",
    ["check-ignore", "-q", args.privateRoot],
    "private_path_ignored",
    dependencies,
  );
  if (ignored.exitCode !== 0) {
    dependencies.fail(
      "evidence_invalid",
      "private campaign root is not ignored",
      "private_path_ignored",
    );
  }
  await requireMode(wifiPath, 0o600, "wifi_restore_input", dependencies);
  let plan: string;
  let tasks: string;
  try {
    plan = await readFile(planPath, "utf8");
    tasks = await readFile(path.join(workspace, "TASKS.md"), "utf8");
  } catch {
    dependencies.fail("evidence_invalid", "campaign plan binding is unavailable", "source_identity");
  }
  if (args.campaignOrdinal !== 5
    || createHash("sha256").update(plan).digest("hex") !== planSha256
    || !tasks.includes(`### ${taskId}`)) {
    dependencies.fail("evidence_invalid", "campaign plan binding is invalid", "source_identity");
  }
  const poolPath = await poolRestoreInput(workspace, dependencies);
  let manifestDocument: string;
  let manifest: JsonObject;
  try {
    manifestDocument = await readFile(manifestPath, "utf8");
    const value: unknown = JSON.parse(manifestDocument);
    if (typeof value !== "object" || value === null || Array.isArray(value)) throw new Error();
    manifest = value as JsonObject;
  } catch {
    dependencies.fail("evidence_invalid", "package manifest is unavailable", "source_identity");
  }
  const headResult = await runAtCheckpoint(
    workspace,
    "git",
    ["rev-parse", "HEAD"],
    "source_identity",
    dependencies,
  );
  const status = await runAtCheckpoint(
    workspace,
    "git",
    ["status", "--porcelain", "--untracked-files=all"],
    "source_identity",
    dependencies,
  );
  const head = headResult.stdout.trim();
  const unexpectedStatus = status.stdout
    .split(/\r?\n/u)
    .filter(line => line.length > 0 && line !== restoreProjectionStatus);
  if (status.exitCode !== 0 || unexpectedStatus.length !== 0
    || typeof manifest["source_commit"] !== "string"
    || manifest["source_commit"] !== head) {
    dependencies.fail(
      "evidence_invalid",
      "campaign source or package is not exact clean HEAD",
      "source_identity",
    );
  }
  return {
    privateRoot,
    projection,
    manifestPath,
    wifiPath,
    poolPath,
    manifestDocument,
    manifest,
    head,
  };
}
