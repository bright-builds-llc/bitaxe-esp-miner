import { NATIVE_AUDITOR_SOURCES } from "../noise-native-readiness.mjs";
import { lstat, mkdir, readFile, realpath } from "node:fs/promises";
import { basename, dirname, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { BUNDLE, PAGE, admitTrust, canonicalDirectory, cleanPushed, fileDigest, git, ignored,
  missing, nonce, packageSnapshot } from "../fixed-usb-qualification/contract.mjs";
import { inspectPredecessor } from "./predecessor.mjs";
import { verifyArtifactSnapshot } from "../fixed-usb-qualification/snapshot.mjs";
import { BASE_CONTRACT_SHA256 } from "./contract-v2.mjs";
import { canonical, check, digest, inventory, privateRoot, proof, protectedPath, readJson, retain, verifyInventory, writeNew } from "./files.mjs";

export const AMENDMENT_SHA256 = "64d086a8955f7715ce59b2e5ac7cf04d7cf3338b8cbd1e086ee762c117dcb121";
export const SCHEMA = "noise-serial-context-v2";
export const BASE_PATH = "docs/hardware/str005-noise-serial-qualification.md";
export const AMENDMENT_PATH = "docs/hardware/str005-noise-parity-scope-amendment.md";
const HERE = dirname(fileURLToPath(import.meta.url));
const SOURCE_DIRS = ["scripts/str005-noise-serial", "scripts/fixed-usb-qualification", "tools/stratum-v2-fixture",
  "crates/bitaxe-stratum", "crates/bitaxe-worker-control", "scripts/host-stalls"];
const SOURCE_FILES = [BASE_PATH, AMENDMENT_PATH, "Cargo.lock", "Cargo.toml", "MODULE.bazel",
  "firmware/bitaxe/bwg/deployment-trust.json", ...NATIVE_AUDITOR_SOURCES,
  "firmware/bitaxe/src/noise_serial_runtime.rs", "firmware/bitaxe/src/production_mining_session.rs", "firmware/bitaxe/src/production_mining_session/transport.rs",
  "firmware/bitaxe/src/production_mining_session/transport/borrow.rs", "tools/automation/src/redaction.ts", "tools/automation/src/noise-serial-redaction.ts"];

async function taskAdmission(root) {
  const text = await readFile(resolve(root, "TASKS.md"), "utf8");
  let active = false;
  const ids = [];
  for (const line of text.split(/\r?\n/u)) {
    if (line.startsWith("## ")) active = line === "## Active";
    if (active && line.startsWith("### ")) ids.push(line.slice(4).split(/\s/u)[0]);
  }
  check(ids.filter((id) => id === "task-str005-noise-auth-205").length === 1, "noise_live_task_inactive");
  const preparations = ["task-str005-noise-runtime-readiness", "task-str005-noise-fixture-evidence"];
  const archive = await readFile(resolve(root, "TASKS.archive.md"), "utf8");
  const bindings = [];
  for (const id of preparations) {
    check(!new RegExp(`^### ${id}(?:\\s|$)`, "mu").test(text), "noise_preparation_incomplete");
    const headers = [...archive.matchAll(/^### (task-[^\s|]+).*$/gmu)];
    const found = headers.filter((match) => match[1] === id);
    check(found.length === 1, "noise_preparation_receipt_missing");
    const position = headers.indexOf(found[0]), end = headers[position + 1]?.index ?? archive.length;
    const block = archive.slice(found[0].index, end).trimEnd();
    check(/^Status: Complete(?:\b|\s|[(:])/mu.test(block) && !/^Status:.*(?:cancelled|superseded)/imu.test(block), "noise_preparation_not_complete");
    bindings.push({ id, sha256: digest(block) });
  }
  return bindings;
}
async function sourceFiles(root) {
  const paths = git(root, ["ls-files", "--", ...SOURCE_DIRS, ...SOURCE_FILES]).split("\n").filter(Boolean).sort();
  check(paths.length > 0 && SOURCE_FILES.every((path) => paths.includes(path)), "noise_evaluator_membership");
  const files = [];
  for (const path of paths) {
    const fullPath = resolve(root, path), stat = await lstat(fullPath);
    check(stat.isFile() && !stat.isSymbolicLink() && await realpath(fullPath) === fullPath, "noise_evaluator_alias");
    const bytes = await readFile(fullPath);
    files.push({ path, sha256: digest(bytes), length: bytes.length });
  }
  return files;
}
async function contracts(root) {
  const base = await fileDigest(resolve(root, BASE_PATH)), amendment = await fileDigest(resolve(root, AMENDMENT_PATH));
  check(base === BASE_CONTRACT_SHA256 && amendment === AMENDMENT_SHA256, "noise_base_contract_changed");
  const text = await readFile(resolve(root, AMENDMENT_PATH), "utf8");
  check(text.includes("Contract ID: `str005-noise-serial-v2`"), "noise_amendment_profile");
  const binding = { base: { path: BASE_PATH, sha256: base }, amendment: { path: AMENDMENT_PATH, sha256: amendment } };
  return { binding, sha256: digest(canonical(binding)) };
}
async function sources(options, operations) {
  const firmwareRoot = await canonicalDirectory(options.firmwareRoot), gateRoot = await canonicalDirectory(options.gateRoot);
  const source = (operations.git ?? git)(firmwareRoot, ["rev-parse", "HEAD"]), gate = (operations.git ?? git)(gateRoot, ["rev-parse", "HEAD"]);
  (operations.cleanPushed ?? cleanPushed)(firmwareRoot, source); (operations.cleanPushed ?? cleanPushed)(gateRoot, gate);
  const preparationRecords = await (operations.taskAdmission ?? taskAdmission)(firmwareRoot);
  const module = await readFile(resolve(firmwareRoot, "MODULE.bazel"), "utf8");
  const pins = [...module.matchAll(/strip_prefix\s*=\s*"bitaxe-turnstile-system-([a-f0-9]{40})"/gu)];
  check(pins.length === 1 && pins[0][1] === gate, "noise_gate_pin_mismatch");
  const manifest = resolve(options.manifest);
  const packaged = await packageSnapshot(firmwareRoot, manifest, source);
  const fixture = resolve(options.fixtureBinary);
  check(fixture === resolve(firmwareRoot, "bazel-bin/tools/stratum-v2-fixture/stratum_v2_fixture"), "noise_fixture_path");
  const fixtureHash = await fileDigest(fixture), buildReceiptPath = resolve(dirname(fixture), "noise-serial-build-identity.json");
  const built = await readJson(buildReceiptPath);
  check(built.schema === "noise-serial-fixture-build-v2" && built.sourceCommit === source && built.sourceDirty === false &&
    built.fixtureSha256 === fixtureHash && built.writerSha256 === await fileDigest(resolve(firmwareRoot, "scripts/str005-noise-serial/build-identity.mjs")), "noise_fixture_build_identity");
  const bundle = await readFile(resolve(gateRoot, BUNDLE));
  check(bundle.includes(gate) && bundle.includes("worker-noise-diagnostic-start-v2") && bundle.includes("noiseDiagnosticPossession"), "noise_gate_capability");
  const trust = await readJson(resolve(firmwareRoot, "firmware/bitaxe/bwg/deployment-trust.json")); admitTrust(trust, trust);
  return { ...packaged, firmware_root: firmwareRoot, gate_root: gateRoot, manifest, fixture_binary: fixture,
    firmware_commit: source, gate_commit: gate, gate_bundle_sha256: digest(bundle), gate_page_relative_path: PAGE,
    gate_page_sha256: await fileDigest(resolve(gateRoot, PAGE)), trust_sha256: digest(JSON.stringify(trust)),
    sdkconfig_sha256: await fileDigest(resolve(dirname(manifest), "bitaxe-firmware.sdkconfig")),
    preparation_records: preparationRecords, fixture_build_receipt_sha256: await fileDigest(buildReceiptPath),
    fixture_sha256: fixtureHash, contracts: await contracts(firmwareRoot), evaluator: await sourceFiles(firmwareRoot) };
}
async function copySnapshot(root, context) {
  const files = [], manifest = await readJson(context.manifest);
  async function add(path, source) {
    const bytes = await readFile(source); await retain(resolve(root, "qualified-artifacts", path), bytes);
    files.push({ path, sha256: digest(bytes), length: bytes.length });
  }
  await add("firmware/bitaxe-ultra205-package.json", context.manifest);
  for (const item of manifest.artifacts) await add(`firmware/${item.path}`,
    resolve(item.kind === "partition_table" ? context.firmware_root : dirname(context.manifest), item.path));
  for (const name of ["license-inventory", "provenance-manifest"]) await add(`firmware/docs/release/${name}.md`, resolve(context.firmware_root, `docs/release/${name}.md`));
  for (const name of [BUNDLE, PAGE]) await add(`gate/${name}`, resolve(context.gate_root, name));
  await writeNew(resolve(root, "artifact-snapshot.json"), { schema: "fixed-usb-qualified-artifacts-v1", context_sha256: digest(JSON.stringify(context)), files });
  await retain(resolve(root, "fixture/fixture.bin"), await readFile(context.fixture_binary));
  await retain(resolve(root, "fixture/build-identity.json"), await readFile(resolve(dirname(context.fixture_binary), "noise-serial-build-identity.json")));
  for (const item of context.evaluator) {
    const bytes = await readFile(resolve(context.firmware_root, item.path)); check(digest(bytes) === item.sha256, "noise_evaluator_changed");
    await retain(resolve(root, "evaluator", item.path), bytes);
  }
  await verifyArtifactSnapshot(root, context);
}

/** No device/network effects; a failed creation never releases its ordinal reservation. */
export async function preflight(options, operations = {}) {
  check(options.authorityDirectory === undefined && options.poolCredentials === undefined, "noise_credentials_forbidden");
  const root = resolve(options.privateRoot), parent = await privateRoot(dirname(root));
  await missing(root);
  const ordinal = Number(options.attemptOrdinal);
  check(Number.isSafeInteger(ordinal) && ordinal > 0 && basename(root) === `attempt-${String(ordinal).padStart(3, "0")}`, "noise_attempt_name");
  const source = await sources(options, operations);
  check(parent === resolve(source.firmware_root, "scratch/str005-noise-serial"), "noise_namespace");
  (operations.ignored ?? ignored)(source.firmware_root, root);
  const predecessorPath = resolve(options.predecessorReceipt);
  const predecessor = await (operations.inspectPredecessor ?? inspectPredecessor)(predecessorPath), previous = predecessor.previous;
  check(previous.result === "passed" && previous.cleanup_confirmed && previous.next_ordinal === 18 && previous.total_charged_ms === 1560000 &&
    previous.context?.firmware_commit && previous.context?.app_elf_sha256, "noise_predecessor");
  // V2's first positive run is implemented; a later attempt needs reviewed progress.
  check(ordinal === 1, "noise_retry_progress_unverified");
  const inspectNative = operations.inspectNative ?? (await import("../noise-native-readiness.mjs")).inspectNoiseNativeReadiness;
  const native = await inspectNative({ firmwareRoot: source.firmware_root, manifestPath: source.manifest,
    expectedSourceCommit: source.firmware_commit, expectedElfSha256: source.app_elf_sha256 });
  check(native.schema === "noise-serial-native-readiness-v1" && native.result === "selected_native_checks_passed" &&
    native.firmwareCommit === source.firmware_commit && native.elfSha256 === source.app_elf_sha256 &&
    native.sdkconfigSha256 === source.sdkconfig_sha256 && native.hardwareQualified === false, "noise_native_readiness");
  const nativeSources = {
    noiseOwnerSourceSha256: "firmware/bitaxe/src/noise_serial_runtime.rs",
    productionOwnerSourceSha256: "firmware/bitaxe/src/production_mining_session.rs",
    transportOwnerSourceSha256: "firmware/bitaxe/src/production_mining_session/transport.rs",
    borrowSourceSha256: "firmware/bitaxe/src/production_mining_session/transport/borrow.rs",
  };
  for (const [key, path] of Object.entries(nativeSources)) check(native[key] === source.evaluator.find((item) => item.path === path)?.sha256,
    "noise_native_source_join");
  check(Array.isArray(native.auditorSources) && native.auditorSources.length === NATIVE_AUDITOR_SOURCES.length &&
    NATIVE_AUDITOR_SOURCES.every((path) => {
      const matches = native.auditorSources.filter((item) => item.path === path);
      return matches.length === 1 && matches[0].sha256 === source.evaluator.find((item) => item.path === path)?.sha256;
    }), "noise_native_auditor_join");
  const context = { schema: SCHEMA, contract_id: "str005-noise-serial-v2", ...source,
    ordinal, attempt_id: nonce(), predecessor: { path: predecessorPath, sha256: await fileDigest(predecessorPath), inventorySha256: predecessor.inventorySha256 },
    before_source: { firmware_commit: previous.context.firmware_commit, app_elf_sha256: previous.context.app_elf_sha256 },
    original_campaign_id: previous.original_campaign_id, expected_ledger: { next_ordinal: 18, last_ordinal: 17, total_charged_ms: 1560000 },
    native_readiness: native, client_sha256: await fileDigest(resolve(HERE, "client.mjs")), mining_authorized: false, maximum_installations: 5 };
  await writeNew(resolve(parent, `ordinal-${ordinal}.json`), { schema: "noise-serial-assignment-v2", root, context_sha256: digest(JSON.stringify(context)) });
  await (operations.beforeCreate ?? (() => {}))();
  await mkdir(root, { mode: 0o700 });
  await writeNew(resolve(root, "context.json"), { context, sha256: digest(JSON.stringify(context)) });
  await copySnapshot(root, context);
  await retain(resolve(root, "native/bitaxe-firmware.sdkconfig"), await readFile(resolve(dirname(context.manifest), "bitaxe-firmware.sdkconfig")));
  await writeNew(resolve(root, "native/readiness.json"), native);
  await writeNew(resolve(root, "preflight-inventory.json"), { schema: "noise-serial-preflight-inventory-v2", files: await inventory(root) });
  return { ready: true, context_sha256: digest(JSON.stringify(context)), device_effects: false, hardware_qualified: false };
}
export async function loadContext(root, { historical = false, operations = {} } = {}) {
  root = await privateRoot(root);
  const record = await proof(root, "context.json"), context = record.value.context;
  check(context?.schema === SCHEMA && record.value.sha256 === digest(JSON.stringify(context)) && context.contract_id === "str005-noise-serial-v2" &&
    context.mining_authorized === false && context.maximum_installations === 5, "noise_context_integrity");
  const assignment = await proof(dirname(root), `ordinal-${context.ordinal}.json`);
  check(assignment.value.root === root && assignment.value.context_sha256 === record.value.sha256, "noise_assignment_changed");
  await verifyArtifactSnapshot(root, context);
  check(canonical((await proof(root, "native/readiness.json")).value) === canonical(context.native_readiness) &&
    (await proof(root, "fixture/build-identity.json")).sha256 === context.fixture_build_receipt_sha256 &&
    await fileDigest(resolve(root, "native/bitaxe-firmware.sdkconfig")) === context.sdkconfig_sha256, "noise_native_snapshot_changed");
  check(await fileDigest(resolve(root, "fixture/fixture.bin")) === context.fixture_sha256, "noise_fixture_changed");
  for (const entry of context.evaluator) {
    const item = await proofBytes(root, `evaluator/${entry.path}`);
    check(item.length === entry.length && digest(item) === entry.sha256, "noise_evaluator_changed");
  }
  check(await fileDigest(context.predecessor.path) === context.predecessor.sha256, "noise_predecessor_changed");
  const parent = await (operations.inspectPredecessor ?? inspectPredecessor)(context.predecessor.path);
  check(parent.inventorySha256 === context.predecessor.inventorySha256, "noise_predecessor_seal_changed");
  if (!historical) {
    await missing(resolve(root, "sealed-inventory.json")); await missing(resolve(root, "final-result.json"));
    const observed = await sources({ firmwareRoot: context.firmware_root, gateRoot: context.gate_root, manifest: context.manifest, fixtureBinary: context.fixture_binary }, operations);
    for (const key of Object.keys(observed)) check(JSON.stringify(context[key]) === JSON.stringify(observed[key]), "noise_live_source_drift");
    check(await fileDigest(resolve(HERE, "client.mjs")) === context.client_sha256, "noise_client_changed");
  }
  return context;
}
async function proofBytes(root, name) {
  const path = resolve(root, name); check(relative(root, path).startsWith("evaluator/"), "noise_evaluator_path");
  check(await realpath(path) === path, "noise_evaluator_alias"); await protectedPath(path);
  return readFile(path);
}

/** Fast effect gate after full serve admission: no historical traversal or disassembly. */
export async function verifyEffectInputs(context, operations = {}) {
  (operations.cleanPushed ?? cleanPushed)(context.firmware_root, context.firmware_commit);
  (operations.cleanPushed ?? cleanPushed)(context.gate_root, context.gate_commit);
  await (operations.taskAdmission ?? taskAdmission)(context.firmware_root);
  const packaged = await packageSnapshot(context.firmware_root, context.manifest, context.firmware_commit);
  for (const [key, value] of Object.entries(packaged)) check(JSON.stringify(value) === JSON.stringify(context[key]), "noise_package_changed");
  for (const [path, expected] of [[context.fixture_binary, context.fixture_sha256],
    [resolve(dirname(context.fixture_binary), "noise-serial-build-identity.json"), context.fixture_build_receipt_sha256],
    [resolve(context.gate_root, BUNDLE), context.gate_bundle_sha256], [resolve(context.gate_root, PAGE), context.gate_page_sha256],
    [resolve(dirname(context.manifest), "bitaxe-firmware.sdkconfig"), context.sdkconfig_sha256]])
    check(await fileDigest(path) === expected, "noise_effect_input_changed");
}

/** Recompute native measurements outside the five-second endpoint/Start path. */
export async function recheckNative(context, operations = {}) {
  const inspect = operations.inspectNative ?? (await import("../noise-native-readiness.mjs")).inspectNoiseNativeReadiness;
  const value = await inspect({ firmwareRoot: context.firmware_root, manifestPath: context.manifest,
    expectedSourceCommit: context.firmware_commit, expectedElfSha256: context.app_elf_sha256 });
  check(canonical(value) === canonical(context.native_readiness), "noise_native_recheck_changed");
  return value;
}
