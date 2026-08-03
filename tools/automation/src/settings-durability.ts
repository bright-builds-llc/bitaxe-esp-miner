import { createHash } from "node:crypto";
import { access, chmod, mkdir, readFile, stat, writeFile } from "node:fs/promises";
import path from "node:path";

import { flashCommand, flashMonitorCommand, internalCommandSpec, monitorCommand } from "./contracts.generated.js";
import { fetchJsonFromSameOrigin, sendSameOriginRequest, uniqueRuntimeOrigin } from "./http.js";
import type { ProcessPort, RunningProcess } from "./process.js";
import { hasPassiveSafeState } from "./version-evidence.js";
import { assertWithinWorkspace } from "./workspace.js";

export type SettingsDurabilityOptions = {
  readonly privateRoot: string;
  readonly packageManifest: string;
  readonly wifiCredentials: string;
  readonly port: string;
  readonly projection: string;
  readonly captureTimeoutSeconds: number;
};

type JsonObject = Readonly<Record<string, unknown>>;

function sha256(value: string): string {
  return createHash("sha256").update(value).digest("hex");
}

function jsonObject(value: unknown, context: string): JsonObject {
  if (typeof value !== "object" || value === null || Array.isArray(value)) throw new Error(`${context} must be an object`);
  return value as JsonObject;
}

function requiredString(value: JsonObject, field: string, context: string): string {
  const candidate = value[field];
  if (typeof candidate !== "string" || candidate === "") throw new Error(`${context} ${field} must be a non-empty string`);
  return candidate;
}

async function requireAbsentPrivateRoot(privateRoot: string): Promise<void> {
  try {
    await stat(privateRoot);
    throw new Error("private attempt root must be absent before launch");
  } catch (error) {
    if (error instanceof Error && error.message === "private attempt root must be absent before launch") throw error;
    if ((error as NodeJS.ErrnoException).code !== "ENOENT") throw error;
  }
  await mkdir(privateRoot, { mode: 0o700, recursive: true });
  await chmod(privateRoot, 0o700);
}

async function classify(
  processPort: ProcessPort,
  classifierProgram: string,
  trace: string,
  mode: "baseline" | "post-restart",
  maybeExpected?: { readonly session: string; readonly ordinal: number },
): Promise<JsonObject> {
  const args = ["verify-settings-durability", "--trace", trace, "--mode", mode];
  if (maybeExpected !== undefined) {
    args.push("--expected-session", maybeExpected.session, "--expected-ordinal", String(maybeExpected.ordinal));
  }
  const outcome = await processPort.run(internalCommandSpec(classifierProgram, args, (value) => value));
  if (outcome.exitCode !== 0) throw new Error(`${mode} settings classification failed`);
  const value = jsonObject(JSON.parse(outcome.stdout), `${mode} classification`);
  if (value["status"] !== "passed") throw new Error(`${mode} settings evidence did not pass`);
  return value;
}

async function hostname(origin: URL, output: string): Promise<string> {
  const value = jsonObject(await fetchJsonFromSameOrigin(origin, "/api/system/info", output), "system info");
  return requiredString(value, "hostname", "system info");
}

export async function captureSettingsDurability(
  workspaceRoot: string,
  options: SettingsDurabilityOptions,
  processPort: ProcessPort,
  flashProgram: string,
  classifierProgram: string,
  waitForMonitorReady: () => Promise<void> = () => new Promise((resolve) => setTimeout(resolve, 5_000)),
): Promise<unknown> {
  const privateRoot = assertWithinWorkspace(workspaceRoot, options.privateRoot);
  const manifestPath = assertWithinWorkspace(workspaceRoot, options.packageManifest);
  const credentialsPath = assertWithinWorkspace(workspaceRoot, options.wifiCredentials);
  const projectionPath = assertWithinWorkspace(workspaceRoot, options.projection);
  await access(manifestPath);
  await access(credentialsPath);
  await requireAbsentPrivateRoot(privateRoot);
  const manifestDocument = await readFile(manifestPath, "utf8");
  const manifest = jsonObject(JSON.parse(manifestDocument), "package manifest");
  const sourceCommit = requiredString(manifest, "source_commit", "package manifest");
  const referenceCommit = requiredString(manifest, "reference_commit", "package manifest");
  const manifestDigest = sha256(manifestDocument);
  const initialRoot = path.join(privateRoot, "initial");
  await mkdir(initialRoot, { mode: 0o700 });
  const initial = await processPort.run(flashMonitorCommand(flashProgram, {
    board: 205,
    port: options.port,
    manifest: manifestPath,
    wifiCredentials: credentialsPath,
    captureTimeoutSeconds: options.captureTimeoutSeconds,
    evidenceMode: "dual",
    evidenceDir: initialRoot,
  }));
  if (initial.exitCode !== 0) throw new Error("exact-package flash-monitor failed");
  const initialTrace = path.join(initialRoot, "flash-monitor.classifier-input.log");
  const initialDocument = await readFile(initialTrace, "utf8");
  if (!hasPassiveSafeState(initialDocument)) throw new Error("initial boot lacks safe-state evidence");
  const baseline = await classify(processPort, classifierProgram, initialTrace, "baseline");
  const session = requiredString(baseline, "session", "baseline classification");
  const ordinalValue = baseline["boot_ordinal"];
  if (typeof ordinalValue !== "number" || !Number.isSafeInteger(ordinalValue) || ordinalValue < 1) {
    throw new Error("baseline boot ordinal is invalid");
  }
  let origin = uniqueRuntimeOrigin(initialDocument);
  const originalHostname = await hostname(origin, path.join(privateRoot, "original.private.json"));
  const testHostname = originalHostname === "bitaxe-parity-205" ? "bitaxe-parity-alt" : "bitaxe-parity-205";
  let hostnameChanged = false;
  let restorationComplete = false;
  let maybePostMonitor: RunningProcess | undefined;
  try {
    await sendSameOriginRequest(origin, "/api/system", "PATCH", path.join(privateRoot, "patch.private.txt"), { hostname: testHostname });
    hostnameChanged = true;
    if (await hostname(origin, path.join(privateRoot, "immediate.private.json")) !== testHostname) {
      throw new Error("immediate hostname readback mismatch");
    }
    const postRoot = path.join(privateRoot, "post-restart");
    await mkdir(postRoot, { mode: 0o700 });
    maybePostMonitor = processPort.start(monitorCommand(flashProgram, {
      board: 205,
      port: options.port,
      evidenceDir: postRoot,
      captureTimeoutSeconds: options.captureTimeoutSeconds,
    }));
    await waitForMonitorReady();
    await sendSameOriginRequest(origin, "/api/system/restart", "POST", path.join(privateRoot, "restart.private.txt"));
    const postMonitor = await maybePostMonitor.wait();
    maybePostMonitor = undefined;
    if (postMonitor.exitCode !== 0) throw new Error("post-restart monitor failed");
    const postTrace = path.join(postRoot, "flash-monitor.log");
    const postDocument = await readFile(postTrace, "utf8");
    if (!hasPassiveSafeState(postDocument)) throw new Error("post-restart boot lacks safe-state evidence");
    await classify(processPort, classifierProgram, postTrace, "post-restart", { session, ordinal: ordinalValue });
    origin = uniqueRuntimeOrigin(postDocument);
    if (await hostname(origin, path.join(privateRoot, "post-restart.private.json")) !== testHostname) {
      throw new Error("post-restart hostname readback mismatch");
    }
    await sendSameOriginRequest(origin, "/api/system", "PATCH", path.join(privateRoot, "restore.private.txt"), { hostname: originalHostname });
    if (await hostname(origin, path.join(privateRoot, "restored.private.json")) !== originalHostname) {
      throw new Error("restored hostname readback mismatch");
    }
    restorationComplete = true;
    hostnameChanged = false;
  } catch (error) {
    if (maybePostMonitor !== undefined) {
      maybePostMonitor.terminate();
      await maybePostMonitor.wait();
      maybePostMonitor = undefined;
    }
    if (hostnameChanged) {
      try {
        await sendSameOriginRequest(origin, "/api/system", "PATCH", path.join(privateRoot, "recovery-restore.private.txt"), { hostname: originalHostname });
        restorationComplete = await hostname(origin, path.join(privateRoot, "recovery-readback.private.json")) === originalHostname;
      } catch {
        const recovery = await processPort.run(flashCommand(flashProgram, {
          board: 205,
          port: options.port,
          manifest: manifestPath,
          wifiCredentials: credentialsPath,
        }));
        if (recovery.exitCode !== 0) throw new Error("hostname restoration and exact-package recovery failed");
      }
    }
    throw error;
  }
  if (!restorationComplete) throw new Error("hostname restoration was not confirmed");
  const evidence = {
    schema_version: "bitaxe-settings-durability-evidence-v1",
    board: 205,
    source_commit: sourceCommit,
    reference_commit: referenceCommit,
    package_manifest_sha256: manifestDigest,
    workflow: {
      schema_version: "bitaxe-workflow-identity-v1",
      command: "verify-settings-durability",
      request_sha256: sha256(JSON.stringify({ manifest: manifestDigest, port: options.port, timeout: options.captureTimeoutSeconds })),
    },
    boot_observed: true,
    hostname_patch_readback: true,
    normal_restart_observed: true,
    post_restart_persistence: true,
    restoration_complete: true,
    mining_state: "disabled",
    hardware_control_state: "disabled",
    redaction_status: "passed",
  } as const;
  await mkdir(path.dirname(projectionPath), { recursive: true });
  await writeFile(projectionPath, `${JSON.stringify(evidence, null, 2)}\n`, { encoding: "utf8", flag: "wx" });
  return evidence;
}
