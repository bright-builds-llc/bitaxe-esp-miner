import { chmod, lstat, mkdir, readFile, rename, writeFile } from "node:fs/promises";
import path from "node:path";

import type { JsonObject } from "./stratum-v2-noise-connection.js";

type RunProcess = (
  workspace: string,
  program: string,
  args: readonly string[],
  timeoutMillis: number,
) => Promise<{ readonly exitCode: number }>;

export async function publishNoiseAuthProjection(
  workspace: string,
  privateRoot: string,
  publicProjection: string,
  projection: JsonObject,
  expectedSource: string,
  expectedOrdinal: number,
  runProcess: RunProcess,
): Promise<void> {
  const privateCandidate = path.join(workspace, privateRoot, "projection-candidate.private.json");
  const document = `${JSON.stringify(projection, null, 2)}\n`;
  try {
    const metadata = await lstat(privateCandidate);
    if (metadata.isSymbolicLink() || !metadata.isFile() || (metadata.mode & 0o777) !== 0o600
      || await readFile(privateCandidate, "utf8") !== document) {
      throw new Error("projection_candidate_conflict");
    }
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code !== "ENOENT") throw error;
    await writeFile(privateCandidate, document, { flag: "wx", mode: 0o600 });
    await chmod(privateCandidate, 0o600);
  }
  const validator = path.join(
    workspace,
    "bazel-bin/tools/automation/stratum_v2_noise_auth_validator_/stratum_v2_noise_auth_validator",
  );
  const validated = await runProcess(
    workspace,
    "/usr/bin/env",
    ["BAZEL_BINDIR=.", validator, privateCandidate, expectedSource, String(expectedOrdinal)],
    60_000,
  );
  if (validated.exitCode !== 0) throw new Error("projection_rejected");
  const publicPath = path.join(workspace, publicProjection);
  await mkdir(path.dirname(publicPath), { recursive: true });
  const temporary = `${publicPath}.tmp`;
  await writeFile(temporary, document, { flag: "wx", mode: 0o644 });
  await chmod(temporary, 0o644);
  await rename(temporary, publicPath);
}
