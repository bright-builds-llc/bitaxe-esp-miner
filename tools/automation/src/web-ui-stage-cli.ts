// Bazel action entry point: `stage_web_ui --variant <v> --out <dir> [--strip-prefix <p>] --input <path>...`.
import { readInputs, resolveBazelPath, writeStage } from "./web-ui-files.js";
import { maybeWebUiVariant, stageWebUi } from "./web-ui-stage.js";

type StageArguments = { variant: string; out: string; stripPrefix: string; inputs: string[] };

/** Parses `--flag value` and `--flag=value` arguments; `--input` may repeat. */
export function parseStageArguments(argv: readonly string[]): StageArguments {
  const parsed: StageArguments = { variant: "", out: "", stripPrefix: "", inputs: [] };
  for (let index = 0; index < argv.length; index += 1) {
    const argument = argv[index] ?? "";
    const [flag, inline] = argument.includes("=") ? argument.split(/=(.*)/su, 2) : [argument, undefined];
    const value = inline ?? argv[(index += 1)];
    if (value === undefined) throw new Error(`missing value for ${String(flag)}`);
    if (flag === "--variant") parsed.variant = value;
    else if (flag === "--out") parsed.out = value;
    else if (flag === "--strip-prefix") parsed.stripPrefix = value;
    else if (flag === "--input") parsed.inputs.push(value);
    else throw new Error(`unknown argument ${String(flag)}`);
  }
  if (parsed.out === "" || parsed.inputs.length === 0) throw new Error("--out and at least one --input are required");
  return parsed;
}

async function main(): Promise<void> {
  const parsed = parseStageArguments(process.argv.slice(2));
  const maybeVariant = maybeWebUiVariant(parsed.variant);
  if (maybeVariant === null) throw new Error(`unknown web UI variant: ${parsed.variant}`);
  const { files, metadata } = stageWebUi(maybeVariant, await readInputs(parsed.inputs, parsed.stripPrefix));
  await writeStage(resolveBazelPath(parsed.out), files, metadata);
}

if (process.argv[1]?.endsWith("web-ui-stage-cli.js")) {
  main().catch((error: unknown) => {
    process.stderr.write(`web_ui_stage=failed reason=${error instanceof Error ? error.message : String(error)}\n`);
    process.exitCode = 1;
  });
}
