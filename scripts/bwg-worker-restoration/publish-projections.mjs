import { lstat, unlink } from "node:fs/promises";

import { canonicalJson, SCENARIOS } from "./contract.mjs";
import { writeAtomicNew } from "./atomic-publication.mjs";
import { validatePublicProjection } from "./projection-validator.mjs";

export async function publishProjectionSet(completed) {
  if (SCENARIOS.some((scenario) => !completed.has(scenario))) return false;
  if (new Set([...completed.values()].map((entry) =>
    entry.deviceIdentityFingerprintSha256)).size !== 1) {
    throw new Error("device_identity_drift");
  }
  for (const { target } of completed.values()) {
    try {
      await lstat(target);
      throw new Error("projection_exists");
    } catch (error) {
      if (error?.code !== "ENOENT") throw error;
    }
  }
  const published = [];
  try {
    for (const scenario of SCENARIOS) {
      const { target, projection } = completed.get(scenario);
      const publicProjection = validatePublicProjection({
        ...projection,
        sameDeviceAcrossScenarios: true,
      });
      await writeAtomicNew(target, `${canonicalJson(publicProjection)}\n`, 0o644);
      published.push(target);
    }
  } catch (error) {
    const cleanupErrors = [];
    for (const target of published) {
      try {
        await unlink(target);
      } catch (cleanupError) {
        cleanupErrors.push(cleanupError);
      }
    }
    if (cleanupErrors.length > 0) {
      throw new AggregateError([error, ...cleanupErrors], "projection_rollback_failed");
    }
    throw error;
  }
  return true;
}
