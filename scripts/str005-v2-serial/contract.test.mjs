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
