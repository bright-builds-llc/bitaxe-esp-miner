import path from "node:path";

import {
  type JsonObject,
  type PreparedStratumV2Campaign,
} from "./stratum-v2-campaign-preflight.js";
import { validateRestorableInputs } from "./stratum-v2-campaign-settings.js";
import {
  parseInstalledIdentity,
  type RestoreBundle,
} from "./stratum-v2-restore-model.js";

type RuntimeCheckpoint =
  | "runtime_monitor_process"
  | "runtime_origin"
  | "runtime_settings"
  | "restoration_inputs"
  | "restore_package";

type RuntimeArgs = {
  readonly port: string;
  readonly wifiCredentials: string;
};

type RuntimeProcessResult = {
  readonly exitCode: number;
  readonly stdout: string;
};

type RuntimeDependencies = {
  readonly fail: (
    category: string,
    message: string,
    checkpoint: RuntimeCheckpoint,
  ) => never;
  readonly preparePreflight: () => Promise<PreparedStratumV2Campaign>;
  readonly runProcess: (
    workspace: string,
    program: string,
    args: readonly string[],
    timeoutMillis: number,
  ) => Promise<RuntimeProcessResult>;
  readonly restoreBundle: RestoreBundle;
  readonly restoreBundlePath: string;
};

export type PreparedStratumV2RuntimeAdmission = PreparedStratumV2Campaign & {
  readonly settings: JsonObject;
  readonly theme: JsonObject;
  readonly restoreBundle: RestoreBundle;
  readonly restoreBundlePath: string;
  readonly changedPackage: boolean;
};

function object(
  value: unknown,
  fail: RuntimeDependencies["fail"],
): JsonObject {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    fail("evidence_invalid", "same-origin response must be an object", "runtime_settings");
  }
  return value as JsonObject;
}

function requiredString(
  value: JsonObject,
  key: string,
  fail: RuntimeDependencies["fail"],
): string {
  const candidate = value[key];
  if (typeof candidate !== "string" || candidate.length === 0) {
    fail("evidence_invalid", "runtime identity is unavailable", "restore_package");
  }
  return candidate;
}

export function singleRuntimeOrigin(
  monitor: string,
  fail: RuntimeDependencies["fail"],
): URL {
  const candidates = [...monitor.matchAll(/\bhttps?:\/\/[A-Za-z0-9.-]+(?::[0-9]+)?\b/gu)]
    .map(match => match[0])
    .filter((value, index, all) => all.indexOf(value) === index);
  if (candidates.length !== 1 || candidates[0] === undefined) {
    fail("hardware_blocked", "monitor did not provide one current origin", "runtime_origin");
  }
  return new URL(candidates[0]);
}

export async function monitorRuntimeOrigin(
  workspace: string,
  flashProgram: string,
  port: string,
  runProcess: RuntimeDependencies["runProcess"],
  fail: RuntimeDependencies["fail"],
): Promise<URL> {
  let outcome: RuntimeProcessResult;
  try {
    outcome = await runProcess(workspace, flashProgram, [
      "monitor", "--board", "205", "--port", port, "--capture-timeout-seconds", "15",
    ], 30_000);
  } catch {
    fail("hardware_blocked", "passive monitor failed", "runtime_monitor_process");
  }
  if (outcome.exitCode !== 0) {
    fail("hardware_blocked", "passive monitor failed", "runtime_monitor_process");
  }
  return singleRuntimeOrigin(outcome.stdout, fail);
}

export async function fetchRuntimeObject(
  origin: URL,
  route: string,
  fail: RuntimeDependencies["fail"],
): Promise<JsonObject> {
  let response: Response;
  try {
    response = await fetch(new URL(route, origin));
  } catch {
    fail("hardware_blocked", "same-origin read failed", "runtime_settings");
  }
  if (!response.ok) fail("hardware_blocked", "same-origin read failed", "runtime_settings");
  let value: unknown;
  try {
    value = await response.json();
  } catch {
    fail("evidence_invalid", "same-origin response is malformed", "runtime_settings");
  }
  return object(value, fail);
}

export async function prepareStratumV2RuntimeAdmission(
  workspace: string,
  args: RuntimeArgs,
  dependencies: RuntimeDependencies,
): Promise<PreparedStratumV2RuntimeAdmission> {
  const preflight = await dependencies.preparePreflight();
  const origin = await monitorRuntimeOrigin(
    workspace,
    path.join(workspace, "bazel-bin/tools/flash/flash"),
    args.port,
    dependencies.runProcess,
    dependencies.fail,
  );
  const settings = await fetchRuntimeObject(origin, "/api/system/info", dependencies.fail);
  const theme = await fetchRuntimeObject(origin, "/api/theme", dependencies.fail);
  await validateRestorableInputs(settings, preflight.wifiPath, preflight.poolPath, (category, message) => {
    dependencies.fail(category, message, "restoration_inputs");
  });
  const installedIdentity = parseInstalledIdentity(settings);
  if (JSON.stringify(installedIdentity) !== JSON.stringify(dependencies.restoreBundle.installed_identity)) {
    dependencies.fail("hardware_blocked", "restore bundle identity mismatch", "restore_package");
  }
  const currentAppElf = requiredString(settings, "appElfSha256", dependencies.fail);
  return {
    ...preflight,
    settings,
    theme,
    restoreBundle: dependencies.restoreBundle,
    restoreBundlePath: dependencies.restoreBundlePath,
    changedPackage: currentAppElf
      !== requiredString(preflight.manifest, "app_elf_sha256", dependencies.fail),
  };
}
