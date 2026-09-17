import { chmod, readFile, readdir } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { execFileSync } from "node:child_process";
import { fixture as noiseFixture, ledger, original } from "../str005-noise-serial/test-fixture.mjs";
import { NATIVE_AUDITOR_SOURCES } from "../noise-native-readiness.mjs";
import { sha256 } from "./values.mjs";
import { AMENDMENT_PATH } from "./context-sources.mjs";
import { loadContext, preflight } from "./context.mjs";
const REPO = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const NATIVE = ["firmware/bitaxe/src/noise_serial_runtime.rs", "firmware/bitaxe/src/production_mining_session.rs",
  "firmware/bitaxe/src/production_mining_session/transport.rs", "firmware/bitaxe/src/production_mining_session/transport/borrow.rs"];
const AUDITORS = ["scripts/v2-native-readiness.mjs", ...NATIVE_AUDITOR_SOURCES];

/** Explicit synthetic filesystem/reader/auditor seam; never usable as live evidence. */
export async function contextFixture(t, { scope = "channel", prepare = true } = {}) {
  const base = await noiseFixture(t, { prepare: false });
  const { firmwareRoot, gateRoot } = base.options;
  await base.put(resolve(firmwareRoot, "TASKS.md"), "## Active\n### task-str005-v2-serial-qualification | synthetic qualification\n");
  await base.put(resolve(firmwareRoot, AMENDMENT_PATH), await readFile(resolve(REPO, AMENDMENT_PATH)));
  await base.put(resolve(firmwareRoot, "docs/hardware/str005-v2-serial-qualification.md"), await readFile(resolve(REPO, "docs/hardware/str005-v2-serial-qualification.md")));
  await base.put(resolve(firmwareRoot, "scripts/v2-native-readiness.mjs"), "// Explicit synthetic auditor; never a native proof.\n");
  for (const name of await readdir(resolve(REPO, "scripts/str005-v2-serial"))) if (name.endsWith(".mjs"))
    await base.put(resolve(firmwareRoot, "scripts/str005-v2-serial", name), await readFile(resolve(REPO, "scripts/str005-v2-serial", name)));
  for (const name of ["client", "operator", "observer-build-identity"]) {
    const path = resolve(firmwareRoot, `scripts/str005-v2-serial/${name}.mjs`);
    try { await readFile(path); }
    catch (error) { if (error.code !== "ENOENT") throw error; await base.put(path, "// Explicit synthetic fixture code; no device effects.\n"); }
  }
  await base.put(resolve(gateRoot, "dist/worker-serial-acceptance/worker-serial-acceptance.js"), `${"b".repeat(40)} bwg-worker-stratum-v2-standard/0.1 worker-stratum-v2-status-v1 stratumV2ChannelStart stratumV2Status stratumV2Scope explicit synthetic bundle`);
  const observerPath = resolve(firmwareRoot, "bazel-bin/tools/http-transport/cadence_observer");
  await base.put(observerPath, "explicit synthetic observer binary"); await chmod(observerPath, 0o700);
  await base.put(resolve(firmwareRoot, "tools/http-transport/src/lib.rs"), "// explicit synthetic observer source\n");
  await base.put(resolve(dirname(observerPath), "v2-observer-build-identity.json"), JSON.stringify({ schema: "str005-v2-observer-build-v1",
    sourceCommit: "a".repeat(40), sourceDirty: false, observerSha256: sha256(await readFile(observerPath)),
    writerSha256: sha256(await readFile(resolve(firmwareRoot, "scripts/str005-v2-serial/observer-build-identity.mjs"))) }));
  const fixtureBinary = base.options.fixtureBinary;
  await base.put(resolve(dirname(fixtureBinary), "v2-serial-build-identity.json"), JSON.stringify({ schema: "str005-v2-fixture-build-v1",
    sourceCommit: "a".repeat(40), sourceDirty: false, fixtureSha256: sha256(await readFile(fixtureBinary)),
    writerSha256: sha256(await readFile(resolve(firmwareRoot, "scripts/str005-v2-serial/build-identity.mjs"))) }));
  const parent = resolve(firmwareRoot, "scratch/str005-v2-serial");
  await base.put(resolve(parent, ".synthetic-parent"), "explicit synthetic namespace");
  const previousRoot = resolve(base.base, "accepted-noise");
  await base.put(resolve(previousRoot, "final-result.json"), JSON.stringify({ synthetic: true }));
  const previous = { root: previousRoot, resultSha256: "c".repeat(64), sealSha256: "d".repeat(64), ledger: structuredClone(ledger), original: structuredClone(original),
    context: { schema: "noise-serial-context-v2", firmware_commit: "d".repeat(40), app_elf_sha256: "e".repeat(64), original_campaign_id: Buffer.alloc(16, 4).toString("base64url") } };
  const operations = { ...base.operations, nativeSourceFiles: [...NATIVE], nativeAuditorSources: [...AUDITORS],
    inspectNative: async (input) => ({ schema: "str005-v2-native-readiness-v1", result: "selected_native_checks_passed",
      firmwareCommit: input.expectedSourceCommit, elfSha256: input.expectedElfSha256, hardwareQualified: false,
      sdkconfigSha256: sha256(await readFile(resolve(dirname(input.manifestPath), "bitaxe-firmware.sdkconfig"))),
      sourceFiles: await Promise.all(NATIVE.map(async (path) => ({ path, sha256: sha256(await readFile(resolve(firmwareRoot, path))) }))),
      auditorSources: await Promise.all(AUDITORS.map(async (path) => ({ path, sha256: sha256(await readFile(resolve(firmwareRoot, path))) }))) }),
    inspectPredecessor: async () => previous };
  execFileSync("git", ["-C", firmwareRoot, "add", "."]);
  let predecessorReceipt = resolve(previousRoot, "final-result.json");
  if (scope === "share") {
    const channelOptions = { ...base.options, scope: "channel", privateRoot: resolve(parent, "channel-001"), predecessorReceipt };
    await preflight(channelOptions, operations);
    const channel = await loadContext(channelOptions.privateRoot, { operations });
    const channelPrevious = { ...previous, root: channelOptions.privateRoot, context: channel, resultSha256: "e".repeat(64), sealSha256: "f".repeat(64) };
    operations.inspectPredecessor = async (_path, requestedScope) => requestedScope === "share" ? channelPrevious : previous;
    predecessorReceipt = resolve(channelOptions.privateRoot, "final-result.json");
  }
  const options = { ...base.options, scope, privateRoot: resolve(parent, `${scope}-001`), predecessorReceipt };
  delete options.attemptOrdinal;
  if (prepare) await preflight(options, operations);
  return { ...base, parent, root: options.privateRoot, options, operations, previous,
    context: prepare ? await loadContext(options.privateRoot, { operations }) : null };
}
