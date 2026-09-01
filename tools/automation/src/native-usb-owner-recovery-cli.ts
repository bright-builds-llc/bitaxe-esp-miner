import {
  ownerRecoveryWorkspaceRoot,
  parseOwnerRecoveryArgs,
  runOwnerRecovery,
} from "./native-usb-owner-recovery.js";

try {
  const [action, ...values] = process.argv.slice(2);
  const result = await runOwnerRecovery(
    ownerRecoveryWorkspaceRoot(),
    parseOwnerRecoveryArgs(action, values),
  );
  process.stdout.write(`${JSON.stringify(result)}\n`);
} catch (error) {
  const category = error instanceof Error ? error.message : "unexpected_failure";
  process.stdout.write(`${JSON.stringify({
    schema_version: "bitaxe-native-usb-owner-recovery-failure-v1",
    status: "failed",
    category,
  })}\n`);
  process.exitCode = 1;
}
