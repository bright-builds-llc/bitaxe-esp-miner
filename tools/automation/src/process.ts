import { spawn } from "node:child_process";

import type { CommandSpec } from "./contracts.generated.js";

export type ProcessOutcome = {
  readonly exitCode: number;
  readonly stdout: string;
  readonly stderr: string;
  readonly timedOut: boolean;
};

export type ProcessPort = {
  readonly run: (spec: CommandSpec<unknown>) => Promise<ProcessOutcome>;
  readonly loadEspEnvironment: () => Promise<Readonly<Record<string, string>>>;
};

export function createFakeProcessPort(
  run: ProcessPort["run"],
  environment: Readonly<Record<string, string>> = {},
): ProcessPort {
  return {
    run,
    async loadEspEnvironment() {
      return environment;
    },
  };
}

export function allowedEnvironment(source: NodeJS.ProcessEnv | Readonly<Record<string, string>>): Record<string, string> {
  const allowed: Record<string, string> = {};
  const exact = new Set([
    "PATH", "HOME", "USER", "TMPDIR", "LANG", "LC_ALL", "TERM", "SHELL", "VIRTUAL_ENV",
    "LIBCLANG_PATH", "MCU", "OPENOCD_SCRIPTS", "PYTHONNOUSERSITE", "CCACHE_DIR",
    "CARGO_HOME", "CARGO_NET_GIT_FETCH_WITH_CLI", "CARGO_TARGET_DIR", "CARGO_TERM_COLOR",
    "RUSTUP_HOME", "RUSTUP_TOOLCHAIN", "RUST_BACKTRACE",
    "BITAXE_BUILD_PROVENANCE_STAMP", "BITAXE_BUILD_TIMESTAMP_UTC_FILE",
  ]);
  const prefixes = ["IDF_", "ESP_"];
  for (const [key, maybeValue] of Object.entries(source)) {
    if (maybeValue === undefined) continue;
    if (/(?:TOKEN|PASSWORD|SECRET|CREDENTIAL|API_KEY)/iu.test(key)) continue;
    if (exact.has(key) || prefixes.some((prefix) => key.startsWith(prefix))) allowed[key] = maybeValue;
  }
  return allowed;
}

export function createLocalProcessPort(options: { readonly cwd: string; readonly timeoutMs: number }): ProcessPort {
  const run: ProcessPort["run"] = (spec) => {
    const child = spawn(spec.program, spec.args, {
      cwd: options.cwd,
      env: allowedEnvironment(spec.environment ?? process.env),
      stdio: ["ignore", "pipe", "pipe"],
    });
    const stdout: Buffer[] = [];
    const stderr: Buffer[] = [];
    child.stdout.on("data", (chunk: Buffer) => stdout.push(chunk));
    child.stderr.on("data", (chunk: Buffer) => stderr.push(chunk));
    let timedOut = false;
    let killTimeout: NodeJS.Timeout | undefined;
    const timeout = setTimeout(() => {
      timedOut = true;
      child.kill("SIGTERM");
      killTimeout = setTimeout(() => child.kill("SIGKILL"), 5_000);
    }, options.timeoutMs);
    return new Promise<ProcessOutcome>((resolve, reject) => {
      child.once("error", (error) => {
        clearTimeout(timeout);
        if (killTimeout !== undefined) clearTimeout(killTimeout);
        reject(error);
      });
      child.once("close", (exitCode) => {
        clearTimeout(timeout);
        if (killTimeout !== undefined) clearTimeout(killTimeout);
        resolve({
          exitCode: exitCode ?? 1,
          stdout: Buffer.concat(stdout).toString("utf8"),
          stderr: Buffer.concat(stderr).toString("utf8"),
          timedOut,
        });
      });
    });
  };
  return {
    loadEspEnvironment() {
      return new Promise((resolve, reject) => {
        const child = spawn(
          "/bin/zsh",
          ["-lc", "source \"$HOME/export-esp.sh\" >/dev/null 2>&1 && /usr/bin/env -0"],
          { cwd: options.cwd, env: allowedEnvironment(process.env), stdio: ["ignore", "pipe", "pipe"] },
        );
        const stdout: Buffer[] = [];
        const stderr: Buffer[] = [];
        child.stdout.on("data", (chunk: Buffer) => stdout.push(chunk));
        child.stderr.on("data", (chunk: Buffer) => stderr.push(chunk));
        child.once("error", reject);
        child.once("close", (exitCode) => {
          if (exitCode !== 0) {
            reject(new Error(`ESP environment loader failed with exit ${String(exitCode)}`));
            return;
          }
          const environment: Record<string, string> = {};
          for (const entry of Buffer.concat(stdout).toString("utf8").split("\0")) {
            const separator = entry.indexOf("=");
            if (separator <= 0) continue;
            environment[entry.slice(0, separator)] = entry.slice(separator + 1);
          }
          resolve(allowedEnvironment(environment));
        });
      });
    },
    run,
  };
}
