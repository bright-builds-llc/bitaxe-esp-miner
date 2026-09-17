import { fstatSync } from "node:fs";
import { once } from "node:events";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { parseArgs } from "./contract.mjs";
import { check } from "./values.mjs";

/** The closed command surface never obtains authority for recovery or review. */
export async function main(argv, operations = {}) {
  const { action, options } = parseArgs(argv);
  check(action !== "recover", "v2_serial_recovery_unavailable");
  if (action === "preflight") return (await import("./context.mjs")).preflight(options, operations);
  if (action === "finalize") return (await import("./finalize.mjs")).finalize(options.privateRoot, options.cleanupReceipt, operations);
  if (action === "review") return (await import("./finalize.mjs")).review(options.privateRoot, operations);
  const stdout = fstatSync(1);
  check(stdout.isFile() && (stdout.mode & 0o777) === 0o600, "v2_protected_stdout_required");
  const server = await (await import("./server.mjs")).createSupervisor(options, operations);
  const closed = new Promise((done) => server.once("close", done));
  let maybeCleanup, maybeFailure = null;
  const stop = () => {
    // Observe rejection immediately, then propagate it after every owner was
    // given cleanup. Closing active HTTP bodies happens inside this call.
    maybeCleanup ??= server.closeQualificationResources().then(() => null, (error) => error);
  };
  for (const signal of ["SIGINT", "SIGTERM"]) process.once(signal, stop);
  try {
    server.listen(0, "127.0.0.1");
    await once(server, "listening"); await server.qualificationReady;
    process.stdout.write(`qualification_url=http://127.0.0.1:${server.address().port}/\n`);
    await closed;
  } catch (error) { maybeFailure = error; }
  finally {
    for (const signal of ["SIGINT", "SIGTERM"]) process.removeListener(signal, stop);
    stop(); const cleanupFailure = await maybeCleanup; maybeFailure ??= cleanupFailure;
  }
  if (maybeFailure) throw maybeFailure;
  return { supervisor: "closed", hardware_qualified: false };
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main(process.argv.slice(2)).then((value) => process.stdout.write(`${JSON.stringify(value)}\n`)).catch((error) => {
    const code = typeof error.code === "string" && /^v2_[a-z_]+$/u.test(error.code) ? error.code : "v2_operation_rejected";
    process.stdout.write(`${JSON.stringify({ ready: false, hardware_qualified: false, error: code })}\n`);
    process.exitCode = 1;
  });
}
