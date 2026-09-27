import { resolve } from "node:path";
import { loadOperatorContext } from "./context.mjs";
import { writeNew } from "../str005-noise-serial/files.mjs";
/** A marked synthetic bounded operation, not a firmware or device substitute. */
export async function installCandidate(root, index) {
  await loadOperatorContext(root);
  await writeNew(resolve(root, `synthetic-operation-${index}.json`), { index, started: true });
  await new Promise(done => setTimeout(done, 250));
  return { install_verified: true };
}
