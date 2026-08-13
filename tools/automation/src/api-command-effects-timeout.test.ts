import assert from "node:assert/strict";
import { chmod, mkdtemp, mkdir, readFile, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import {
  ApiCommandEffectsError,
  captureApiCommandEffects,
  type ApiCommandEffectsOptions,
} from "./api-command-effects.js";
import { COMMAND_EFFECTS_TRANSACTION_BUDGET } from "./api-command-effects-budget.js";
import { createFakeProcessPort, type ProcessOutcome } from "./process.js";

const ok = (): ProcessOutcome => ({ exitCode: 0, stdout: "", stderr: "", timedOut: false });

async function privateJson(output: string, value: unknown): Promise<void> {
  await writeFile(output, `${JSON.stringify(value)}\n`, { mode: 0o600 });
  await chmod(output, 0o600);
}

async function fixture(): Promise<{ root: string; options: ApiCommandEffectsOptions }> {
  const root = await mkdtemp(path.join(os.tmpdir(), "api-command-effects-timeout-"));
  await writeFile(path.join(root, "MODULE.bazel"), "module(name = \"fixture\")\n");
  await mkdir(path.join(root, "inputs"));
  const manifest = path.join(root, "inputs", "package.json");
  const wifi = path.join(root, "inputs", "wifi.json");
  await writeFile(manifest, JSON.stringify({
    source_commit: "a".repeat(40),
    reference_commit: "b".repeat(40),
    app_elf_sha256: "c".repeat(64),
  }));
  await writeFile(wifi, "{}\n");
  return {
    root,
    options: {
      privateRoot: "scratch/attempt-001",
      packageManifest: manifest,
      wifiCredentials: wifi,
      port: "/dev/private-sensitive-port",
      projection: path.join(root, "docs", "api-command-effects.json"),
      durationSeconds: 600,
    },
  };
}

async function waitForStop(attempt: string): Promise<void> {
  while (true) {
    try {
      await readFile(path.join(attempt, "fixture.stop.private"), "utf8");
      return;
    } catch (error) {
      if ((error as NodeJS.ErrnoException).code !== "ENOENT") throw error;
      await new Promise((resolve) => setTimeout(resolve, 1));
    }
  }
}

function timedOutPort(
  root: string,
  recoveryMode: "closed" | "missing" | "malformed",
  observeTimeout: (maybeTimeoutMillis: number | undefined) => void,
) {
  return createFakeProcessPort(async (spec, maybeTimeoutMillis) => {
    if (spec.program === "/sbin/route") return { ...ok(), stdout: "interface: en0\n" };
    if (spec.program === "/usr/sbin/ipconfig") return { ...ok(), stdout: "192.0.2.44\n" };
    const attempt = path.join(root, "scratch", "attempt-001");
    if (spec.program.endsWith("api-command-effects-stratum-pool")) {
      await privateJson(path.join(attempt, "fixture-ready.private.json"), {
        status: "ready", fixture: "api-command-effects-v1", bound_port: 43210,
      });
      await waitForStop(attempt);
      await privateJson(path.join(attempt, "fixture-report.private.json"), {
        status: "stopped", fixture: "api-command-effects-v1",
      });
      return ok();
    }
    if (spec.program.endsWith("flash")) {
      observeTimeout(maybeTimeoutMillis);
      if (recoveryMode !== "missing") {
        const campaign = path.join(attempt, "campaign");
        await mkdir(campaign, { mode: 0o700 });
        if (recoveryMode === "closed") {
          await privateJson(path.join(campaign, "campaign-result.json"), {
            safe_stop: "confirmed", usb_cleanup: "ready",
          });
        } else {
          await writeFile(path.join(campaign, "campaign-result.json"), "not-json\n", { mode: 0o600 });
        }
        await privateJson(path.join(campaign, "campaign-network.private.json"), {
          recovery_pause_request_count: 1,
        });
      }
      return { ...ok(), exitCode: 1, timedOut: true };
    }
    throw new Error(`unexpected child ${spec.program}`);
  });
}

for (const recoveryMode of ["closed", "missing", "malformed"] as const) {
  test(`outer timeout preserves primary category with ${recoveryMode} recovery`, async () => {
    // Arrange
    const value = await fixture();
    let maybeObservedTimeout: number | undefined;

    // Act
    const error = await captureApiCommandEffects(
      value.root,
      value.options,
      timedOutPort(value.root, recoveryMode, (value) => { maybeObservedTimeout = value; }),
      path.join(value.root, "bin", "api-command-effects-stratum-pool"),
      path.join(value.root, "bin", "flash"),
      path.join(value.root, "bin", "device-session"),
      () => undefined,
    ).then(() => undefined, (caught: unknown) => caught);

    // Assert
    assert(error instanceof ApiCommandEffectsError);
    assert.equal(error.category, "timeout");
    assert.equal(maybeObservedTimeout, COMMAND_EFFECTS_TRANSACTION_BUDGET.parentTimeoutMillis);
    assert.deepEqual(error.publicValue, {
      stage: "command_effects",
      safe_stop_confirmed: recoveryMode === "closed",
      cleanup_complete: recoveryMode === "closed",
      recovery_attempted: recoveryMode === "closed",
      secondary_recovery_failure: false,
    });
    await assert.rejects(readFile(value.options.projection, "utf8"), { code: "ENOENT" });
  });
}
