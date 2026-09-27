import { readFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { BUNDLE } from "../fixed-usb-qualification/contract.mjs";
import { canonical, inventory, proof, protectedPath, retain, verifyInventory, writeNew } from "../str005-noise-serial/files.mjs";
import { writeSourceSnapshot, verifySourceSnapshot } from "./source-snapshot.mjs";
import { validateWriter } from "./native-writer.mjs";
import { validateTrust } from "./sources.mjs";
import { validateLayoutCheck } from "./source-layout-check.mjs";
import { CONTEXT_V2, check, schema, sha256 } from "./values.mjs";
export async function writeSnapshot(root, context, writer, correction, closure) {
  await retain(resolve(root, "snapshot/manifest.json"), await readFile(context.package.manifest));
  await writeNew(resolve(root, "snapshot/native-writer.json"), writer);
  const manifest = JSON.parse(await readFile(context.package.manifest));
  for (const file of manifest.artifacts) await retain(resolve(root, "snapshot/package", file.kind), await readFile(resolve(file.kind === "partition_table" ? context.firmwareRoot : dirname(context.package.manifest), file.path)));
  for (const [name, path] of [["page", context.gate.pageRelativePath], ["bundle", BUNDLE]]) await retain(resolve(root, "snapshot/gate", name), await readFile(resolve(context.gateRoot, path)));
  await retain(resolve(root, "snapshot/trust.json"), await readFile(resolve(context.firmwareRoot, "firmware/bitaxe/bwg/deployment-trust.json")));
  await writeNew(resolve(root, "snapshot/native.json"), context.nativeReadiness);
  await writeSourceSnapshot(root, context);
  await writeNew(resolve(root, "snapshot/source-layout-check.json"), correction);
  await retain(resolve(root, "snapshot/preflight-closure.json"), (await proof(dirname(closure.closurePath), closure.closurePath)).bytes);
  await writeNew(resolve(root, "preflight-inventory.json"), { schema: schema("preflight-inventory"), files: await inventory(root) });
}

/** Verifies frozen bytes only; this does not admit a staged context for effects. */
export async function verifySnapshot(root, context) {
  const frozen = (await proof(root, "preflight-inventory.json")).value;
  check(frozen.schema === schema("preflight-inventory"), "bootstrap_snapshot");
  for (const file of frozen.files) {
    check(typeof file.path === "string" && !file.path.startsWith("/") && !file.path.split("/").includes(".."), "bootstrap_snapshot_path");
    await protectedPath(resolve(root, file.path));
    const data = await readFile(resolve(root, file.path)); check(data.length === file.length && sha256(data) === file.sha256, "bootstrap_snapshot_changed");
  }
  const expectedSnapshot = frozen.files.filter(file => file.path.startsWith("snapshot/")).map(file => ({ ...file, path: file.path.slice("snapshot/".length) }));
  await verifyInventory(resolve(root, "snapshot"), expectedSnapshot);
  await verifySourceSnapshot(root, context);
  check((await proof(root, "snapshot/manifest.json")).sha256 === context.package.manifest_sha256 &&
    sha256(JSON.stringify((await proof(root, "snapshot/trust.json")).value)) === context.trustSha256 &&
    canonical((await proof(root, "snapshot/native.json")).value) === canonical(context.nativeReadiness), "bootstrap_snapshot_binding");
  for (const file of context.package.artifacts) check(sha256(await readFile(resolve(root, "snapshot/package", file.kind))) === file.sha256, "bootstrap_package_snapshot");
  validateTrust((await proof(root, "snapshot/trust.json")).value);
  validateWriter((await proof(root, "snapshot/native-writer.json")).value, context);
  if (context.schema === CONTEXT_V2) {
    validateLayoutCheck((await proof(root, "snapshot/source-layout-check.json")).value, context);
    check((await proof(root, "snapshot/preflight-closure.json")).sha256 === context.preflightSupersession.closureSha256, "bootstrap_closure_changed");
  }
}
