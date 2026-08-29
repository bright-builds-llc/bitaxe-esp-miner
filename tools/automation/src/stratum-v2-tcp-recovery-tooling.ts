import { access, lstat, realpath } from "node:fs/promises";
import { constants } from "node:fs";
import path from "node:path";

export class TcpPayloadRecoveryToolingError extends Error {
  public constructor(public readonly checkpoint: string) {
    super(checkpoint);
    this.name = "TcpPayloadRecoveryToolingError";
  }
}

const esptoolCandidates = [
  ".embuild/espressif/python_env/idf5.5_py3.14_env/bin/esptool.py",
  ".embuild/espressif/python_env/idf5.5_py3.9_env/bin/esptool.py",
] as const;
const nvsPythonRelative =
  ".embuild/espressif/python_env/idf5.5_py3.9_env/bin/python";

function fail(checkpoint: string): never {
  throw new TcpPayloadRecoveryToolingError(checkpoint);
}

async function containedEsptool(workspace: string): Promise<string> {
  const workspaceReal = await realpath(workspace);
  for (const relative of esptoolCandidates) {
    const candidate = path.join(workspace, relative);
    try {
      const metadata = await lstat(candidate);
      if (metadata.isSymbolicLink() || !metadata.isFile()) continue;
      const candidateReal = await realpath(candidate);
      const relativeReal = path.relative(workspaceReal, candidateReal);
      if (relativeReal !== "" && !relativeReal.startsWith("..")
        && !path.isAbsolute(relativeReal)) return candidate;
    } catch { continue; }
  }
  fail("restore_esptool");
}

export async function validateTcpPayloadRecoveryTooling(
  workspace: string,
  runProcess: (
    workspace: string,
    program: string,
    args: readonly string[],
    timeoutMillis: number,
  ) => Promise<{ readonly exitCode: number; readonly stdout: string; readonly stderr: string }>,
): Promise<void> {
  await containedEsptool(workspace);
  const nvsPython = path.join(workspace, nvsPythonRelative);
  try {
    await access(nvsPython, constants.X_OK);
  } catch { fail("restore_nvs_python"); }
  const importResult = await runProcess(
    workspace,
    nvsPython,
    ["-c", "import esp_idf_nvs_partition_gen"],
    10_000,
  );
  if (importResult.exitCode !== 0) fail("restore_nvs_python");
}
