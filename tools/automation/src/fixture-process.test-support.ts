import { internalCommandSpec } from "./contracts.generated.js";
import { createLocalProcessPort, type ProcessPort } from "./process.js";

type FixtureInterpreter = "node" | "shell";

/** Execute registered fixture source through its declared interpreter and the real process adapter. */
export function createFixtureProcessPort(
  options: Parameters<typeof createLocalProcessPort>[0],
  fixtures: Readonly<Record<string, FixtureInterpreter>>,
): ProcessPort {
  const local = createLocalProcessPort(options);
  const nodeProgram = process.env["JS_BINARY__NODE_BINARY"] ?? process.execPath;
  return {
    loadEspEnvironment: local.loadEspEnvironment,
    run(spec, maybeLifetime) {
      const maybeInterpreter = Object.hasOwn(fixtures, spec.program) ? fixtures[spec.program] : undefined;
      if (maybeInterpreter === undefined) return local.run(spec, maybeLifetime);
      return local.run(internalCommandSpec(
        maybeInterpreter === "node" ? nodeProgram : "/bin/sh",
        [spec.program, ...spec.args],
        spec.result,
        spec.environment,
      ), maybeLifetime);
    },
  };
}
