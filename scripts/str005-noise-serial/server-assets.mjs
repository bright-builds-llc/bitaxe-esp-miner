import { readFile } from "node:fs/promises";
import { resolve } from "node:path";
import { BUNDLE } from "../fixed-usb-qualification/contract.mjs";
import { send } from "../fixed-usb-qualification/http.mjs";
import { check, digest } from "./files.mjs";

export function configuration(context, mode, trust) {
  const source = mode === "before" ? context.before_source : context;
  return { expectedGateCommit: context.gate_commit, expectedFirmwareSourceCommit: source.firmware_commit,
    expectedAppElfSha256: source.app_elf_sha256, trust, noiseQualification: mode,
    noiseIdentities: { before: { firmwareSourceCommit: context.before_source.firmware_commit, appElfSha256: context.before_source.app_elf_sha256 },
      candidate: { firmwareSourceCommit: context.firmware_commit, appElfSha256: context.app_elf_sha256 } } };
}
export async function serveAsset(root, context, pathname, response) {
  if (!["/", `/${BUNDLE}`, "/noise-client.mjs"].includes(pathname)) return false;
  const client = pathname === "/noise-client.mjs", page = pathname === "/";
  const path = client ? resolve(context.firmware_root, "scripts/str005-noise-serial/client.mjs") :
    resolve(root, "qualified-artifacts/gate", page ? context.gate_page_relative_path : BUNDLE);
  let bytes = await readFile(path);
  check(digest(bytes) === (client ? context.client_sha256 : page ? context.gate_page_sha256 : context.gate_bundle_sha256), "noise_asset_changed");
  if (page) bytes = Buffer.from(`${bytes.toString("utf8")}\n<script type="module" src="/noise-client.mjs"></script>`);
  send(response, 200, bytes, page ? "text/html" : "text/javascript");
  return true;
}
