import assert from "node:assert/strict";
import { chmod, mkdir, mkdtemp, readFile, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import { internalCommandSpec } from "./contracts.generated.js";
import { parseInvocation } from "./invocation.js";
import { createFakeProcessPort, type ProcessOutcome } from "./process.js";
import { selfTestCampaignFromInvocation } from "./self-test-campaign.js";

const sourceCommit = "a".repeat(40);
const referenceCommit = "c1915b0a63bfabebdb95a515cedfee05146c1d50";
const appElfSha256 = "b".repeat(64);
const planRelative = "docs/parity/work-plans/20260821T180800Z-SELF-001/PLAN.md";
const rootRelative = "scratch/self001-full-lifecycle/attempt-001";
const projectionRelative =
  "docs/parity/evidence/self001-full-lifecycle/self-test-projection.json";

const ok = (stdout = ""): ProcessOutcome => ({
  exitCode: 0,
  stdout,
  stderr: "",
  timedOut: false,
});

async function sourceInput(relative: string): Promise<string> {
  const maybeRunfiles = process.env["RUNFILES_DIR"];
  const candidates = [
    ...(maybeRunfiles === undefined ? [] : [path.join(maybeRunfiles, "_main", relative)]),
    path.join(process.cwd(), relative),
    path.resolve(process.cwd(), "..", "..", relative),
  ];
  for (const candidate of candidates) {
    try {
      return await readFile(candidate, "utf8");
    } catch (error) {
      if ((error as NodeJS.ErrnoException).code !== "ENOENT") throw error;
    }
  }
  throw new Error(`missing source input ${relative}`);
}

async function fixture() {
  const root = await mkdtemp(path.join(os.tmpdir(), "bitaxe-self-test-"));
  await writeFile(path.join(root, "MODULE.bazel"), "module(name = \"fixture\")\n");
  const plan = await sourceInput(planRelative);
  await mkdir(path.dirname(path.join(root, planRelative)), { recursive: true });
  await writeFile(path.join(root, planRelative), plan);
  await writeFile(path.join(root, "TASKS.md"), [
    "### task-parity-self001-full-lifecycle | fixture",
    `Plan: \`${planRelative}\`.`,
    "One exact-package two-phase BOOT-button campaign uses a mode-0700 root.",
    "",
  ].join("\n"));
  await mkdir(path.join(root, "inputs"));
  const manifest = path.join(root, "inputs/package.json");
  const wifi = path.join(root, "wifi-credentials.json");
  const pool = path.join(root, "pool-credentials.json");
  await writeFile(manifest, JSON.stringify({
    source_commit: sourceCommit,
    reference_commit: referenceCommit,
    app_elf_sha256: appElfSha256,
  }));
  await writeFile(wifi, JSON.stringify({ ssid: "private", wifiPass: "private" }), { mode: 0o600 });
  await writeFile(pool, JSON.stringify({
    poolURL: "private.invalid", poolPort: 3333, poolUser: "private", poolPassword: "private",
  }), { mode: 0o600 });
  await chmod(wifi, 0o600);
  await chmod(pool, 0o600);
  const wrapper = path.join(root, "scratch/self001-full-lifecycle/wrapper-001");
  await mkdir(wrapper, { recursive: true, mode: 0o700 });
  await chmod(wrapper, 0o700);
  const detector = path.join(wrapper, "detector.stdout");
  await writeFile(detector, "espflash_version: 4.3.0\nport: /dev/private\nusb_session: ready\n", { mode: 0o600 });
  await chmod(detector, 0o600);
  return { root, manifest, wifi, pool, detector };
}

function invocation(value: Awaited<ReturnType<typeof fixture>>, action: "start" | "resume") {
  return parseInvocation([
    "self-test-campaign",
    "--action", action,
    "--private-root", rootRelative,
    "--package-manifest", value.manifest,
    "--wifi-credentials", value.wifi,
    "--pool-credentials", value.pool,
    "--detector-output", path.relative(value.root, value.detector),
    "--plan", planRelative,
    "--projection", projectionRelative,
  ]);
}

test("start and resume preserve the protected two-phase contract", async () => {
  const value = await fixture();
  let monitorCount = 0;
  const processPort = createFakeProcessPort(async spec => {
    if (spec.program === "git") {
      if (spec.args.includes("status")) return ok();
      if (spec.args.includes("reference/esp-miner")) return ok(`${referenceCommit}\n`);
      return ok(`${sourceCommit}\n`);
    }
    if (spec.program === "validator") return ok();
    if (spec.args[0] === "monitor") {
      monitorCount += 1;
      if (monitorCount === 1) return ok("device_url=http://device.invalid\n");
      const state = JSON.parse(await readFile(
        path.join(value.root, rootRelative, "campaign-state.private.json"),
        "utf8",
      )) as { lease_hex: string };
      return ok(`self_test_receipt outcome=cancelled lease=${state.lease_hex}\ndevice_url=http://device.invalid\n`);
    }
    if (spec.args[0] === "flash-monitor") {
      const evidenceRoot = String(spec.args[spec.args.indexOf("--evidence-dir") + 1]);
      await mkdir(evidenceRoot, { recursive: true, mode: 0o700 });
      await chmod(evidenceRoot, 0o700);
      const pass = spec.args.some(arg => arg.endsWith("pass-intent.private.json"));
      const statePath = path.join(value.root, rootRelative, "campaign-state.private.json");
      const maybeState = pass
        ? JSON.parse(await readFile(statePath, "utf8")) as { lease_hex: string }
        : undefined;
      const log = pass
        ? [
          "psram_status=available",
          "self_test_stage={\"stage\":\"warming\"}",
          "self_test_stage={\"stage\":\"measuring\"}",
          "self_test_stage={\"stage\":\"evaluating\"}",
          "self_test_stage={\"stage\":\"safe_stopping\"}",
          "self_test_stage={\"stage\":\"restarting\"}",
          "self_test_terminal={\"outcome\":\"passed\"}",
          `self_test_receipt outcome=passed lease=${maybeState?.lease_hex ?? "missing"}`,
          "device_url=http://device.invalid",
        ].join("\n")
        : [
          "self_test_stage={\"stage\":\"measuring\"}",
          "self_test_checkpoint={\"checkpoint\":\"cancel_ready\",\"safe_state\":true,\"failure\":\"planned_evaluation_failure\"}",
        ].join("\n");
      await writeFile(path.join(evidenceRoot, "flash-monitor.classifier-input.log"), log, { mode: 0o600 });
      await chmod(path.join(evidenceRoot, "flash-monitor.classifier-input.log"), 0o600);
      return ok();
    }
    return ok();
  });
  const settings = {
    bootSession: "1".repeat(32), startMiningOnBoot: false, hostname: "bitaxe",
    frequency: 485, coreVoltage: 1200, manualFanSpeed: 100,
    ssid: "private", stratumURL: "private.invalid", stratumPort: 3333,
    stratumUser: "private", useFallbackStratum: false, fallbackStratumURL: "",
  };
  const theme = { colorScheme: "dark" };
  const originalFetch = globalThis.fetch;
  globalThis.fetch = async (input, init) => {
    const url = new URL(typeof input === "string" ? input : input.toString());
    if (init?.method === "PATCH" || init?.method === "POST") {
      return new Response("", { status: 200 });
    }
    return Response.json(url.pathname.endsWith("theme") ? theme : settings);
  };
  try {
    // Act
    const started = await selfTestCampaignFromInvocation(
      value.root, invocation(value, "start"), processPort, "flash", "validator",
    );
    const resumed = await selfTestCampaignFromInvocation(
      value.root, invocation(value, "resume"), processPort, "flash", "validator",
    );

    // Assert
    assert.equal(started["checkpoint"], "cancel_ready");
    assert.equal(resumed["status"], "ready");
    assert.equal(
      JSON.parse(await readFile(path.join(value.root, projectionRelative), "utf8"))["schema_version"],
      "bitaxe-self-test-evidence-v1",
    );
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("invocation rejects missing phase and unknown options", () => {
  assert.throws(() => parseInvocation(["self-test-campaign", "--private-root", rootRelative]));
  assert.throws(() => parseInvocation(["self-test-campaign", "--action", "start", "--unknown"]));
  assert.deepEqual(
    internalCommandSpec("self-test", ["--action", "start"], value => value).args,
    ["--action", "start"],
  );
});
