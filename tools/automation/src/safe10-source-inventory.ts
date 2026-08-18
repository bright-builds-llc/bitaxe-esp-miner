import { createHash } from "node:crypto";
import { readFile } from "node:fs/promises";
import path from "node:path";

export const safe10ProductionFragments = new Map<string, readonly string[]>([
  ["crates/bitaxe-api/src/observation.rs", ["pub fn is_ultra_205_mining_safe_at("]],
  ["crates/bitaxe-safety/src/mining_preconditions.rs", ["pub struct ProductionMiningPreconditions {"]],
  ["crates/bitaxe-stratum/src/v1/recovery_policy.rs", ["pub struct ProductionReadiness {", "if !self.safety_prerequisites_fresh"]],
  ["crates/bitaxe-stratum/src/v1/production_session/runtime.rs", ["pub fn snapshot(&self) -> ProductionSessionSnapshot"]],
  ["firmware/bitaxe/src/production_mining_session.rs", ["let safety_prerequisites_fresh = observations.is_ultra_205_mining_safe_at(now());"]],
  ["firmware/bitaxe/src/production_mining_session/readiness_trace.rs", ["safety_sample_fresh: readiness.safety_prerequisites_fresh"]],
  ["tools/flash/src/campaign/markers.rs", ["pub(super) observation_freshness: ObservationFreshnessMarker"]],
  ["tools/flash/src/campaign/evidence.rs", ["observation_freshness: maybe_terminal.map"]],
  ["tools/flash/src/campaign/network/model.rs", ["safety_valid: self.safety_valid"]],
]);

export const safe10EvaluatorFragments = new Map<string, readonly string[]>([
  ["tools/automation/src/safe10-evidence.ts", ["export async function projectSafe10Evidence("]],
  ["tools/automation/src/safe10-source-inventory.ts", []],
  ["tools/automation/src/cli.ts", ['invocation.command === "project-safe10-evidence"']],
  ["tools/automation/src/invocation.ts", ['"project-safe10-evidence": {']],
  ["tools/automation/src/detector.ts", ["export async function portFromDetectorOutput("]],
  ["tools/automation/src/process.ts", ["export function createLocalProcessPort("]],
  ["crates/bitaxe-automation-contracts/src/safe10_evidence.rs", ["pub struct Safe10Evidence {"]],
  ["crates/bitaxe-automation-contracts/src/bin/validate_safe10_evidence.rs", ["evidence.validate()?;"]],
]);

export const safe10ReferenceFragments = new Map<string, readonly string[]>([
  ["reference/esp-miner/main/tasks/protocol_coordinator.c", ["void protocol_coordinator_task(void *pvParameters)"]],
  ["reference/esp-miner/main/tasks/power_management_task.c", ["void POWER_MANAGEMENT_task(void * pvParameters)"]],
]);

function verifyFragments(document: string, fragments: readonly string[]): void {
  for (const fragment of fragments) {
    if (document.split(fragment).length !== 2) {
      throw new Error("SAFE-10 source semantics are invalid");
    }
  }
}

export async function safe10CurrentInventory(workspaceRoot: string): Promise<{
  readonly digest: string;
  readonly productionDigest: string;
  readonly pathCount: number;
}> {
  const digest = createHash("sha256");
  const productionDigest = createHash("sha256");
  for (const [relative, fragments] of [
    ...safe10ProductionFragments,
    ...safe10EvaluatorFragments,
    ...safe10ReferenceFragments,
  ]) {
    const document = await readFile(path.join(workspaceRoot, relative));
    verifyFragments(document.toString("utf8"), fragments);
    digest.update(relative).update("\0").update(document).update("\0");
    if (safe10ProductionFragments.has(relative)) {
      productionDigest.update(relative).update("\0").update(document).update("\0");
    }
  }
  return {
    digest: digest.digest("hex"),
    productionDigest: productionDigest.digest("hex"),
    pathCount: safe10ProductionFragments.size
      + safe10EvaluatorFragments.size
      + safe10ReferenceFragments.size,
  };
}

export function safe10AttemptProductionDigest(documents: ReadonlyMap<string, Buffer>): string {
  const digest = createHash("sha256");
  for (const relative of safe10ProductionFragments.keys()) {
    const document = documents.get(relative);
    if (document === undefined) throw new Error("SAFE-10 attempt source is incomplete");
    digest.update(relative).update("\0").update(document).update("\0");
  }
  return digest.digest("hex");
}
