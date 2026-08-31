import {
  parseRomExitArgs,
  romExitWorkspaceRoot,
  runRomExit,
} from "./native-usb-rom-exit.js";

try {
  const [action, ...values] = process.argv.slice(2);
  const result = await runRomExit(
    romExitWorkspaceRoot(),
    parseRomExitArgs(action, values),
  );
  process.stdout.write(`${JSON.stringify(result)}\n`);
} catch (error) {
  const category = error instanceof Error ? error.message : "unexpected_failure";
  process.stdout.write(`${JSON.stringify({
    schema_version: "bitaxe-native-usb-rom-exit-failure-v1",
    status: "failed",
    category,
  })}\n`);
  process.exitCode = 1;
}
