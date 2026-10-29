// The board a SELF-001 campaign runs on (ADR-0041 item 14): the Ultra 205's verified attempt-005
// contract, kept exactly, a later 205 attempt under the rerun plan, or a Gamma 601 attempt under its
// own root, plan and catalog default.

export type SelfTestCampaignBoard = 205 | 601;

export type SelfTestCampaignProfile = {
  readonly board: SelfTestCampaignBoard;
  readonly attemptOrdinal: number;
  readonly root: string;
  readonly projection: string;
  readonly plan: string;
  readonly planSha256: string;
  readonly activeTask: string;
  readonly taskTerms: readonly string[];
  readonly frequencyMhz: number;
  readonly coreVoltageMv: number;
};

const ultra205Plan = "docs/parity/work-plans/20260822T024037Z-SELF-001-RETRY-4/PLAN.md";
const ultra205RerunPlan = "docs/parity/work-plans/20261029T050000Z-SELF-001-ULTRA205-RERUN/PLAN.md";
const gamma601Plan = "docs/parity/work-plans/20261026T000000Z-SELF-001-GAMMA601/PLAN.md";

export const ULTRA205_SELF_TEST_PROFILE: SelfTestCampaignProfile = Object.freeze({
  board: 205,
  attemptOrdinal: 5,
  root: "scratch/self001-full-lifecycle/attempt-005",
  projection: "docs/parity/evidence/self001-full-lifecycle/self-test-projection.json",
  plan: ultra205Plan,
  planSha256: "0c9a03ec490967fc95989d88b91848c7d4ed740a76825822a8107d94e8fd7f84",
  activeTask: "task-parity-self001-full-lifecycle",
  taskTerms: Object.freeze([ultra205Plan, "two-phase", "BOOT-button", "mode-0700"]),
  frequencyMhz: 485,
  coreVoltageMv: 1_200,
});

const ultra205Root = /^scratch\/self001-full-lifecycle\/attempt-(\d{3})$/u;
const gamma601Root = /^scratch\/self001-gamma601\/attempt-(\d{3})$/u;

/** The ordinal a root names under `pattern`, or 0 when it names none. */
function rootOrdinal(pattern: RegExp, relativeRoot: string): number {
  const match = pattern.exec(relativeRoot);
  return match?.[1] === undefined ? 0 : Number(match[1]);
}

/** A 205 attempt after the verified one: the same contract under the rerun plan (F4.6). */
function ultra205RerunProfile(ordinal: number, relativeRoot: string): SelfTestCampaignProfile {
  const attempt = `attempt-${String(ordinal).padStart(3, "0")}`;
  return Object.freeze({
    ...ULTRA205_SELF_TEST_PROFILE,
    attemptOrdinal: ordinal,
    root: relativeRoot,
    projection: `docs/parity/evidence/self001-full-lifecycle/${attempt}/self-test-projection.json`,
    plan: ultra205RerunPlan,
    planSha256: "f1ed7dc3ffc400b1d73387a3532c933420907f2aa1000b840e5bceab5c51c31d",
    activeTask: "task-gamma-601-bring-up",
    taskTerms: Object.freeze([ultra205RerunPlan, "two-phase", "BOOT-button", "mode-0700"]),
  });
}

/**
 * The profile for `maybeBoard` (the 205 when absent) whose private root is `relativeRoot`, or
 * null when the root is not one the board's contract admits.
 */
export function selfTestCampaignProfile(
  maybeBoard: string | undefined,
  relativeRoot: string,
): SelfTestCampaignProfile | null {
  if (maybeBoard === undefined || maybeBoard === "205") {
    if (relativeRoot === ULTRA205_SELF_TEST_PROFILE.root) return ULTRA205_SELF_TEST_PROFILE;
    const ordinal = rootOrdinal(ultra205Root, relativeRoot);
    return ordinal > ULTRA205_SELF_TEST_PROFILE.attemptOrdinal
      ? ultra205RerunProfile(ordinal, relativeRoot)
      : null;
  }
  if (maybeBoard !== "601") return null;
  const ordinal = rootOrdinal(gamma601Root, relativeRoot);
  if (ordinal < 1) return null;
  const attempt = `attempt-${String(ordinal).padStart(3, "0")}`;
  return Object.freeze({
    board: 601,
    attemptOrdinal: ordinal,
    root: relativeRoot,
    projection: `docs/parity/evidence/self001-gamma601/${attempt}/self-test-projection.json`,
    plan: gamma601Plan,
    planSha256: "584b784b96b06867d7854b485cd0d4c5f829ce3711345f23d132fd90a307e098",
    activeTask: "task-gamma-601-bring-up",
    taskTerms: Object.freeze([gamma601Plan, "two-phase", "BOOT-button", "mode-0700"]),
    frequencyMhz: 525,
    coreVoltageMv: 1_150,
  });
}

/**
 * Whether the campaign admits the package: production only, on either board, since the 601's
 * promotion (ADR-0041 item 15). A package without a variant reads as production (older packages,
 * such as the 205's attempt-005 one). Every test build is refused: an overheat-trip build would
 * trip at the warm-up target, and the 601's checks never run the self-test.
 */
export function admitsCampaignPackage(manifest: Readonly<Record<string, unknown>>): boolean {
  const maybeVariant = manifest["firmware_variant"];
  return maybeVariant === undefined || maybeVariant === "production";
}
