import { mkdir, readFile } from "node:fs/promises";
import { resolve } from "node:path";
import { missing } from "../fixed-usb-qualification/contract.mjs";
import { privateRoot, protectedPath, retain } from "./files.mjs";
export const CLEANUP_FILES = ["browser.json", "fixture.json", "supervisor.json", "resources.json", "receipt.json"];
/** Freeze parent evidence before classification, including its incomplete membership. */
export async function snapshotCleanup(root) {
  const destination = resolve(root, "final-inputs/cleanup"); await missing(resolve(root, "final-inputs"));
  await mkdir(destination, { recursive: true, mode: 0o700 });
  try { await privateRoot(`${root}.cleanup`); }
  catch (error) { if (error.code === "ENOENT") return; throw error; }
  for (const name of CLEANUP_FILES) {
    const source = resolve(`${root}.cleanup`, name);
    try { await protectedPath(source); await retain(resolve(destination, name), await readFile(source)); }
    catch (error) { if (error.code !== "ENOENT") throw error; }
  }
}
