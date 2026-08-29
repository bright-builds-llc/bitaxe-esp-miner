import {
  inspectTcpPayloadDiagnosticPreflight,
  tcpPayloadDiagnosticFailureResult,
  tcpPayloadDiagnosticWorkspaceRoot,
  parseTcpPayloadDiagnosticArgs,
  runTcpPayloadDiagnostic,
} from "./stratum-v2-tcp-payload.js";
import { finalizeTcpPayloadDiagnostic } from "./stratum-v2-tcp-finalize.js";
import { runTcpPayloadRecovery } from "./stratum-v2-tcp-recovery.js";

async function main(): Promise<number> {
  try {
    const [action, ...values] = process.argv.slice(2);
    const args = parseTcpPayloadDiagnosticArgs(action, values);
    const workspace = tcpPayloadDiagnosticWorkspaceRoot();
    const result = action === "finalize"
      ? await finalizeTcpPayloadDiagnostic(workspace, args)
      : action === "preflight"
      ? await inspectTcpPayloadDiagnosticPreflight(workspace, args)
      : action === "recover"
        ? await runTcpPayloadRecovery(workspace, args)
        : await runTcpPayloadDiagnostic(workspace, args);
    process.stdout.write(`${JSON.stringify(result)}\n`);
    return 0;
  } catch (error) {
    process.stdout.write(`${JSON.stringify(tcpPayloadDiagnosticFailureResult(error))}\n`);
    return 1;
  }
}

process.exitCode = await main();
