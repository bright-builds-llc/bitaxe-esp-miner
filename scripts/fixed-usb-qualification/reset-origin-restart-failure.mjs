import { digest, exactObject, QualificationError, requireCondition as check, writeNew } from "./contract.mjs";
import { proof } from "./cadence-premining-evidence.mjs";
import { resolve } from "node:path";

/** Preserve the first closed failure code; a later cleanup failure cannot replace it. */
export async function saveRestartFailure(root, context, error) {
  const code = typeof error === "string" ? error : error instanceof QualificationError ? error.code : "restart_local_operation_failed";
  check(/^[a-z][a-z0-9_]{0,95}$/u.test(code), "restart_failure_code");
  const path = resolve(root, "restart-failure.json"),
    hash = digest(JSON.stringify(context));
  try {
    await writeNew(path, { schema: "fixed-usb-restart-failure-v1", context_sha256: hash, code });
  } catch (failure) {
    if (failure.code !== "EEXIST") throw failure;
    const existing = (await proof(path)).value;
    exactObject(existing, ["schema", "context_sha256", "code"]);
    check(
      existing.schema === "fixed-usb-restart-failure-v1" &&
        existing.context_sha256 === hash &&
        /^[a-z][a-z0-9_]{0,95}$/u.test(existing.code),
      "restart_failure_changed",
    );
  }
}
