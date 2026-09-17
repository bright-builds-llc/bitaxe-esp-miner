import { lstat, readFile, realpath } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { BUNDLE, PAGE, admitTrust, canonicalDirectory, cleanPushed, fileDigest, git, packageSnapshot, readJson } from "../fixed-usb-qualification/contract.mjs";
import { canonical } from "../str005-noise-serial/files.mjs";
import { requireActiveTask } from "./contract.mjs";
import { AMENDMENT_PATH, AMENDMENT_SHA256, PERMISSION_AMENDMENT_PATH, PERMISSION_AMENDMENT_SHA256, check, CONTRACT_PATH, CONTRACT_SHA256, object, sha256 } from "./values.mjs";

export { AMENDMENT_PATH, AMENDMENT_SHA256 } from "./values.mjs";
const SOURCE_DIRS = ["scripts/str005-v2-serial", "scripts/str005-noise-serial", "scripts/fixed-usb-qualification", "scripts/host-stalls",
  "tools/stratum-v2-fixture", "tools/http-transport", "crates/bitaxe-stratum", "crates/bitaxe-worker-control", "firmware/bitaxe/src"];
const SOURCE_FILES = [CONTRACT_PATH, AMENDMENT_PATH, PERMISSION_AMENDMENT_PATH, "Cargo.toml", "Cargo.lock", "MODULE.bazel", "firmware/bitaxe/bwg/deployment-trust.json",
  "tools/automation/src/redaction.ts", "scripts/str005-v2-serial/client.mjs", "scripts/str005-v2-serial/operator.mjs", "scripts/str005-v2-serial/observer-build-identity.mjs", "scripts/str005-v2-serial/permission-correction.mjs"];

/** Test injection is code-only; production cannot select an auditor through flags or environment. */
export async function nativeInterface(operations = {}) {
  if (operations.inspectNative) {
    check(Array.isArray(operations.nativeSourceFiles) && Array.isArray(operations.nativeAuditorSources), "v2_native_test_interface");
    return { inspect: operations.inspectNative, sources: operations.nativeSourceFiles, auditors: operations.nativeAuditorSources };
  }
  const module = await import("../v2-native-readiness.mjs");
  return { inspect: module.inspectV2NativeReadiness, sources: module.V2_NATIVE_SOURCE_FILES, auditors: module.V2_NATIVE_AUDITOR_SOURCES };
}
export async function readContractBinding(root) {
  const binding = { base: { path: CONTRACT_PATH, sha256: CONTRACT_SHA256 }, amendment: { path: AMENDMENT_PATH, sha256: AMENDMENT_SHA256 },
    permission: { path: PERMISSION_AMENDMENT_PATH, sha256: PERMISSION_AMENDMENT_SHA256 } };
  for (const item of Object.values(binding)) check(await fileDigest(resolve(root, item.path)) === item.sha256, "v2_contract_changed");
  return { contracts: binding, contractSha256: sha256(canonical(binding)) };
}
export function validateSourcePath(path) {
  check(typeof path === "string" && /^[A-Za-z0-9_./-]+$/u.test(path) && !path.startsWith("/") &&
    path.split("/").every((part) => part !== ".." && part !== "." && part.length > 0), "v2_evaluator_path");
}
export async function sourceInventory(root, required) {
  const paths = git(root, ["ls-files", "--", ...SOURCE_DIRS, ...SOURCE_FILES, ...required]).split("\n").filter(Boolean).sort();
  check(paths.length > 0 && new Set(paths).size === paths.length && [...SOURCE_FILES, ...required].every((path) => paths.includes(path)), "v2_evaluator_membership");
  const entries = [];
  for (const path of paths) {
    validateSourcePath(path);
    const fullPath = resolve(root, path), stat = await lstat(fullPath);
    check(stat.isFile() && !stat.isSymbolicLink() && await realpath(fullPath) === fullPath, "v2_evaluator_alias");
    const bytes = await readFile(fullPath); entries.push({ path, sha256: sha256(bytes), length: bytes.length });
  }
  return entries;
}
export async function inspectSources(options, native, operations = {}) {
  const firmwareRoot = await canonicalDirectory(options.firmwareRoot), gateRoot = await canonicalDirectory(options.gateRoot);
  requireActiveTask(await readFile(resolve(firmwareRoot, "TASKS.md"), "utf8"));
  const binding = await readContractBinding(firmwareRoot);
  const command = operations.git ?? git;
  const firmwareCommit = command(firmwareRoot, ["rev-parse", "HEAD"]), gateCommit = command(gateRoot, ["rev-parse", "HEAD"]);
  (operations.cleanPushed ?? cleanPushed)(firmwareRoot, firmwareCommit); (operations.cleanPushed ?? cleanPushed)(gateRoot, gateCommit);
  const pins = [...(await readFile(resolve(firmwareRoot, "MODULE.bazel"), "utf8")).matchAll(/strip_prefix\s*=\s*"bitaxe-turnstile-system-([a-f0-9]{40})"/gu)];
  check(pins.length === 1 && pins[0][1] === gateCommit, "v2_gate_pin_mismatch");
  const manifest = resolve(options.manifest), packaged = await packageSnapshot(firmwareRoot, manifest, firmwareCommit);
  const fixture = resolve(options.fixtureBinary);
  check(fixture === resolve(firmwareRoot, "bazel-bin/tools/stratum-v2-fixture/stratum_v2_fixture"), "v2_fixture_path");
  const receiptPath = resolve(dirname(fixture), "v2-serial-build-identity.json"), built = await readJson(receiptPath);
  object(built, ["schema", "sourceCommit", "sourceDirty", "fixtureSha256", "writerSha256"]);
  const fixtureHash = await fileDigest(fixture);
  check(built.schema === "str005-v2-fixture-build-v1" && built.sourceCommit === firmwareCommit && built.sourceDirty === false &&
    built.fixtureSha256 === fixtureHash && built.writerSha256 === await fileDigest(resolve(firmwareRoot, "scripts/str005-v2-serial/build-identity.mjs")), "v2_fixture_build_identity");
  const observerPath = resolve(firmwareRoot, "bazel-bin/tools/http-transport/cadence_observer");
  const observerStat = await lstat(observerPath);
  check(observerStat.isFile() && !observerStat.isSymbolicLink() && (observerStat.mode & 0o111) !== 0, "v2_observer_binary");
  const observer = { path: await realpath(observerPath), sha256: await fileDigest(observerPath) };
  const observerReceiptPath = resolve(dirname(observerPath), "v2-observer-build-identity.json"), observerBuilt = await readJson(observerReceiptPath);
  object(observerBuilt, ["schema", "sourceCommit", "sourceDirty", "observerSha256", "writerSha256"]);
  check(observerBuilt.schema === "str005-v2-observer-build-v1" && observerBuilt.sourceCommit === firmwareCommit && observerBuilt.sourceDirty === false &&
    observerBuilt.observerSha256 === observer.sha256 && observerBuilt.writerSha256 === await fileDigest(resolve(firmwareRoot, "scripts/str005-v2-serial/observer-build-identity.mjs")),
    "v2_observer_build_identity");
  const bundle = await readFile(resolve(gateRoot, BUNDLE));
  check([gateCommit, "bwg-worker-stratum-v2-standard/0.1", "worker-stratum-v2-status-v1", "stratumV2ChannelStart", "stratumV2Status", "stratumV2Scope"]
    .every((marker) => bundle.includes(marker)), "v2_gate_capability");
  const trust = await readJson(resolve(firmwareRoot, "firmware/bitaxe/bwg/deployment-trust.json")); admitTrust(trust, trust);
  const required = [...native.sources, ...native.auditors];
  check(native.sources.length > 0 && native.auditors.length > 0 && new Set(required).size === required.length, "v2_native_source_inventory");
  return { ...packaged, ...binding, firmware_root: firmwareRoot, gate_root: gateRoot, firmware_commit: firmwareCommit, gate_commit: gateCommit,
    manifest, cadence_observer: observer, observer_build_receipt_sha256: await fileDigest(observerReceiptPath), fixture_binary: fixture, fixture_sha256: fixtureHash, fixture_build_receipt_sha256: await fileDigest(receiptPath),
    gate_bundle_sha256: sha256(bundle), gate_page_relative_path: PAGE, gate_page_sha256: await fileDigest(resolve(gateRoot, PAGE)),
    trust_sha256: sha256(JSON.stringify(trust)), sdkconfig_sha256: await fileDigest(resolve(dirname(manifest), "bitaxe-firmware.sdkconfig")),
    client_sha256: await fileDigest(resolve(firmwareRoot, "scripts/str005-v2-serial/client.mjs")),
    operator_sha256: await fileDigest(resolve(firmwareRoot, "scripts/str005-v2-serial/operator.mjs")),
    evaluator: await sourceInventory(firmwareRoot, required), native_source_files: native.sources, native_auditor_sources: native.auditors };
}
export function requireNativeReadiness(context, receipt) {
  check(receipt?.schema === "str005-v2-native-readiness-v1" && receipt.result === "selected_native_checks_passed" &&
    receipt.firmwareCommit === context.firmware_commit && receipt.elfSha256 === context.app_elf_sha256 &&
    receipt.sdkconfigSha256 === context.sdkconfig_sha256 && receipt.hardwareQualified === false, "v2_native_readiness");
  for (const [key, paths] of [["sourceFiles", context.native_source_files], ["auditorSources", context.native_auditor_sources]]) {
    check(Array.isArray(receipt[key]) && receipt[key].length === paths.length && paths.every((path) => {
      const found = receipt[key].filter((entry) => entry.path === path);
      return found.length === 1 && found[0].sha256 === context.evaluator.find((entry) => entry.path === path)?.sha256;
    }), "v2_native_source_join");
  }
}
