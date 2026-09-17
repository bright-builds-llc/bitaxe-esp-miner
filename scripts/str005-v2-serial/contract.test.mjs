import assert from "node:assert/strict";
import test from "node:test";
import { attemptName, parseArgs, requireActiveTask, requireAuthorityOption, TASK_ID } from "./contract.mjs";

const preflight = ["preflight", "--scope", "channel", "--private-root", "/private/scratch/channel-001",
  "--firmware-root", "/firmware", "--gate-root", "/gate", "--package-manifest", "/package.json",
  "--fixture-binary", "/fixture", "--predecessor-receipt", "/predecessor.json"];

test("only the closed effect-free preflight options are accepted", () => {
  // Arrange / Act
  const result = parseArgs(preflight);
  // Assert
  assert.equal(result.options.scope, "channel");
  assert.equal(result.options.authorityDirectory, undefined);
  for (const name of ["authority-directory", "pool-credentials", "endpoint", "force", "attempt-ordinal"])
    assert.throws(() => parseArgs([...preflight, `--${name}`, "rejected"]), { code: "v2_option_rejected" });
});

test("duplicate options cannot hide a conflicting root or scope", () => {
  assert.throws(() => parseArgs([...preflight, "--scope", "share"]), { code: "v2_option_rejected" });
  assert.throws(() => parseArgs([...preflight, "--private-root", "/other"]), { code: "v2_option_rejected" });
});

test("canonical attempt names separate host assignment from mining ordinal", () => {
  assert.equal(attemptName("/private/channel-001", "channel"), 1);
  assert.equal(attemptName("/private/share-001", "share"), 1);
  for (const name of ["channel-000", "channel-1", "channel-0001", "share-001", "channel-9007199254740992"])
    assert.throws(() => attemptName(`/private/${name}`, "channel"));
});

test("a finalized task ID cannot be admitted from an archive or Future heading", () => {
  assert.doesNotThrow(() => requireActiveTask(`## Active\n### ${TASK_ID} | date | title\n`));
  for (const heading of ["## Future", "## Archive", "## Completed"])
    assert.throws(() => requireActiveTask(`${heading}\n### ${TASK_ID} | date | title\n`), { code: "v2_live_task_inactive" });
  assert.throws(() => requireActiveTask(`## Active\n### ${TASK_ID} | date | title\n## Future\n### ${TASK_ID} | duplicate\n`));
});

test("channel rejects signing paths before a caller may read them", () => {
  assert.throws(() => requireAuthorityOption("channel", { authorityDirectory: "/do-not-open" }), { code: "v2_authority_scope" });
  assert.throws(() => requireAuthorityOption("share", {}), { code: "v2_authority_scope" });
  assert.doesNotThrow(() => requireAuthorityOption("channel", {}));
  assert.doesNotThrow(() => requireAuthorityOption("share", { authorityDirectory: "/admitted-private-authority" }));
});

test("recovery accepts no reset, flash or historical attempt option", () => {
  assert.equal(parseArgs(["recover", "--private-root", "/private/channel-001"]).action, "recover");
  assert.throws(() => parseArgs(["recover", "--private-root", "/private/channel-001", "--attempt-root", "/old"]));
});

test("permission closure actions take only the source root and no effect inputs", () => {
  // Arrange / Act / Assert
  for (const action of ["close-permission", "review-permission"]) {
    assert.equal(parseArgs([action, "--private-root", "/private/channel-001"]).action, action);
    for (const flag of ["authority-directory", "cleanup-receipt", "endpoint", "supersede-permission", "output"])
      assert.throws(() => parseArgs([action, "--private-root", "/private/channel-001", `--${flag}`, "/forbidden"]), { code: "v2_option_rejected" });
  }
});
test("supersession is an optional Channel preflight path, never a Share or relative override", () => {
  // Arrange / Act
  const result = parseArgs([...preflight, "--supersede-permission", "/private/channel-001.permission-closure.json"]);
  // Assert
  assert.equal(result.options.supersedePermission, "/private/channel-001.permission-closure.json");
  assert.throws(() => parseArgs([...preflight, "--supersede-permission", "relative.json"]), { code: "v2_permission_scope" });
  const share = [...preflight]; share[2] = "share";
  assert.throws(() => parseArgs([...share, "--supersede-permission", "/private/channel-001.permission-closure.json"]), { code: "v2_permission_scope" });
});

test("cleanup successor actions expose no source overrides, signing inputs or output destination", () => {
  // Arrange / Act / Assert.
  for (const action of ["prepare-channel-successor", "review-channel-successor"]) {
    assert.equal(parseArgs([action, "--private-root", "/private/channel-002"]).action, action);
    for (const flag of ["authority-directory", "cleanup-receipt", "endpoint", "supersede-channel", "supersede-permission", "output", "force"])
      assert.throws(() => parseArgs([action, "--private-root", "/private/channel-002", `--${flag}`, "/forbidden"]), { code: "v2_option_rejected" });
  }
});

test("cleanup supersession is Channel-only and mutually exclusive with permission supersession", () => {
  // Arrange.
  const path = "/private/channel-002.successor-readiness.json";
  // Act / Assert.
  assert.equal(parseArgs([...preflight, "--supersede-channel", path]).options.supersedeChannel, path);
  assert.throws(() => parseArgs([...preflight, "--supersede-channel", "relative.json"]), { code: "v2_cleanup_scope" });
  const share = [...preflight]; share[2] = "share";
  assert.throws(() => parseArgs([...share, "--supersede-channel", path]), { code: "v2_cleanup_scope" });
  assert.throws(() => parseArgs([...preflight, "--supersede-channel", path, "--supersede-permission", "/private/closure.json"]), { code: "v2_supersession_conflict" });
});
