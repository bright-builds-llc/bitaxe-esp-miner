import { lstat, readFile } from "node:fs/promises";
import path from "node:path";

import type { ProcessOutcome } from "./process.js";

const CHECKPOINT_SCHEMA = "bitaxe-identify-checkpoint-v2";
const POLL_INTERVAL_MILLIS = 50;

export type OperatorCheckpointKind = "ready" | "rendered" | "cleared";
type ConfirmationCondition = "ready_to_watch" | "identify_frame_visible" | "identify_frame_absent";

export type OperatorCheckpointSignal = {
  readonly schema_version: "bitaxe-operator-checkpoint-v2";
  readonly command: "api-command-effects-campaign";
  readonly checkpoint: OperatorCheckpointKind;
  readonly confirm_when: ConfirmationCondition;
  readonly expected_frame: readonly ["", "BITAXE IDENTIFY", "Hello!", ""];
  readonly operator_ready_timeout_seconds: 3600;
  readonly identify_duration_seconds: 30;
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

const checkpoints: readonly OperatorCheckpointKind[] = ["ready", "rendered", "cleared"];
const expectedFrame = ["", "BITAXE IDENTIFY", "Hello!", ""] as const;

function checkpointPath(root: string, checkpoint: OperatorCheckpointKind): string {
  return path.join(root, `identify-${checkpoint}.required.json`);
}

function confirmationCondition(checkpoint: OperatorCheckpointKind): ConfirmationCondition {
  switch (checkpoint) {
    case "ready": return "ready_to_watch";
    case "rendered": return "identify_frame_visible";
    case "cleared": return "identify_frame_absent";
  }
}

function signal(checkpoint: OperatorCheckpointKind): OperatorCheckpointSignal {
  return {
    schema_version: "bitaxe-operator-checkpoint-v2",
    command: "api-command-effects-campaign",
    checkpoint,
    confirm_when: confirmationCondition(checkpoint),
    expected_frame: expectedFrame,
    operator_ready_timeout_seconds: 3600,
    identify_duration_seconds: 30,
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
  checkpoint: OperatorCheckpointKind,
  strict: boolean,
): Promise<OperatorCheckpointSignal | undefined> {
  const input = checkpointPath(root, checkpoint);
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
    || Object.keys(value).sort().join(",") !== "checkpoint,schema,status"
    || value["schema"] !== CHECKPOINT_SCHEMA
    || value["checkpoint"] !== checkpoint
    || value["status"] !== "required"
  ) {
    if (strict) throw new OperatorCheckpointError();
    return undefined;
  }
  return signal(checkpoint);
}

async function delay(): Promise<void> {
  await new Promise((resolve) => setTimeout(resolve, POLL_INTERVAL_MILLIS));
}

async function emitAvailable(
  campaignRoot: string,
  sink: OperatorCheckpointSink,
  nextCheckpoint: number,
  strict: boolean,
): Promise<number> {
  let next = nextCheckpoint;
  while (next < checkpoints.length) {
    const checkpoint = checkpoints[next];
    if (checkpoint === undefined) throw new OperatorCheckpointError();
    const maybeSignal = await readCheckpoint(campaignRoot, checkpoint, strict);
    if (maybeSignal === undefined) {
      for (const later of checkpoints.slice(next + 1)) {
        if (await readCheckpoint(campaignRoot, later, strict) !== undefined) {
          throw new OperatorCheckpointError();
        }
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

  let nextCheckpoint = 0;
  let maybeCheckpointError: OperatorCheckpointError | undefined;
  while (maybeSettlement === undefined) {
    if (maybeCheckpointError === undefined) {
      try {
        nextCheckpoint = await emitAvailable(campaignRoot, sink, nextCheckpoint, false);
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
      nextCheckpoint = await emitAvailable(campaignRoot, sink, nextCheckpoint, true);
      if (resolved.outcome.exitCode === 0 && nextCheckpoint !== checkpoints.length) {
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
