import { resolve } from "node:path";
import { proof, writeNew } from "../str005-noise-serial/files.mjs";
import { contextHash } from "./context.mjs";
import { code, schema } from "./values.mjs";
export async function fail(root, context, source, stage, error, observationSha256 = null) {
  const value = { schema: schema("failure"), contextSha256: contextHash(context), source, stage, code: code(error), observationSha256 };
  try { await writeNew(resolve(root, "failure.json"), value); return value; }
  catch (e) { if (e.code !== "EEXIST") throw e; return (await proof(root, "failure.json")).value; }
}
