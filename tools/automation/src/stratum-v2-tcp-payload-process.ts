import { spawn, type ChildProcess } from "node:child_process";

export type ManagedDiagnosticProcessResult = {
  readonly exitCode: number;
  readonly stdout: string;
  readonly stderr: string;
};

export class ManagedDiagnosticProcessError extends Error {
  public constructor(
    public readonly category: "timeout" | "evidence_invalid",
    public readonly checkpoint: string,
  ) {
    super(`${category}:${checkpoint}`);
    this.name = "ManagedDiagnosticProcessError";
  }
}
const maximumOutputBytes = 1_048_576;

export function tcpPayloadDiagnosticValidatorArgs(
  candidate: string,
  expectedSource: string,
  expectedOrdinal: number,
): readonly string[] {
  return [
    "run", "//tools/automation:stratum_v2_tcp_payload_validator", "--",
    candidate, expectedSource, String(expectedOrdinal),
  ];
}

export function terminateManagedProcessGroup(child: ChildProcess): void {
  if (child.pid === undefined || child.exitCode !== null) return;
  try { process.kill(-child.pid, "SIGTERM"); }
  catch { child.kill("SIGTERM"); }
}

export async function runManagedDiagnosticProcess(
  workspace: string,
  program: string,
  args: readonly string[],
  timeoutMillis: number,
  checkpoint: string,
): Promise<ManagedDiagnosticProcessResult> {
  return new Promise((resolve, reject) => {
    const child = spawn(program, [...args], {
      cwd: workspace,
      env: process.env,
      detached: true,
      stdio: ["ignore", "pipe", "pipe"],
    });
    const stdout: Buffer[] = [];
    const stderr: Buffer[] = [];
    let outputBytes = 0;
    let terminal: "running" | "timeout" | "output_limit" = "running";
    let forceTimer: NodeJS.Timeout | undefined;
    const terminate = () => {
      terminateManagedProcessGroup(child);
      forceTimer = setTimeout(() => {
        if (child.pid === undefined || child.exitCode !== null) return;
        try { process.kill(-child.pid, "SIGKILL"); }
        catch { child.kill("SIGKILL"); }
      }, 2_000);
    };
    const capture = (destination: Buffer[], chunk: Buffer) => {
      outputBytes += chunk.length;
      if (outputBytes > maximumOutputBytes) {
        if (terminal === "running") {
          terminal = "output_limit";
          terminate();
        }
        return;
      }
      destination.push(chunk);
    };
    child.stdout?.on("data", (chunk: Buffer) => capture(stdout, chunk));
    child.stderr?.on("data", (chunk: Buffer) => capture(stderr, chunk));
    const timeout = setTimeout(() => {
      if (terminal !== "running") return;
      terminal = "timeout";
      terminate();
    }, timeoutMillis);
    child.once("error", reject);
    child.once("close", exitCode => {
      clearTimeout(timeout);
      if (forceTimer !== undefined) clearTimeout(forceTimer);
      if (terminal === "timeout" || terminal === "output_limit") {
        reject(new ManagedDiagnosticProcessError(
          terminal === "timeout" ? "timeout" : "evidence_invalid",
          checkpoint,
        ));
        return;
      }
      resolve({
        exitCode: exitCode ?? 1,
        stdout: Buffer.concat(stdout).toString("utf8"),
        stderr: Buffer.concat(stderr).toString("utf8"),
      });
    });
  });
}
