import { bootChainWorkspaceRoot, parseBootChainArgs, runBootChain } from "./native-usb-boot-chain-integrity.js";

try {
  const [action, ...values] = process.argv.slice(2);
  const result = await runBootChain(bootChainWorkspaceRoot(), parseBootChainArgs(action, values));
  process.stdout.write(`${JSON.stringify(result)}\n`);
} catch (error) {
  process.stdout.write(`${JSON.stringify({ schema_version: "bitaxe-native-usb-boot-chain-failure-v1", status: "failed", category: error instanceof Error ? error.message : "unexpected_failure" })}\n`);
  process.exitCode = 1;
}
