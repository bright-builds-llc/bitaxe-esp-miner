import { lstat, readFile } from "node:fs/promises";
import path from "node:path";

import type { ProcessOutcome } from "./process.js";

const CHECKPOINT_SCHEMA = "bitaxe-identify-checkpoint-v1";
const POLL_INTERVAL_MILLIS = 50;

export type OperatorCheckpointObservation = "rendered" | "cleared";

export type OperatorCheckpointSignal = {
  readonly schema_version: "bitaxe-operator-checkpoint-v1";
  readonly command: "api-command-effects-campaign";
  readonly observation: OperatorCheckpointObservation;
  readonly status: "required";
};

export type OperatorCheckpointSink = (
  signal: OperatorCheckpointSignal,
) => void | Promise<void>;

export class OperatorCheckpointError extends Error {
  public constructor() {
    super("operator checkpoint handoff is invalid");
    this.name = "OperatorCheckpointError";
  }
}

type SupervisedCampaign = {
  readonly outcome: ProcessOutcome;
  readonly maybeCheckpointError?: OperatorCheckpointError;
};

type CampaignSettlement =
  | { readonly kind: "outcome"; readonly outcome: ProcessOutcome }
  | { readonly kind: "error"; readonly error: unknown };

const observations: readonly OperatorCheckpointObservation[] = ["rendered", "cleared"];

function checkpointPath(root: string, observation: OperatorCheckpointObservation): string {
  return path.join(root, `identify-${observation}.required.json`);
}

function signal(observation: OperatorCheckpointObservation): OperatorCheckpointSignal {
  return {
    schema_version: "bitaxe-operator-checkpoint-v1",
    command: "api-command-effects-campaign",
    observation,
    status: "required",
  };
}

function isMissing(error: unknown): boolean {
  return (error as NodeJS.ErrnoException).code === "ENOENT";
}

function isObject(value: unknown): value is Readonly<Record<string, unknown>> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

async function readCheckpoint(
  root: string,
  observation: OperatorCheckpointObservation,
  strict: boolean,
): Promise<OperatorCheckpointSignal | undefined> {
  const input = checkpointPath(root, observation);
  let metadata;
  try {
    metadata = await lstat(input);
  } catch (error) {
    if (isMissing(error)) return undefined;
    if (strict) throw new OperatorCheckpointError();
    return undefined;
  }
  if (!metadata.isFile() || metadata.isSymbolicLink() || (metadata.mode & 0o777) !== 0o600) {
    if (strict) throw new OperatorCheckpointError();
    return undefined;
  }
  let value: unknown;
  try {
    value = JSON.parse(await readFile(input, "utf8"));
  } catch {
    if (strict) throw new OperatorCheckpointError();
    return undefined;
  }
  if (
    !isObject(value)
    || Object.keys(value).sort().join(",") !== "observation,schema,status"
    || value["schema"] !== CHECKPOINT_SCHEMA
    || value["observation"] !== observation
    || value["status"] !== "required"
  ) {
    if (strict) throw new OperatorCheckpointError();
    return undefined;
  }
  return signal(observation);
}

async function delay(): Promise<void> {
  await new Promise((resolve) => setTimeout(resolve, POLL_INTERVAL_MILLIS));
}

async function emitAvailable(
  campaignRoot: string,
  sink: OperatorCheckpointSink,
  nextObservation: number,
  strict: boolean,
): Promise<number> {
  let next = nextObservation;
  while (next < observations.length) {
    const observation = observations[next];
    if (observation === undefined) throw new OperatorCheckpointError();
    const maybeSignal = await readCheckpoint(campaignRoot, observation, strict);
    if (maybeSignal === undefined) {
      if (
        observation === "rendered"
        && await readCheckpoint(campaignRoot, "cleared", strict) !== undefined
      ) {
        throw new OperatorCheckpointError();
      }
      break;
    }
    await sink(maybeSignal);
    next += 1;
  }
  return next;
}

export async function superviseOperatorCheckpoints(
  campaignPromise: Promise<ProcessOutcome>,
  campaignRoot: string,
  sink: OperatorCheckpointSink,
): Promise<SupervisedCampaign> {
  let maybeSettlement: CampaignSettlement | undefined;
  const settlement: Promise<CampaignSettlement> = campaignPromise.then(
    (outcome): CampaignSettlement => ({ kind: "outcome", outcome }),
    (error: unknown): CampaignSettlement => ({ kind: "error", error }),
  );
  void settlement.then((value) => { maybeSettlement = value; });

  let nextObservation = 0;
  let maybeCheckpointError: OperatorCheckpointError | undefined;
  while (maybeSettlement === undefined) {
    if (maybeCheckpointError === undefined) {
      try {
        nextObservation = await emitAvailable(campaignRoot, sink, nextObservation, false);
      } catch {
        maybeCheckpointError = new OperatorCheckpointError();
      }
    }
    if (maybeSettlement === undefined) await delay();
  }

  const resolved = await settlement;
  if (resolved.kind === "error") throw resolved.error;
  if (maybeCheckpointError === undefined) {
    try {
      nextObservation = await emitAvailable(campaignRoot, sink, nextObservation, true);
      if (resolved.outcome.exitCode === 0 && nextObservation !== observations.length) {
        maybeCheckpointError = new OperatorCheckpointError();
      }
    } catch {
      maybeCheckpointError = new OperatorCheckpointError();
    }
  }
  return {
    outcome: resolved.outcome,
    ...(maybeCheckpointError === undefined ? {} : { maybeCheckpointError }),
  };
}

export function formatOperatorCheckpointSignal(signalValue: OperatorCheckpointSignal): string {
  return `bitaxe-automation-checkpoint: ${JSON.stringify(signalValue)}\n`;
}

export function emitOperatorCheckpointSignal(signalValue: OperatorCheckpointSignal): void {
  process.stderr.write(formatOperatorCheckpointSignal(signalValue));
}
