// Test-only loader: replace admission, never spawn/filesystem/umask behavior.
// No production entrypoint accepts this hook as a flag or environment option.
import { registerHooks } from "node:module";
import { readFile, realpath } from "node:fs/promises";
import { resolve, sep } from "node:path";
import { tmpdir } from "node:os";
const root = process.argv[2], temporary = await realpath(tmpdir());
if (typeof root !== "string" || root !== await realpath(root) || !root.startsWith(`${temporary}${sep}`) ||
  await readFile(resolve(root, ".operator-private-output-test"), "utf8") !== "synthetic-filesystem-only\n") throw Error("operator_test_root_rejected");
const target = new URL("./operator-execution.mjs", import.meta.url).href;
const files = new URL("../fixed-usb-qualification/contract.mjs", import.meta.url).href;
registerHooks({
  load(url, context, next) {
    if (url !== target) return next(url, context);
    return { format: "module", shortCircuit: true, source: `
      import { missing } from ${JSON.stringify(files)};
      export const quoteJustArgument = value => value;
      export async function admitExecution(root, mode, index, permit) {
        if (root !== ${JSON.stringify(root)} || mode !== "flash" || index !== 0 || permit?.kind !== "synthetic-test")
          throw Error("operator_test_admission_rejected");
        await missing(root + "/install-0");
        return { context: {firmware_root: root}, argv: ["flash-monitor"] };
      }` };
  },
});
