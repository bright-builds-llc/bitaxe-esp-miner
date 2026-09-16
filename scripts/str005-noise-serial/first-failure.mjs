import { resolve } from "node:path";
import { digest, proof, writeNew } from "./files.mjs";
import { parseFixtureTerminal } from "./fixture.mjs";

/** Persist once after the caller synchronously latches its terminal-failure fence. */
export async function saveFirstFailure(root, context, code, atHostMs, maybeCause, maybeProvenance) {
  let cause = maybeCause ?? { stage: "evidence", category: "evidence_incomplete", detail: "missing" };
  let provenance = maybeProvenance ?? { kind: "host", code };
  if (!maybeCause && code.startsWith("noise_fixture_")) {
    try {
      const receipt = await proof(root, "fixture-run/terminal.json"), parsed = parseFixtureTerminal(receipt.value);
      if (parsed.failure) { cause = parsed.failure; provenance = { kind: "fixture", sha256: receipt.sha256 }; }
    } catch (error) { if (error.code !== "ENOENT") provenance = { kind: "host", code: "noise_fixture_evidence_invalid" }; }
  }
  await writeNew(resolve(root, "failure.json"), { schema: "noise-serial-host-failure-v2", contextSha256: digest(JSON.stringify(context)),
    code, atHostMs, cause, provenance });
}
