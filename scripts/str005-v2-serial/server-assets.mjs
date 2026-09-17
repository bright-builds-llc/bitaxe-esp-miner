import { readFile } from "node:fs/promises";
import { resolve } from "node:path";
import { BUNDLE } from "../fixed-usb-qualification/contract.mjs";
import { send } from "../fixed-usb-qualification/http.mjs";
import { check, sha256 } from "./values.mjs";

export function configuration(context, phase, trust) {
  check(["before", "candidate"].includes(phase), "v2_phase");
  const source = phase === "before" ? context.before_source : context;
  const identity = (value) => ({ firmwareSourceCommit: value.firmware_commit, appElfSha256: value.app_elf_sha256 });
  return { expectedGateCommit: context.gate_commit, expectedFirmwareSourceCommit: source.firmware_commit,
    expectedAppElfSha256: source.app_elf_sha256, trust, stratumV2Qualification: phase,
    stratumV2Identities: { before: identity(context.before_source), candidate: identity(context) }, stratumV2Scope: context.scope };
}

export async function serveAsset(root, context, pathname, response) {
  if (!["/", `/${BUNDLE}`, "/v2-client.mjs"].includes(pathname)) return false;
  const client = pathname === "/v2-client.mjs", page = pathname === "/";
  const path = client ? resolve(context.firmware_root, "scripts/str005-v2-serial/client.mjs") :
    resolve(root, "qualified-artifacts/gate", page ? context.gate_page_relative_path : BUNDLE);
  let bytes = await readFile(path);
  check(sha256(bytes) === (client ? context.client_sha256 : page ? context.gate_page_sha256 : context.gate_bundle_sha256), "v2_asset_changed");
  if (page) bytes = Buffer.from(`${bytes.toString("utf8")}\n<script type="module" src="/v2-client.mjs"></script>`);
  send(response, 200, bytes, page ? "text/html" : "text/javascript"); return true;
}
