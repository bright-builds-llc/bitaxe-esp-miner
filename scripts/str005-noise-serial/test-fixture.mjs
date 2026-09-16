import { NATIVE_AUDITOR_SOURCES } from "../noise-native-readiness.mjs";
import { generateKeyPairSync } from "node:crypto";
import { execFileSync } from "node:child_process";
import { chmod, mkdir, mkdtemp, readFile, readdir, realpath, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { BUNDLE, PAGE, digest, writeNew } from "../fixed-usb-qualification/contract.mjs";
import { preflight, loadContext, BASE_PATH, AMENDMENT_PATH } from "./context.mjs";
const REPO = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
export const SOURCE = "a".repeat(40), GATE = "b".repeat(40);
const BASELINE = Buffer.alloc(16, 4).toString("base64url");
export async function fixture(t, { prepare = true } = {}) {
  const base = await realpath(await mkdtemp(resolve(tmpdir(), "noise-v2-")));
  t.after(() => rm(base, { recursive: true, force: true }));
  const firmwareRoot = resolve(base, "firmware"), gateRoot = resolve(base, "gate"), parent = resolve(firmwareRoot, "scratch/str005-noise-serial"), root = resolve(parent, "attempt-001");
  const put = async (path, content) => { await mkdir(dirname(path), { recursive: true, mode: 0o700 }); await writeFile(path, content, { mode: 0o600 }); };
  await mkdir(parent, { recursive: true, mode: 0o700 });
  await put(resolve(firmwareRoot, "TASKS.md"), "## Active\n### task-str005-noise-auth-205 | synthetic live task\n");
  await put(resolve(firmwareRoot, "TASKS.archive.md"), "### task-str005-noise-runtime-readiness | synthetic\nStatus: Complete\n\n### task-str005-noise-fixture-evidence | synthetic\nStatus: Complete\n");
  for (const path of [BASE_PATH, AMENDMENT_PATH, "Cargo.lock", "Cargo.toml", "MODULE.bazel", ...NATIVE_AUDITOR_SOURCES, "firmware/bitaxe/src/noise_serial_runtime.rs", "firmware/bitaxe/src/production_mining_session.rs", "firmware/bitaxe/src/production_mining_session/transport.rs", "firmware/bitaxe/src/production_mining_session/transport/borrow.rs", "tools/automation/src/redaction.ts", "tools/automation/src/noise-serial-redaction.ts"])
    await put(resolve(firmwareRoot, path), await readFile(resolve(REPO, path)));
  await put(resolve(firmwareRoot, "MODULE.bazel"), `# synthetic test pin\nstrip_prefix = "bitaxe-turnstile-system-${GATE}"\n`);
  for (const name of await readdir(resolve(REPO, "scripts/str005-noise-serial"))) if (name.endsWith(".mjs"))
    await put(resolve(firmwareRoot, "scripts/str005-noise-serial", name), await readFile(resolve(REPO, "scripts/str005-noise-serial", name)));
  const key = generateKeyPairSync("ed25519").publicKey.export({ format: "jwk" });
  const keys = [{ kid: "synthetic", kty: "OKP", crv: "Ed25519", x: key.x, alg: "Ed25519", use: "sig", key_ops: ["verify"] }];
  const trust = { profile: "bwg-worker-deployment-trust/0.2", updateAuthority: { issuer: "fixture", audience: "fixture", keys }, workLeaseAuthority: { issuer: "fixture", audience: "fixture", keys } };
  await put(resolve(firmwareRoot, "firmware/bitaxe/bwg/deployment-trust.json"), JSON.stringify(trust));
  await put(resolve(gateRoot, BUNDLE), `${GATE} worker-noise-diagnostic-start-v2 noiseDiagnosticPossession synthetic bundle`);
  await put(resolve(gateRoot, PAGE), '<!doctype html><body><pre id="state"></pre></body>');
  const manifest = resolve(firmwareRoot, "bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json"), artifacts = [];
  for (const kind of ["firmware_elf", "firmware_ota_image", "www_spiffs_image", "factory_merged_image", "partition_table", "otadata_initial", "bootloader", "partition_table_binary"]) {
    const path = kind === "partition_table" ? "firmware/bitaxe/partitions-ultra205.csv" : `${kind}.bin`, bytes = Buffer.from(`synthetic-${kind}`);
    await put(resolve(kind === "partition_table" ? firmwareRoot : dirname(manifest), path), bytes);
    artifacts.push({ kind, path, sha256: digest(bytes) });
  }
  const update_segments = [["bootloader", 0], ["partition_table_binary", 0x8000], ["firmware_ota_image", 0x10000], ["www_spiffs_image", 0x410000], ["otadata_initial", 0xf10000]]
    .map(([artifact_kind, offset]) => ({ artifact_kind, offset, length: Buffer.byteLength(`synthetic-${artifact_kind}`) }));
  await put(manifest, JSON.stringify({ schema_version: 4, build_identity: { source_dirty: false }, source_commit: SOURCE,
    reference_commit: "c".repeat(40), app_elf_sha256: artifacts[0].sha256, artifacts, update_segments }));
  await put(resolve(dirname(manifest), "bitaxe-firmware.sdkconfig"), "synthetic sdkconfig");
  for (const name of ["license-inventory", "provenance-manifest"]) await put(resolve(firmwareRoot, `docs/release/${name}.md`), "synthetic public metadata");
  const fixtureBinary = resolve(firmwareRoot, "bazel-bin/tools/stratum-v2-fixture/stratum_v2_fixture"); await put(fixtureBinary, "synthetic executable fixture");
  await put(resolve(dirname(fixtureBinary), "noise-serial-build-identity.json"), JSON.stringify({ schema: "noise-serial-fixture-build-v2", sourceCommit: SOURCE,
    sourceDirty: false, fixtureSha256: digest("synthetic executable fixture"), writerSha256: digest(await readFile(resolve(REPO, "scripts/str005-noise-serial/build-identity.mjs"))) }));
  const previousPath = resolve(base, "previous.json"); await put(previousPath, JSON.stringify({ synthetic: true }));
  execFileSync("git", ["init", "-q", firmwareRoot]); execFileSync("git", ["-C", firmwareRoot, "add", "."]);
  const previous = { result: "passed", cleanup_confirmed: true, next_ordinal: 18, total_charged_ms: 1560000,
    context: { firmware_commit: "d".repeat(40), app_elf_sha256: "e".repeat(64) }, original_campaign_id: BASELINE };
  const operations = { cleanPushed() {}, ignored() {}, git(path) { return path === firmwareRoot ? SOURCE : GATE; },
    inspectPredecessor: async () => ({ previous, inventorySha256: "f".repeat(64) }),
    inspectNative: async (input) => ({ schema: "noise-serial-native-readiness-v1", result: "selected_native_checks_passed", firmwareCommit: input.expectedSourceCommit,
      elfSha256: input.expectedElfSha256, sdkconfigSha256: digest("synthetic sdkconfig"), hardwareQualified: false,
      ...Object.fromEntries(await Promise.all(Object.entries({ noiseOwnerSourceSha256: "firmware/bitaxe/src/noise_serial_runtime.rs",
        productionOwnerSourceSha256: "firmware/bitaxe/src/production_mining_session.rs", transportOwnerSourceSha256: "firmware/bitaxe/src/production_mining_session/transport.rs",
        borrowSourceSha256: "firmware/bitaxe/src/production_mining_session/transport/borrow.rs" }).map(async ([key, path]) => [key, digest(await readFile(resolve(firmwareRoot, path)))]))),
      auditorSources: await Promise.all(NATIVE_AUDITOR_SOURCES.map(async (path) => ({ path, sha256: digest(await readFile(resolve(firmwareRoot, path))) }))) } ),
    processSnapshot: async () => [],
    execFileSync() { throw Object.assign(new Error("synthetic absence"), { status: 1, signal: null, stdout: "", stderr: "" }); } };
  const options = { firmwareRoot, gateRoot, privateRoot: root, manifest, fixtureBinary, attemptOrdinal: 1, predecessorReceipt: previousPath };
  if (prepare) await preflight(options, operations);
  return { base, root, parent, options, operations, context: prepare ? await loadContext(root, { operations }) : null, put };
}
export function state(context, phase = "candidate", closed = false) {
  const source = phase === "before" ? context.before_source : context;
  return { schema: "worker-serial-acceptance-v1", gateCommit: context.gate_commit, expectedFirmwareSourceCommit: source.firmware_commit,
    expectedAppElfSha256: source.app_elf_sha256, status: closed ? "closed" : "ready", connected: !closed, running: false,
    heartbeatSuppressed: false, renewalsConfirmed: 0, deviceRestorationConfirmed: true, deviceBaselineConfirmed: true, deviceLeaseInactive: true,
    serialOwnershipReleased: closed, preservation: { schema: "worker-preservation-continuity-v1", baseline_id: BASELINE,
      settings_match: true, authorization_high_water_match: true, device_identity_match: true, mine_on_boot: false } };
}
export const ledger = { schema: "worker-qualification-ledger-v1", next_ordinal: 18, last_completed_ordinal: 17, total_charged_ms: 1560000, pending: false };
export const original = { schema: "worker-budget-review-v1", campaign_match: true, reserved_mask: 7, completed_mask: 7, charged_ms: 240000, pending: false };
