import { resolve } from "node:path";
import { admitOperatorParent } from "./operator-parent.mjs";
import { main } from "./main.mjs";
import { check, object } from "./values.mjs";
check(process.connected && process.argv.length === 3, "v2_operator_owner");
const root = resolve(process.argv[2]);
const timer = setTimeout(() => process.exit(1), 10000);
process.once("message", async input => {
  try {
    object(input, ["binding", "maybeAuthorityDirectory"]);
    await admitOperatorParent(root, input.binding); clearTimeout(timer);
    const argv = ["serve", "--private-root", root];
    if (input.maybeAuthorityDirectory !== null) argv.push("--authority-directory", input.maybeAuthorityDirectory);
    const result = await main(argv); process.stdout.write(JSON.stringify(result) + "\n");
    if (process.connected) process.disconnect();
  } catch { process.exitCode = 1; clearTimeout(timer); if (process.connected) process.disconnect(); }
});
