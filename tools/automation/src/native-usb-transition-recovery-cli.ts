import {
  finalizeNativeUsbRecovery,
  nativeUsbRecoveryFailure,
  nativeUsbRecoveryWorkspaceRoot,
  parseNativeUsbRecoveryArgs,
  preflightNativeUsbRecovery,
  startNativeUsbRecovery,
} from "./native-usb-transition-recovery.js";

async function main(): Promise<number> {
  try {
    const [action, ...values] = process.argv.slice(2);
    const args = parseNativeUsbRecoveryArgs(action, values);
    const workspace = nativeUsbRecoveryWorkspaceRoot();
    const result = action === "preflight"
      ? await preflightNativeUsbRecovery(workspace, args)
      : action === "start"
        ? await startNativeUsbRecovery(workspace, args)
        : await finalizeNativeUsbRecovery(workspace, args);
    process.stdout.write(`${JSON.stringify(result)}\n`);
    return 0;
  } catch (error) {
    process.stdout.write(`${JSON.stringify(nativeUsbRecoveryFailure(error))}\n`);
    return 1;
  }
}

process.exitCode = await main();
