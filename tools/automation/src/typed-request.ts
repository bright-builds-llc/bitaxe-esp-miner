import { readFile } from "node:fs/promises";

import {
  InvocationError,
  optionValue,
  type ParsedInvocation,
} from "./invocation.js";
import { assertWithinWorkspace } from "./workspace.js";

export async function typedRequestArguments(
  root: string,
  invocation: ParsedInvocation,
): Promise<string[]> {
  const request = assertWithinWorkspace(root, optionValue(invocation, "--request"));
  const value: unknown = JSON.parse(await readFile(request, "utf8"));
  if (typeof value !== "object" || value === null) {
    throw new InvocationError("typed request must be a JSON object");
  }
  const workflow = (value as Record<string, unknown>)["workflow"];
  if (typeof workflow !== "object" || workflow === null) {
    throw new InvocationError("typed request workflow is missing");
  }
  const identity = workflow as Record<string, unknown>;
  if (identity["schema_version"] !== "bitaxe-workflow-identity-v1") {
    throw new InvocationError("typed request workflow schema is invalid");
  }
  if (typeof identity["command"] !== "string"
    || typeof identity["request_sha256"] !== "string") {
    throw new InvocationError("typed request workflow identity is incomplete");
  }
  return ["--manifest", request, "--workflow", identity["command"], "--request-sha256", identity["request_sha256"]];
}
