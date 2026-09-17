import { resolve } from "node:path";
import { writeNew } from "../str005-noise-serial/files.mjs";
import { nullable, sha256, uint } from "./values.mjs";
import { failure as validateDeviceCause } from "./device-record.mjs";

/** Sticky cause with sanitized codes. Actual device failures retain their native provenance. */
export function failureRecorder(root, context, now) {
  let maybeFailure = null, maybeWriteError = null, writing = Promise.resolve();
  return {
    failed: () => maybeFailure !== null,
    current: () => maybeFailure === null ? null : structuredClone(maybeFailure),
    async settled() { await writing; if (maybeWriteError !== null) throw maybeWriteError; },
    fail(code, maybeDeviceCause = null, maybeSourceSequence = null) {
      if (maybeFailure !== null) return;
      nullable(maybeDeviceCause, validateDeviceCause); nullable(maybeSourceSequence, uint);
      const safeCode = typeof code === "string" && /^(?:v2|noise|iterative)_[a-z_]+$/u.test(code) ? code : "v2_operation_failed";
      maybeFailure = { schema: "str005-v2-first-failure-v1", contextSha256: sha256(JSON.stringify(context)),
        code: safeCode, atHostMs: now(), deviceCause: structuredClone(maybeDeviceCause), sourceSequence: maybeSourceSequence };
      // Event callbacks cannot await disk I/O; settled() propagates any write failure.
      writing = writeNew(resolve(root, "failure.json"), maybeFailure).catch((error) => { maybeWriteError = error; });
    },
  };
}
