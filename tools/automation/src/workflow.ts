import type { CommandSpec } from "./contracts.generated.js";
import type { ProcessOutcome, ProcessPort } from "./process.js";

export async function executeCommandSpec(
  spec: CommandSpec<unknown>,
  processPort: ProcessPort,
): Promise<ProcessOutcome> {
  const outcome = await processPort.run(spec);
  if (outcome.timedOut) throw new Error("workflow timed out");
  if (outcome.exitCode !== 0) throw new Error("workflow process failed");
  return outcome;
}
