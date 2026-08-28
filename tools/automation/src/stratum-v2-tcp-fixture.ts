import { spawn, type ChildProcess } from "node:child_process";
import path from "node:path";

import { terminateManagedProcessGroup } from "./stratum-v2-tcp-payload-process.js";

const maximumOutputBytes = 1_048_576;

export type TcpPayloadFixtureOwner = {
  readonly child: ChildProcess;
  readonly completion: Promise<number>;
  readonly stdout: Buffer[];
  readonly stderr: Buffer[];
};

export function startTcpPayloadFixture(
  workspace: string,
  fixtureRoot: string,
  host: string,
  expectedPeer: string,
): TcpPayloadFixtureOwner {
  const child = spawn(
    path.join(workspace, "bazel-bin/tools/stratum-v2-fixture/stratum_v2_fixture"),
    tcpPayloadFixtureArgs(fixtureRoot, host, expectedPeer),
    { cwd: workspace, env: process.env, detached: true, stdio: ["ignore", "pipe", "pipe"] },
  );
  const stdout: Buffer[] = [];
  const stderr: Buffer[] = [];
  let outputBytes = 0;
  const capture = (destination: Buffer[]) => (chunk: Buffer) => {
    outputBytes += chunk.length;
    if (outputBytes <= maximumOutputBytes) destination.push(chunk);
    else terminateTcpPayloadFixture(child);
  };
  child.stdout?.on("data", capture(stdout));
  child.stderr?.on("data", capture(stderr));
  const completion = new Promise<number>((resolve, reject) => {
    child.once("error", reject);
    child.once("close", code => resolve(code ?? 1));
  });
  return { child, completion, stdout, stderr };
}

export function tcpPayloadFixtureArgs(
  fixtureRoot: string,
  host: string,
  expectedPeer: string,
): string[] {
  return [
    "--private-root", fixtureRoot, "--listen-address", `${host}:0`,
    "--accept-timeout-seconds", "300", "--session-timeout-seconds", "120",
    "--mode", "tcp-payload", "--expected-peer-address", expectedPeer,
  ];
}

export function terminateTcpPayloadFixture(child: ChildProcess): void {
  terminateManagedProcessGroup(child);
}
