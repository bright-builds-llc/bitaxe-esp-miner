import { NATIVE_AUDITOR_SOURCES } from "../noise-native-readiness.mjs";
import { lstat, mkdir, readFile, realpath } from "node:fs/promises";
import { basename, dirname, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { BUNDLE, PAGE, admitTrust, canonicalDirectory, cleanPushed, fileDigest, git, ignored,
  missing, nonce, packageSnapshot } from "../fixed-usb-qualification/contract.mjs";
import { inspectHelperPassPredecessor, inspectPredecessor, inspectRecoveryPredecessor } from "./predecessor.mjs";
import { verifyArtifactSnapshot } from "../fixed-usb-qualification/snapshot.mjs";
import { BASE_CONTRACT_SHA256 } from "./contract-v2.mjs";
import { canonical, check, digest, inventory, privateRoot, proof, protectedPath, readJson, retain, verifyInventory, writeNew } from "./files.mjs";

export const AMENDMENT_SHA256 = "64d086a8955f7715ce59b2e5ac7cf04d7cf3338b8cbd1e086ee762c117dcb121";
export const SCHEMA = "noise-serial-context-v2";
export const BASE_PATH = "docs/hardware/str005-noise-serial-qualification.md";
export const AMENDMENT_PATH = "docs/hardware/str005-noise-parity-scope-amendment.md";
export const SUCCESSOR_PATH = "docs/hardware/device-noise-helper-amendment.md";
export const SUCCESSOR_SHA256 = "5fa5de42c44811dbb8b374446cbcc6d9e89358e1f752b775f4da22238367facf";
export const CONTROL_SUCCESSOR_PATH = "docs/hardware/control-stack-port-reuse-amendment.md";
export const CONTROL_SUCCESSOR_SHA256 = "2730e27edb7f547aa467634386919da79d3a57d5bfaabebc131ce7a18e90f66b";
const PUBLICATION = "docs/parity/evidence/str005-noise-serial";
/** Each profile owns one namespace, task gate, predecessor basis and expected ledger. */
export const PROFILES = Object.freeze({
  historical: { namespace: "scratch/str005-noise-serial", task: "task-str005-noise-auth-205", enabledLine: null, successor: null,
    inspect: inspectPredecessor, ledger: { next_ordinal: 18, last_ordinal: 17, total_charged_ms: 1560000 },
    admits: (previous) => previous.result === "passed" && previous.cleanup_confirmed === true },
  "device-noise-helper": { namespace: "scratch/device-noise-worker-stack", task: "task-device-noise-worker-stack",
    enabledLine: "Device noise serial hardware: enabled.", successor: { path: SUCCESSOR_PATH, sha256: SUCCESSOR_SHA256 },
    inspect: inspectRecoveryPredecessor, ledger: { next_ordinal: 22, last_ordinal: 21, total_charged_ms: 2280000 },
    // Reviewed continuations: each binds the exact sealed prior attempt and its remediation.
    continuations: Object.freeze({
      2: { attempt: "attempt-001", resultSha256: "48ba50e976d68c8103e6d29016b2f42a2c543199cdcbc90051d393536e171568",
        inventorySha256: "d63eaaa471f5a27e25f1dfb0c54b4c0c409a6fd5e512066882725a4a0e17878e",
        remediation: "owner_operates_native_port_chooser_with_visible_tab" },
      // Attempt-002 installed its candidate, then failed review on launcher-umask
      // evidence modes before sealing; its install evidence fixes the before identity.
      3: { attempt: "attempt-002", resultSha256: "5df645bf9f3c321cfea0682b53451193946715bf1981b5e277f59502f310026a",
        inventorySha256: null, remediation: "operator_admission_restricts_umask",
        installed: { firmware_commit: "9301a2761dfcd59a20212c5bdd084f794155b62f",
          app_elf_sha256: "c7d6d8315754348c8a2c53d4dda540846c477440befa49b213e8cea7760e4a65",
          evidence: { "install-0.claim.json": "4526051b939b583461360bf98c14030877e7d8b712e29c84b89d618a1fddab5c",
            "install-0.exit.json": "5cda8c1ea18893ebb0e483fbb68b6cc39f3fe9e65a53db18e86396dfbea8f00b",
            "install-0/flash-command-evidence.json": "679c3d7f39af4116cdd0492e612925e90cc363b7921848fe68d028343d660ee6" } } },
    }),
    admits: (previous) => previous.basis === "current_safe_recovery" && previous.cleanup_confirmed === true &&
      previous.last_ordinal === 21 },
  "control-stack-port-reuse": { namespace: "scratch/control-stack-port-reuse", task: "task-control-stack-port-reuse-run",
    enabledLine: "Control stack port reuse hardware: enabled.", successor: { path: CONTROL_SUCCESSOR_PATH, sha256: CONTROL_SUCCESSOR_SHA256 },
    publication: "docs/parity/evidence/control-stack-port-reuse",
    inspect: inspectHelperPassPredecessor, ledger: { next_ordinal: 22, last_ordinal: 21, total_charged_ms: 2280000 },
    admits: (previous) => previous.basis === "device_noise_helper_pass" && previous.cleanup_confirmed === true &&
      previous.last_ordinal === 21 },
});
/** Public projection path; ordinals restart per profile, so each profile owns its directory. */
export function publicationPath(context) {
  return resolve(context.firmware_root, profileOf(context).publication ?? PUBLICATION, `attempt-${String(context.ordinal).padStart(3, "0")}.json`);
}
/** A later ordinal needs the exact unverified prior attempt. A sealed attempt must show no
 * device write; an installed one binds its exact install evidence as the before identity. */
export async function inspectContinuation(parent, ordinal, continuation) {
  check(continuation, "noise_retry_progress_unverified");
  await proof(parent, `ordinal-${ordinal - 1}.json`);
  const prior = resolve(parent, continuation.attempt);
  const result = await proof(prior, "final-result.json");
  check(result.sha256 === continuation.resultSha256 && result.value.schema === "noise-serial-result-v2" &&
    result.value.status === "unverified", "noise_continuation_predecessor");
  const binding = { attempt: continuation.attempt, result_sha256: continuation.resultSha256,
    inventory_sha256: continuation.inventorySha256, remediation: continuation.remediation };
  if (!continuation.installed) {
    const seal = await proof(prior, "sealed-inventory.json");
    check(seal.sha256 === continuation.inventorySha256 && seal.value.schema === "noise-serial-seal-v2" &&
      Array.isArray(seal.value.files), "noise_continuation_predecessor");
    await verifyInventory(prior, seal.value.files, new Set(["sealed-inventory.json"]));
    check(seal.value.files.every((item) => !/^install-/u.test(item.path)), "noise_continuation_device_effect");
    return binding;
  }
  const installed = continuation.installed, values = {};
  for (const [name, sha256] of Object.entries(installed.evidence)) {
    const item = await readFile(resolve(prior, name));
    check(digest(item) === sha256, "noise_continuation_install_evidence");
    values[name] = JSON.parse(item.toString("utf8"));
  }
  const flash = values["install-0/flash-command-evidence.json"], context = (await proof(prior, "context.json")).value.context;
  check(values["install-0.exit.json"].code === 0 && flash.flash_status === "completed" && flash.trusted_output === true &&
    flash.observed_firmware_commit === installed.firmware_commit && flash.fixed_serial_assessment?.startup_complete === true &&
    flash.fixed_serial_assessment?.safe_baseline_confirmed === true && context.firmware_commit === installed.firmware_commit &&
    context.app_elf_sha256 === installed.app_elf_sha256, "noise_continuation_install_evidence");
  return { ...binding, before_source: { firmware_commit: installed.firmware_commit, app_elf_sha256: installed.app_elf_sha256 } };
}
export function profileOf(context) {
  const name = context.profile ?? "historical";
  check(Object.hasOwn(PROFILES, name) && (context.profile === undefined) === (name === "historical"), "noise_profile");
  return { name, ...PROFILES[name] };
}
const HERE = dirname(fileURLToPath(import.meta.url));
const SOURCE_DIRS = ["scripts/str005-noise-serial", "scripts/fixed-usb-qualification", "tools/stratum-v2-fixture",
  "crates/bitaxe-stratum", "crates/bitaxe-worker-control", "scripts/host-stalls"];
const SOURCE_FILES = [BASE_PATH, AMENDMENT_PATH, SUCCESSOR_PATH, CONTROL_SUCCESSOR_PATH, "Cargo.lock", "Cargo.toml", "MODULE.bazel",
  "firmware/bitaxe/bwg/deployment-trust.json", ...NATIVE_AUDITOR_SOURCES,
  "firmware/bitaxe/src/noise_serial_runtime.rs", "firmware/bitaxe/src/noise_completion_stack.rs", "firmware/bitaxe/src/production_mining_session.rs", "firmware/bitaxe/src/production_mining_session/transport.rs",
  "firmware/bitaxe/src/production_mining_session/transport/borrow.rs", "tools/automation/src/redaction.ts", "tools/automation/src/noise-serial-redaction.ts"];

async function taskAdmission(root, profile = PROFILES.historical) {
  const text = await readFile(resolve(root, "TASKS.md"), "utf8");
  let active = false, current = null;
  const ids = [], blocks = new Map();
  for (const line of text.split(/\r?\n/u)) {
    if (line.startsWith("## ")) { active = line === "## Active"; current = null; }
    if (active && line.startsWith("### ")) { current = line.slice(4).split(/\s/u)[0]; ids.push(current); blocks.set(current, []); }
    else if (current) blocks.get(current).push(line);
  }
  check(ids.filter((id) => id === profile.task).length === 1, "noise_live_task_inactive");
  check(profile.enabledLine === null || blocks.get(profile.task).filter((line) => line === profile.enabledLine).length === 1,
    "noise_live_task_disabled");
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
async function contracts(root, profile = PROFILES.historical) {
  const base = await fileDigest(resolve(root, BASE_PATH)), amendment = await fileDigest(resolve(root, AMENDMENT_PATH));
  check(base === BASE_CONTRACT_SHA256 && amendment === AMENDMENT_SHA256, "noise_base_contract_changed");
  const text = await readFile(resolve(root, AMENDMENT_PATH), "utf8");
  check(text.includes("Contract ID: `str005-noise-serial-v2`"), "noise_amendment_profile");
  const binding = { base: { path: BASE_PATH, sha256: base }, amendment: { path: AMENDMENT_PATH, sha256: amendment } };
  if (profile.successor) {
    const successor = await fileDigest(resolve(root, profile.successor.path));
    check(successor === profile.successor.sha256, "noise_base_contract_changed");
    check((await readFile(resolve(root, profile.successor.path), "utf8")).includes(`successor profile \`${profile.name}\``), "noise_amendment_profile");
    binding.successor = { path: profile.successor.path, sha256: successor };
  }
  return { binding, sha256: digest(canonical(binding)) };
}
async function sources(options, operations, profile = PROFILES.historical) {
  const firmwareRoot = await canonicalDirectory(options.firmwareRoot), gateRoot = await canonicalDirectory(options.gateRoot);
  const source = (operations.git ?? git)(firmwareRoot, ["rev-parse", "HEAD"]), gate = (operations.git ?? git)(gateRoot, ["rev-parse", "HEAD"]);
  (operations.cleanPushed ?? cleanPushed)(firmwareRoot, source); (operations.cleanPushed ?? cleanPushed)(gateRoot, gate);
  const preparationRecords = await (operations.taskAdmission ?? taskAdmission)(firmwareRoot, profile);
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
    fixture_sha256: fixtureHash, contracts: await contracts(firmwareRoot, profile), evaluator: await sourceFiles(firmwareRoot) };
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
  const firmwareRoot = await canonicalDirectory(options.firmwareRoot);
  const matches = Object.entries(PROFILES).filter(([, value]) => parent === resolve(firmwareRoot, value.namespace));
  check(matches.length === 1, "noise_namespace");
  const profile = { name: matches[0][0], ...matches[0][1] };
  const source = await sources(options, operations, profile);
  (operations.ignored ?? ignored)(source.firmware_root, root);
  const predecessorPath = resolve(options.predecessorReceipt);
  const predecessor = await (operations.inspectPredecessor ?? profile.inspect)(predecessorPath), previous = predecessor.previous;
  check(profile.admits(previous) && previous.next_ordinal === profile.ledger.next_ordinal &&
    previous.total_charged_ms === profile.ledger.total_charged_ms &&
    previous.context?.firmware_commit && previous.context?.app_elf_sha256, "noise_predecessor");
  // V2's first positive run is implemented; a later attempt needs a reviewed continuation.
  const continuation = ordinal === 1 ? null : await inspectContinuation(parent, ordinal, profile.continuations?.[ordinal]);
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
  const context = { schema: SCHEMA, contract_id: "str005-noise-serial-v2", ...(profile.name === "historical" ? {} : { profile: profile.name }), ...source,
    ordinal, attempt_id: nonce(), predecessor: { path: predecessorPath, sha256: await fileDigest(predecessorPath), inventorySha256: predecessor.inventorySha256 },
    before_source: continuation?.before_source ?? { firmware_commit: previous.context.firmware_commit, app_elf_sha256: previous.context.app_elf_sha256 },
    original_campaign_id: previous.original_campaign_id, expected_ledger: { ...profile.ledger },
    native_readiness: native, client_sha256: await fileDigest(resolve(HERE, "client.mjs")), mining_authorized: false, maximum_installations: 5,
    ...(continuation ? { continuation } : {}) };
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
  const profile = profileOf(context);
  const parent = await (operations.inspectPredecessor ?? profile.inspect)(context.predecessor.path);
  check(parent.inventorySha256 === context.predecessor.inventorySha256, "noise_predecessor_seal_changed");
  if (!historical) {
    await missing(resolve(root, "sealed-inventory.json")); await missing(resolve(root, "final-result.json"));
    const observed = await sources({ firmwareRoot: context.firmware_root, gateRoot: context.gate_root, manifest: context.manifest, fixtureBinary: context.fixture_binary }, operations, profile);
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
  await (operations.taskAdmission ?? taskAdmission)(context.firmware_root, profileOf(context));
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
