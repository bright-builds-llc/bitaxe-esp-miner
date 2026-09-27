import test from "node:test";
import assert from "node:assert/strict";
import { stableIdentity } from "./test-provenance.mjs";
const status = `STABLE_BITAXE_SOURCE_COMMIT ${"a".repeat(40)}\nSTABLE_BITAXE_SOURCE_DIRTY false\n`;
test("stable test identity excludes volatile status fields", () => { assert.deepEqual(stableIdentity(status + "BUILD_TIMESTAMP 123\n"), { schema: "usb-bootstrap-reader-test-identity-v1", sourceCommit: "a".repeat(40), sourceDirty: false }); });
test("dirty identity remains explicitly dirty", () => { assert.equal(stableIdentity(status.replace("false", "true")).sourceDirty, true); });
test("missing or duplicate source fields reject", () => { assert.throws(() => stableIdentity("")); assert.throws(() => stableIdentity(status + "STABLE_BITAXE_SOURCE_DIRTY false\n")); });
