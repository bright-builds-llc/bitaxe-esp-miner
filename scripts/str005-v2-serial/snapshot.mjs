import { readFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { BUNDLE, PAGE, readJson } from "../fixed-usb-qualification/contract.mjs";
import { verifyArtifactSnapshot } from "../fixed-usb-qualification/snapshot.mjs";
import { canonical, inventory, privateRoot, proof, protectedPath, retain, writeNew } from "../str005-noise-serial/files.mjs";
import { requireNativeReadiness, validateSourcePath } from "./context-sources.mjs";
import { validateHostCorrection } from "./host-correction.mjs";
import { validatePermissionCorrection } from "./permission-correction.mjs";
import { check, object, sha256 } from "./values.mjs";

/** Fresh thirteen-artifact snapshot, plus immutable source/native/fixture inputs. */
export async function createSnapshot(root, context, { permissionCorrection, hostCorrection } = {}) {
  const files = [], manifest = await readJson(context.manifest);
  async function add(path, source) {
    const bytes = await readFile(source); await retain(resolve(root, "qualified-artifacts", path), bytes);
    files.push({ path, sha256: sha256(bytes), length: bytes.length });
  }
  await add("firmware/bitaxe-ultra205-package.json", context.manifest);
  for (const item of manifest.artifacts) await add(`firmware/${item.path}`, resolve(item.kind === "partition_table" ? context.firmware_root : dirname(context.manifest), item.path));
  for (const name of ["license-inventory", "provenance-manifest"]) await add(`firmware/docs/release/${name}.md`, resolve(context.firmware_root, `docs/release/${name}.md`));
  for (const path of [BUNDLE, PAGE]) await add(`gate/${path}`, resolve(context.gate_root, path));
  await writeNew(resolve(root, "artifact-snapshot.json"), { schema: "fixed-usb-qualified-artifacts-v1", context_sha256: sha256(JSON.stringify(context)), files });
  await retain(resolve(root, "observer/observer.bin"), await readFile(context.cadence_observer.path));
  await retain(resolve(root, "observer/build-identity.json"), await readFile(resolve(dirname(context.cadence_observer.path), "v2-observer-build-identity.json")));
  await retain(resolve(root, "fixture/fixture.bin"), await readFile(context.fixture_binary));
  await retain(resolve(root, "fixture/build-identity.json"), await readFile(resolve(dirname(context.fixture_binary), "v2-serial-build-identity.json")));
  await retain(resolve(root, "native/bitaxe-firmware.sdkconfig"), await readFile(resolve(dirname(context.manifest), "bitaxe-firmware.sdkconfig")));
  await writeNew(resolve(root, "native/readiness.json"), context.native_readiness);
  for (const entry of context.evaluator) {
    validateSourcePath(entry.path); const bytes = await readFile(resolve(context.firmware_root, entry.path));
    check(sha256(bytes) === entry.sha256 && bytes.length === entry.length, "v2_evaluator_changed");
    await retain(resolve(root, "evaluator", entry.path), bytes);
  }
  if (["str005-v2-serial-context-v2", "str005-v2-serial-context-v3"].includes(context.schema)) await writeNew(resolve(root, "permission-correction.json"),
    validatePermissionCorrection(permissionCorrection, context));
  if (context.schema === "str005-v2-serial-context-v3") await writeNew(resolve(root, "host-correction.json"), validateHostCorrection(hostCorrection, context));
  await verifySnapshot(root, context, { creating: true });
  await writeNew(resolve(root, "preflight-inventory.json"), { schema: "str005-v2-serial-preflight-inventory-v1", files: await inventory(root) });
}
export async function verifySnapshot(root, context, { creating = false } = {}) {
  await privateRoot(root); await verifyArtifactSnapshot(root, context);
  if (["str005-v2-serial-context-v2", "str005-v2-serial-context-v3"].includes(context.schema)) validatePermissionCorrection((await proof(root, "permission-correction.json")).value, context);
  if (context.schema === "str005-v2-serial-context-v3") validateHostCorrection((await proof(root, "host-correction.json")).value, context);
  check(canonical((await proof(root, "native/readiness.json")).value) === canonical(context.native_readiness), "v2_native_snapshot_changed");
  requireNativeReadiness(context, context.native_readiness);
  object(context.cadence_observer, ["path", "sha256"]);
  check(sha256(await protectedBytes(root, "observer/observer.bin")) === context.cadence_observer.sha256 &&
    (await proof(root, "observer/build-identity.json")).sha256 === context.observer_build_receipt_sha256, "v2_observer_snapshot_changed");
  check(sha256(await protectedBytes(root, "fixture/fixture.bin")) === context.fixture_sha256 &&
    (await proof(root, "fixture/build-identity.json")).sha256 === context.fixture_build_receipt_sha256 &&
    sha256(await protectedBytes(root, "native/bitaxe-firmware.sdkconfig")) === context.sdkconfig_sha256, "v2_auxiliary_snapshot_changed");
  check(Array.isArray(context.evaluator) && context.evaluator.length > 0 && new Set(context.evaluator.map((entry) => entry.path)).size === context.evaluator.length,
    "v2_evaluator_inventory");
  for (const name of ["client", "operator"]) check(context[`${name}_sha256`] === context.evaluator.find((entry) => entry.path === `scripts/str005-v2-serial/${name}.mjs`)?.sha256,
    "v2_helper_snapshot_binding");
  for (const entry of context.evaluator) {
    object(entry, ["path", "sha256", "length"]); validateSourcePath(entry.path);
    const bytes = await protectedBytes(root, `evaluator/${entry.path}`);
    check(sha256(bytes) === entry.sha256 && bytes.length === entry.length, "v2_evaluator_snapshot_changed");
  }
  if (!creating) await verifyPreflightInventory(root, context);
}
async function protectedBytes(root, name) {
  validateSourcePath(name);
  const path = resolve(root, name); let parent = dirname(path);
  while (parent !== root) { await privateRoot(parent); parent = dirname(parent); }
  await protectedPath(path); return readFile(path);
}
async function verifyPreflightInventory(root, context) {
  const record = (await proof(root, "preflight-inventory.json")).value;
  object(record, ["schema", "files"]);
  check(record.schema === "str005-v2-serial-preflight-inventory-v1" && Array.isArray(record.files), "v2_preflight_inventory");
  const artifact = (await proof(root, "artifact-snapshot.json")).value;
  const expected = new Set(["context.json", "artifact-snapshot.json", "observer/observer.bin", "observer/build-identity.json", "fixture/fixture.bin", "fixture/build-identity.json",
    "native/readiness.json", "native/bitaxe-firmware.sdkconfig", ...(["str005-v2-serial-context-v2", "str005-v2-serial-context-v3"].includes(context.schema) ? ["permission-correction.json"] : []),
    ...(context.schema === "str005-v2-serial-context-v3" ? ["host-correction.json"] : []), ...artifact.files.map((item) => `qualified-artifacts/${item.path}`),
    ...context.evaluator.map((item) => `evaluator/${item.path}`)]);
  for (const item of record.files) {
    object(item, ["path", "sha256", "length"]);
    check(expected.delete(item.path), "v2_preflight_membership");
    const bytes = await protectedBytes(root, item.path);
    check(sha256(bytes) === item.sha256 && bytes.length === item.length, "v2_preflight_changed");
  }
  check(expected.size === 0, "v2_preflight_membership");
}
