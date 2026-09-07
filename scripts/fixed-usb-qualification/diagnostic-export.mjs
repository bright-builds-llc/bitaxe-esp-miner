import { spawn } from "node:child_process";
import { resolve } from "node:path";
import { digest, exactObject, nonce, QualificationError, requireCondition, writeNew } from "./contract.mjs";

export async function validateDiagnosticExport(input, gateRoot, program = "bun") {
  exactObject(input, ["schema", "observations"]);
  requireCondition(input.schema === "worker-diagnostic-export-v1" && Array.isArray(input.observations) && input.observations.length <= 40,
    "diagnostic_export_shape");
  return new Promise((resolveResult, reject) => {
    const script = "const m=await import(process.argv.at(-1));process.stdout.write(JSON.stringify(m.parseWorkerDiagnosticExport(await Bun.stdin.json())));";
    const child = spawn(program, ["-e", script, resolve(gateRoot, "web/worker-diagnostic-export.ts")], { cwd: gateRoot, stdio: ["pipe", "pipe", "pipe"] });
    const chunks = []; let size = 0, failed = false;
    const timer = setTimeout(() => { failed = true; child.kill("SIGKILL"); }, 5000);
    child.stdout.on("data", (chunk) => { size += chunk.length; if (size > 65536) { failed = true; child.kill("SIGKILL"); } else chunks.push(chunk); });
    child.stderr.on("data", () => { failed = true; });
    child.stdin.on("error", () => { failed = true; child.kill("SIGKILL"); });
    child.once("error", () => { clearTimeout(timer); reject(new QualificationError("diagnostic_validator_unavailable")); });
    child.once("close", (code) => {
      clearTimeout(timer);
      if (failed || code !== 0) { reject(new QualificationError("diagnostic_export_rejected")); return; }
      try { resolveResult(JSON.parse(Buffer.concat(chunks).toString("utf8"))); }
      catch { reject(new QualificationError("diagnostic_validator_output")); }
    });
    child.stdin.end(JSON.stringify(input));
  });
}
export async function saveDiagnosticExport(root, context, input, validate) {
  const value = await validate(input);
  const file = `diagnostic-export-${nonce()}.json`;
  await writeNew(resolve(root, file), { schema: "fixed-usb-diagnostic-export-v1", context_sha256: digest(JSON.stringify(context)), export: value,
    hardware_authority: false });
  return { diagnostic_export_saved: true, review_file: file };
}
