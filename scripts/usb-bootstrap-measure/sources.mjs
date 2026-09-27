import { actualNodePath } from "./node-executable.mjs";
import { execFileSync } from "node:child_process";
import { readFile, realpath, lstat } from "node:fs/promises";
import { resolve } from "node:path";
import { BUNDLE, PAGE, canonicalDirectory, cleanPushed, git, fileDigest, packageSnapshot, admitTrust } from "../fixed-usb-qualification/contract.mjs";
import { canonical } from "../str005-noise-serial/files.mjs";
import { nativeInterface } from "../str005-v2-serial/context-sources.mjs";
import { CONTEXT_V4, ACCOUNTING_AMENDMENT, CONTRACT, PREFLIGHT_AMENDMENT, CONTEXT_V2, CONTEXT_V3, CORRECTION_CONTRACT, TASK, check, object, sha256 } from "./values.mjs";
export const SOURCE_ROOTS = ["scripts", "tools/flash", "tools/device-session", "tools/automation", "tools/stratum-v2-fixture", "tools/http-transport", "crates", "firmware/bitaxe", ".cargo", "docs/hardware", "Cargo.toml", "Cargo.lock", "MODULE.bazel", "BUILD.bazel", "Justfile"];
export function requireTask(text) {
  const block = text.split(/^## /mu).find(part => part.startsWith("Active\n"));
  check(block && [...text.matchAll(new RegExp(`^### ${TASK}(?:\\s|$)`, "gmu"))].length === 1 && new RegExp(`^### ${TASK}(?:\\s|$)`, "mu").test(block), "bootstrap_task_inactive");
}
export async function sources(root) {
  const paths = git(root, ["ls-files", "--", ...SOURCE_ROOTS]).split("\n").filter(Boolean).sort();
  check(paths.includes(CONTRACT.path) && paths.includes("scripts/usb-bootstrap-measure/main.mjs"), "bootstrap_source_inventory");
  return Promise.all(paths.map(async path => {
    const full = resolve(root, path), stat = await lstat(full); check(stat.isFile() && !stat.isSymbolicLink(), "bootstrap_source_alias");
    const bytes = await readFile(full); return { path, sha256: sha256(bytes), length: bytes.length };
  }));
}
async function tool(path, args, versionPattern) {
  path = await realpath(path); const stat = await lstat(path); check(stat.isFile(), "bootstrap_tool");
  const version = execFileSync(path, args, { encoding: "utf8", timeout: 10000, maxBuffer: 8192, stdio: ["ignore", "pipe", "pipe"] }).trim();
  check(versionPattern.test(version), "bootstrap_tool_version"); return { path, version, sha256: await fileDigest(path) };
}
export async function hostTools(root) {
  const espflash = execFileSync("which", ["espflash"], { encoding: "utf8", timeout: 5000 }).trim();
  const candidates = ["idf5.5_py3.14_env", "idf5.5_py3.9_env"].map(name => resolve(root, `.embuild/espressif/python_env/${name}/bin/esptool.py`));
  let maybeEsptool;
  for (const path of candidates) { try { await lstat(path); maybeEsptool = path; break; } catch (e) { if (e.code !== "ENOENT") throw e; } }
  check(maybeEsptool, "bootstrap_tool");
  return { espflash: await tool(espflash, ["--version"], /^espflash 4\.5\.0$/u), managedEsptool: await tool(maybeEsptool, ["version"], /\d+\.\d+/u),
    node: await tool(await actualNodePath(), ["--version"], /^v\d+\.\d+\.\d+$/u) };
}
export async function inspectSources(options, operations = {}) {
  const firmwareRoot = await canonicalDirectory(options.firmwareRoot), gateRoot = await canonicalDirectory(options.gateRoot);
  requireTask(await readFile(resolve(firmwareRoot, "TASKS.md"), "utf8"));
  check(await fileDigest(resolve(firmwareRoot, CONTRACT.path)) === CONTRACT.sha256 && await fileDigest(resolve(firmwareRoot, CORRECTION_CONTRACT.path)) === CORRECTION_CONTRACT.sha256 && await fileDigest(resolve(firmwareRoot, ACCOUNTING_AMENDMENT.path)) === ACCOUNTING_AMENDMENT.sha256, "bootstrap_contract_changed");
  const commit = (operations.git ?? git)(firmwareRoot, ["rev-parse", "HEAD"]), gateCommit = (operations.git ?? git)(gateRoot, ["rev-parse", "HEAD"]);
  (operations.cleanPushed ?? cleanPushed)(firmwareRoot, commit); (operations.cleanPushed ?? cleanPushed)(gateRoot, gateCommit);
  const pins = [...(await readFile(resolve(firmwareRoot, "MODULE.bazel"), "utf8")).matchAll(/strip_prefix\s*=\s*"bitaxe-turnstile-system-([a-f0-9]{40})"/gu)];
  check(pins.length === 1 && pins[0][1] === gateCommit, "bootstrap_gate_pin");
  const packageFacts = await (operations.packageSnapshot ?? packageSnapshot)(firmwareRoot, options.manifest, commit);
  const trust = JSON.parse(await readFile(resolve(firmwareRoot, "firmware/bitaxe/bwg/deployment-trust.json"))); validateTrust(trust);
  const native = await nativeInterface(operations), nativeReadiness = await native.inspect({ firmwareRoot, manifestPath: options.manifest, expectedSourceCommit: commit, expectedElfSha256: packageFacts.app_elf_sha256 });
  check(nativeReadiness.result === "selected_native_checks_passed" && nativeReadiness.hardwareQualified === false && nativeReadiness.firmwareCommit === commit &&
    nativeReadiness.elfSha256 === packageFacts.app_elf_sha256, "bootstrap_native_unverified");
  return { firmwareRoot, gateRoot, package: { ...packageFacts, firmware_commit: commit, manifest: resolve(options.manifest) },
    gate: { commit: gateCommit, bundleSha256: await fileDigest(resolve(gateRoot, BUNDLE)), pageSha256: await fileDigest(resolve(gateRoot, PAGE)), pageRelativePath: PAGE },
    trustSha256: sha256(JSON.stringify(trust)), sourceInventory: await sources(firmwareRoot), nativeReadiness, hostTools: await (operations.hostTools ?? hostTools)(firmwareRoot) };
}
export async function verifyCurrent(context, operations = {}) {
  requireTask(await readFile(resolve(context.firmwareRoot, "TASKS.md"), "utf8"));
  if ([CONTEXT_V3, CONTEXT_V4].includes(context.schema)) check(await fileDigest(resolve(context.firmwareRoot, CORRECTION_CONTRACT.path)) === CORRECTION_CONTRACT.sha256, "bootstrap_contract_changed");
  if (context.schema === CONTEXT_V2) check(await fileDigest(resolve(context.firmwareRoot, PREFLIGHT_AMENDMENT.path)) === PREFLIGHT_AMENDMENT.sha256, "bootstrap_contract_changed");
  (operations.cleanPushed ?? cleanPushed)(context.firmwareRoot, context.package.firmware_commit);
  (operations.cleanPushed ?? cleanPushed)(context.gateRoot, context.gate.commit);
  check(canonical(await sources(context.firmwareRoot)) === canonical(context.sourceInventory), "bootstrap_sources_changed");
  const packaged = await (operations.packageSnapshot ?? packageSnapshot)(context.firmwareRoot, context.package.manifest, context.package.firmware_commit);
  check(canonical({ ...packaged, firmware_commit: context.package.firmware_commit, manifest: context.package.manifest }) === canonical(context.package), "bootstrap_package_changed");
  check(await fileDigest(resolve(context.gateRoot, BUNDLE)) === context.gate.bundleSha256 && await fileDigest(resolve(context.gateRoot, PAGE)) === context.gate.pageSha256, "bootstrap_gate_changed");
  if (context.schema === CONTEXT_V4) check(await fileDigest(resolve(context.firmwareRoot, ACCOUNTING_AMENDMENT.path)) === ACCOUNTING_AMENDMENT.sha256, "bootstrap_contract_changed");
  for (const value of Object.values(context.hostTools)) check(await fileDigest(value.path) === value.sha256, "bootstrap_tool_changed");
}

export function validateTrust(trust) {
  object(trust, ["profile", "updateAuthority", "workLeaseAuthority"]);
  object(trust.updateAuthority, ["issuer", "audience", "role", "keys"]);
  object(trust.workLeaseAuthority, ["profile", "issuer", "audience", "role", "keys"]);
  admitTrust(trust, trust); return trust;
}
