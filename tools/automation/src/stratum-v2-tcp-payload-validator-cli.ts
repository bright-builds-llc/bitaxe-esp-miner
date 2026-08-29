import { validateTcpPayloadDiagnosticProjection } from "./stratum-v2-tcp-payload-validator.js";
import { sourceWorkspaceRoot } from "./workspace.js";

const [candidate, expectedSource, ordinal, extra] = process.argv.slice(2);
if (candidate === undefined || expectedSource === undefined || ordinal === undefined
  || !/^[1-9][0-9]*$/u.test(ordinal) || extra !== undefined) {
  process.exitCode = 2;
} else {
  try {
    const configured = process.env["BUILD_WORKSPACE_DIRECTORY"];
    const workspace = sourceWorkspaceRoot(
      configured === undefined ? [process.cwd()] : [configured, process.cwd()],
    );
    await validateTcpPayloadDiagnosticProjection(
      candidate,
      expectedSource,
      Number(ordinal),
      workspace,
    );
    process.stdout.write('{"status":"accepted"}\n');
  } catch {
    process.exitCode = 1;
  }
}
