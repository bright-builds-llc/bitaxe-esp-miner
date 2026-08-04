import assert from "node:assert/strict";
import { chmod, mkdtemp, mkdir, readFile, stat, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import { createFakeProcessPort, createLocalProcessPort, type ProcessOutcome, type ProcessPort } from "./process.js";
import { captureThemeDurability, ThemeDurabilityError } from "./theme-durability.js";

const ok = (stdout = ""): ProcessOutcome => ({ exitCode: 0, stdout, stderr: "", timedOut: false });
const failed = (): ProcessOutcome => ({ exitCode: 1, stdout: "", stderr: "", timedOut: false });
const trace = "safe_state: mining=disabled asic_work_submission=disabled hardware_control=disabled\nruntime_origin session=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa device_url=http://127.0.0.1:8080 redacted=true\n";
const nodeProgram = process.env["JS_BINARY__NODE_BINARY"] ?? process.execPath;

const readyProjection = {
  schema_version: "esp-device-session-v1",
  terminal_category: "ready",
  platform_category: "macos",
  board_category: "205",
  same_physical_device: true,
  stable_enumeration: true,
  reenumerated: false,
  reader_armed: true,
  pre_restart_serial_delivery: true,
  post_restart_serial_delivery: true,
  serial_delivery: "correlated",
  request_outcome: "response_received",
  request_attempt_count: 1,
  service_loss_observed: true,
  trusted_origin_preserved: true,
  application_recovered: true,
  build_identity_matches: true,
  boot_session_changed: true,
  boot_ordinal_advanced_by_one: true,
  software_reset_observed: true,
  postcondition_matches: true,
  usb_disappearance_count: 0,
  enumeration_change_count: 0,
  serial_byte_count: 128,
  http_observation_count: 3,
  duration_millis: 1_000,
  cleanup_complete: true,
} as const;

type Fixture = {
  readonly root: string;
  readonly manifest: string;
  readonly credentials: string;
  readonly projection: string;
};

async function fixture(name: string): Promise<Fixture> {
  const root = await mkdtemp(path.join(os.tmpdir(), `bitaxe-theme-${name}-`));
  await writeFile(path.join(root, "MODULE.bazel"), "module(name = \"fixture\")\n");
  await mkdir(path.join(root, "inputs"));
  const manifest = path.join(root, "inputs", "package.json");
  const credentials = path.join(root, "inputs", "wifi.json");
  await writeFile(manifest, JSON.stringify({
    source_commit: "a".repeat(40),
    reference_commit: "b".repeat(40),
    app_elf_sha256: "c".repeat(64),
  }));
  await writeFile(credentials, "{}\n");
  return { root, manifest, credentials, projection: path.join(root, "docs", "projection.json") };
}

function options(value: Fixture) {
  return {
    privateRoot: "scratch/attempt",
    packageManifest: value.manifest,
    wifiCredentials: value.credentials,
    port: "/dev/test-sensitive-port",
    projection: value.projection,
    captureTimeoutSeconds: 360,
  } as const;
}

function installThemeApi(configuration: { readonly failRestore?: boolean } = {}) {
  const originalTheme = {
    colorScheme: "private-original-scheme",
    accentColors: { "--private-original-color": "#123456" },
  };
  let currentTheme = structuredClone(originalTheme);
  let postCount = 0;
  const originalFetch = globalThis.fetch;
  globalThis.fetch = async (input, init) => {
    const target = new URL(String(input));
    if (target.pathname === "/api/system/info") {
      return new Response(JSON.stringify({ hostname: "private-hostname" }), { status: 200 });
    }
    if (init?.method === "POST") {
      postCount += 1;
      if (configuration.failRestore === true && postCount >= 2) throw new Error("private restore failure");
      currentTheme = JSON.parse(String(init.body)) as typeof originalTheme;
      return new Response(JSON.stringify({ status: "ok" }), { status: 200 });
    }
    return new Response(JSON.stringify(currentTheme), { status: 200 });
  };
  return {
    originalTheme,
    currentTheme: () => currentTheme,
    restore: () => { globalThis.fetch = originalFetch; },
  };
}

function fakePort(
  sessionProjection: unknown,
  configuration: {
    readonly sessionOutcome?: ProcessOutcome;
    readonly omitProjection?: boolean;
    readonly recoveryOutcome?: ProcessOutcome;
    readonly onCommand?: (command: string) => void;
  } = {},
): ProcessPort {
  return createFakeProcessPort(async (spec) => {
    const command = spec.args[0] ?? "";
    configuration.onCommand?.(command);
    if (command === "flash-monitor") {
      const evidenceRoot = spec.args[spec.args.indexOf("--evidence-dir") + 1];
      assert.notEqual(evidenceRoot, undefined);
      await writeFile(path.join(String(evidenceRoot), "flash-monitor.classifier-input.log"), trace);
      return ok();
    }
    if (command === "verify-settings-durability") {
      return ok(JSON.stringify({ status: "passed", session: "a".repeat(32), boot_ordinal: 4 }));
    }
    if (command === "reboot-live") {
      if (configuration.omitProjection !== true) {
        const output = spec.args[spec.args.indexOf("--projection-output") + 1];
        assert.notEqual(output, undefined);
        await writeFile(String(output), `${JSON.stringify(sessionProjection)}\n`);
      }
      return configuration.sessionOutcome ?? ok();
    }
    if (command === "flash") return configuration.recoveryOutcome ?? ok();
    throw new Error(`unexpected process ${spec.args.join(" ")}`);
  });
}

async function capture(value: Fixture, processPort: ProcessPort): Promise<unknown> {
  return captureThemeDurability(value.root, options(value), processPort, "flash", "classifier", "device-session");
}

async function themeError(promise: Promise<unknown>): Promise<ThemeDurabilityError> {
  try {
    await promise;
    assert.fail("expected theme durability capture to fail");
  } catch (error) {
    assert.ok(error instanceof ThemeDurabilityError);
    return error;
  }
}

test("ready device session restores the exact theme and emits closed evidence", async () => {
  // Arrange
  const value = await fixture("ready");
  const api = installThemeApi();
  const commands: string[] = [];

  try {
    // Act
    await capture(value, fakePort(readyProjection, { onCommand: (command) => commands.push(command) }));

    // Assert
    const publicDocument = await readFile(value.projection, "utf8");
    const publicValue = JSON.parse(publicDocument) as Record<string, unknown>;
    assert.deepEqual(api.currentTheme(), api.originalTheme);
    assert.equal(publicValue["schema_version"], "bitaxe-theme-durability-evidence-v1");
    assert.deepEqual(publicValue["restart_session"], readyProjection);
    assert.equal(publicValue["restoration_complete"], true);
    assert.doesNotMatch(publicDocument, /private|bitaxe-parity|test-sensitive-port|127\.0\.0\.1|#0b5fff/u);
    assert.deepEqual(commands, ["flash-monitor", "verify-settings-durability", "reboot-live"]);
    const privateRoot = path.join(value.root, "scratch", "attempt");
    assert.equal((await stat(privateRoot)).mode & 0o777, 0o700);
    assert.equal((await stat(path.join(privateRoot, "device-session-intent.private.json"))).mode & 0o777, 0o600);
  } finally {
    api.restore();
  }
});

test("every non-ready device-session category blocks evidence and restores the theme", async () => {
  const categories = [
    "incomplete", "observer_unqualified", "restart_request_not_sent",
    "restart_attribution_ambiguous", "usb_identity_unavailable", "usb_identity_drift",
    "service_recovery_timeout", "boot_identity_invalid", "build_identity_mismatch",
    "session_not_advanced", "reset_reason_wrong", "ordinal_not_next", "postcondition_mismatch",
  ];
  for (const terminalCategory of categories) {
    // Arrange
    const value = await fixture(terminalCategory);
    const api = installThemeApi();
    try {
      // Act
      const error = await themeError(capture(value, fakePort({
        ...readyProjection,
        terminal_category: terminalCategory,
      }, { sessionOutcome: failed() })));

      // Assert
      assert.equal(error.category, "hardware_blocked");
      assert.equal(error.publicValue["terminal_category"], terminalCategory);
      assert.equal(error.publicValue["restoration_complete"], true);
      assert.deepEqual(api.currentTheme(), api.originalTheme);
      await assert.rejects(readFile(value.projection, "utf8"), { code: "ENOENT" });
    } finally {
      api.restore();
    }
  }
});

test("missing malformed timeout and launch failures keep typed primary categories", async () => {
  for (const testCase of [
    { name: "missing", projection: readyProjection, omit: true, outcome: failed(), category: "evidence_invalid" },
    { name: "malformed", projection: { ...readyProjection, schema_version: "wrong" }, omit: false, outcome: failed(), category: "evidence_invalid" },
    { name: "timeout", projection: readyProjection, omit: true, outcome: { ...failed(), timedOut: true }, category: "timeout" },
  ] as const) {
    // Arrange
    const value = await fixture(testCase.name);
    const api = installThemeApi();
    try {
      // Act
      const error = await themeError(capture(value, fakePort(testCase.projection, {
        omitProjection: testCase.omit,
        sessionOutcome: testCase.outcome,
      })));

      // Assert
      assert.equal(error.category, testCase.category);
      assert.equal(error.publicValue["restoration_complete"], true);
    } finally {
      api.restore();
    }
  }

  const value = await fixture("launch");
  const api = installThemeApi();
  const basePort = fakePort(readyProjection);
  const throwingPort: ProcessPort = {
    ...basePort,
    run: async (spec) => {
      if (spec.args[0] === "reboot-live") throw new Error("private /dev/test-sensitive-port");
      return basePort.run(spec);
    },
  };
  try {
    const error = await themeError(capture(value, throwingPort));
    assert.equal(error.category, "process_failed");
    assert.doesNotMatch(JSON.stringify(error.publicValue), /private|test-sensitive-port/u);
  } finally {
    api.restore();
  }
});

test("failed restoration uses exact-package recovery without replacing the primary failure", async () => {
  // Arrange
  const value = await fixture("recovery");
  const api = installThemeApi({ failRestore: true });
  const commands: string[] = [];

  try {
    // Act
    const error = await themeError(capture(value, fakePort({
      ...readyProjection,
      terminal_category: "usb_identity_drift",
    }, {
      sessionOutcome: failed(),
      recoveryOutcome: failed(),
      onCommand: (command) => commands.push(command),
    })));

    // Assert
    assert.equal(error.category, "hardware_blocked");
    assert.equal(error.publicValue["terminal_category"], "usb_identity_drift");
    assert.equal(error.publicValue["recovery_flash_used"], true);
    assert.equal(error.publicValue["secondary_recovery_failure"], true);
    assert.equal(error.publicValue["restoration_complete"], false);
    assert.equal(commands.at(-1), "flash");
  } finally {
    api.restore();
  }
});

test("real child processes produce only the requested projection artifacts", async () => {
  // Arrange
  const value = await fixture("real-child");
  const api = installThemeApi();
  const child = path.join(value.root, "child.mjs");
  await writeFile(child, `#!${nodeProgram}
import { mkdir, writeFile } from "node:fs/promises";
import path from "node:path";
const args = process.argv.slice(2);
if (args[0] === "flash-monitor") {
  const root = args[args.indexOf("--evidence-dir") + 1];
  await mkdir(root, { recursive: true });
  await writeFile(path.join(root, "flash-monitor.classifier-input.log"), ${JSON.stringify(trace)});
} else if (args[0] === "verify-settings-durability") {
  process.stdout.write(JSON.stringify({ status: "passed", session: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", boot_ordinal: 4 }));
} else if (args[0] === "reboot-live") {
  const output = args[args.indexOf("--projection-output") + 1];
  await writeFile(output, JSON.stringify(${JSON.stringify(readyProjection)}));
} else {
  process.exitCode = 98;
}
`);
  await chmod(child, 0o700);
  const processPort = createLocalProcessPort({ cwd: value.root, timeoutMs: 5_000 });

  try {
    // Act
    await captureThemeDurability(value.root, options(value), processPort, child, child, child);

    // Assert
    const projection = JSON.parse(await readFile(value.projection, "utf8")) as Record<string, unknown>;
    assert.equal(projection["schema_version"], "bitaxe-theme-durability-evidence-v1");
    await assert.rejects(
      readFile(path.join(value.root, "scratch", "attempt", "post-restart", "flash-monitor.log"), "utf8"),
      { code: "ENOENT" },
    );
  } finally {
    api.restore();
  }
});
