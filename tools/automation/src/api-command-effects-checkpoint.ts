import { lstat, readFile } from "node:fs/promises";
import path from "node:path";

import type { ProcessOutcome } from "./process.js";

const CHECKPOINT_SCHEMA = "bitaxe-identify-checkpoint-v3";
const POLL_INTERVAL_MILLIS = 50;

export type OperatorCheckpointKind = "ready" | "rendered" | "replayed" | "cleared";
type ConfirmationCondition = "ready_to_watch" | "identify_frame_visible" | "identify_frame_absent";

export type OperatorCheckpointSignal = {
  readonly schema_version: "bitaxe-operator-checkpoint-v5";
  readonly command: "api-command-effects-campaign";
  readonly checkpoint: OperatorCheckpointKind;
  readonly confirm_when: ConfirmationCondition;
  readonly expected_frame: readonly ["", "BITAXE IDENTIFY", "Hello!", ""];
  readonly identify_duration_seconds: 30;
  readonly operator_wait_policy: "unbounded";
  readonly effect_evidence_window_seconds: 30 | "not_applicable";
  readonly local_signal_required: true;
  readonly starts_identify_window: boolean;
  readonly replay_supported: boolean;
  readonly replay_limit: 1;
  readonly replay_starts_identify_window: boolean;
  readonly late_confirmation_policy: "reject" | "not_applicable";
  readonly decline_supported: true;
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
  readonly checkpointKinds: readonly OperatorCheckpointKind[];
  readonly maybeCheckpointError?: OperatorCheckpointError;
};

type CampaignSettlement =
  | { readonly kind: "outcome"; readonly outcome: ProcessOutcome }
  | { readonly kind: "error"; readonly error: unknown };

type CheckpointCursor = "ready" | "rendered" | "replayed_or_cleared" | "cleared" | "complete";

const laterCheckpoints: Readonly<Record<Exclude<CheckpointCursor, "complete">, readonly OperatorCheckpointKind[]>> = {
  ready: ["rendered", "replayed", "cleared"],
  rendered: ["replayed", "cleared"],
  replayed_or_cleared: [],
  cleared: [],
};
const expectedFrame = ["", "BITAXE IDENTIFY", "Hello!", ""] as const;

function checkpointPath(root: string, checkpoint: OperatorCheckpointKind): string {
  return path.join(root, `identify-${checkpoint}.required.json`);
}

function confirmationCondition(checkpoint: OperatorCheckpointKind): ConfirmationCondition {
  switch (checkpoint) {
    case "ready": return "ready_to_watch";
    case "rendered": return "identify_frame_visible";
    case "replayed": return "identify_frame_visible";
    case "cleared": return "identify_frame_absent";
  }
}

function signal(checkpoint: OperatorCheckpointKind): OperatorCheckpointSignal {
  const effectBounded = checkpoint === "rendered" || checkpoint === "replayed";
  return {
    schema_version: "bitaxe-operator-checkpoint-v5",
    command: "api-command-effects-campaign",
    checkpoint,
    confirm_when: confirmationCondition(checkpoint),
    expected_frame: expectedFrame,
    identify_duration_seconds: 30,
    operator_wait_policy: "unbounded",
    effect_evidence_window_seconds: effectBounded ? 30 : "not_applicable",
    // Consuming ready starts the device's 30-second parity effect. Requiring
    // the local command keeps transport or conversational latency outside it.
    local_signal_required: true,
    starts_identify_window: checkpoint === "ready",
    replay_supported: checkpoint === "rendered",
    replay_limit: 1,
    replay_starts_identify_window: checkpoint === "rendered",
    late_confirmation_policy: effectBounded ? "reject" : "not_applicable",
    decline_supported: true,
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

async function assertLaterAbsent(
  campaignRoot: string,
  cursor: Exclude<CheckpointCursor, "complete">,
  strict: boolean,
): Promise<void> {
  for (const later of laterCheckpoints[cursor]) {
    if (await readCheckpoint(campaignRoot, later, strict) !== undefined) {
      throw new OperatorCheckpointError();
    }
  }
}

async function emitAvailable(
  campaignRoot: string,
  sink: OperatorCheckpointSink,
  cursor: CheckpointCursor,
  strict: boolean,
): Promise<CheckpointCursor> {
  let next = cursor;
  while (next !== "complete") {
    if (next === "replayed_or_cleared") {
      const maybeReplayed = await readCheckpoint(campaignRoot, "replayed", strict);
      const maybeCleared = await readCheckpoint(campaignRoot, "cleared", strict);
      if (maybeReplayed !== undefined) {
        await sink(maybeReplayed);
        next = "cleared";
        continue;
      }
      if (maybeCleared !== undefined) {
        await sink(maybeCleared);
        return "complete";
      }
      return next;
    }

    const checkpoint = next;
    const maybeSignal = await readCheckpoint(campaignRoot, checkpoint, strict);
    if (maybeSignal === undefined) {
      await assertLaterAbsent(campaignRoot, next, strict);
      return next;
    }
    await sink(maybeSignal);
    next = checkpoint === "ready"
      ? "rendered"
      : checkpoint === "rendered"
        ? "replayed_or_cleared"
        : "complete";
  }
  return next;
}

export async function superviseOperatorCheckpoints(
  campaignPromise: Promise<ProcessOutcome>,
  campaignRoot: string,
  sink: OperatorCheckpointSink,
): Promise<SupervisedCampaign> {
  const checkpointKinds: OperatorCheckpointKind[] = [];
  const recordingSink: OperatorCheckpointSink = async (checkpoint) => {
    await sink(checkpoint);
    checkpointKinds.push(checkpoint.checkpoint);
  };
  let maybeSettlement: CampaignSettlement | undefined;
  const settlement: Promise<CampaignSettlement> = campaignPromise.then(
    (outcome): CampaignSettlement => ({ kind: "outcome", outcome }),
    (error: unknown): CampaignSettlement => ({ kind: "error", error }),
  );
  void settlement.then((value) => { maybeSettlement = value; });

  let checkpointCursor: CheckpointCursor = "ready";
  let maybeCheckpointError: OperatorCheckpointError | undefined;
  while (maybeSettlement === undefined) {
    if (maybeCheckpointError === undefined) {
      try {
        checkpointCursor = await emitAvailable(
          campaignRoot,
          recordingSink,
          checkpointCursor,
          false,
        );
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
      checkpointCursor = await emitAvailable(
        campaignRoot,
        recordingSink,
        checkpointCursor,
        true,
      );
      if (resolved.outcome.exitCode === 0 && checkpointCursor !== "complete") {
        maybeCheckpointError = new OperatorCheckpointError();
      }
    } catch {
      maybeCheckpointError = new OperatorCheckpointError();
    }
  }
  return {
    outcome: resolved.outcome,
    checkpointKinds,
    ...(maybeCheckpointError === undefined ? {} : { maybeCheckpointError }),
  };
}

export function formatOperatorCheckpointSignal(signalValue: OperatorCheckpointSignal): string {
  return `bitaxe-automation-checkpoint: ${JSON.stringify(signalValue)}\n`;
}

export function emitOperatorCheckpointSignal(signalValue: OperatorCheckpointSignal): void {
  process.stderr.write(formatOperatorCheckpointSignal(signalValue));
}
