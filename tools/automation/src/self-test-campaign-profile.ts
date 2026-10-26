// The board a SELF-001 campaign runs on (ADR-0041 item 14): the Ultra 205's verified attempt-005
// contract, kept exactly, or a Gamma 601 attempt under its own root, plan and catalog default.

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

const gamma601Root = /^scratch\/self001-gamma601\/attempt-(\d{3})$/u;

/**
 * The profile for `maybeBoard` (the 205 when absent) whose private root is `relativeRoot`, or
 * null when the root is not one the board's contract admits.
 */
export function selfTestCampaignProfile(
  maybeBoard: string | undefined,
  relativeRoot: string,
): SelfTestCampaignProfile | null {
  if (maybeBoard === undefined || maybeBoard === "205") {
    return relativeRoot === ULTRA205_SELF_TEST_PROFILE.root ? ULTRA205_SELF_TEST_PROFILE : null;
  }
  if (maybeBoard !== "601") return null;
  const match = gamma601Root.exec(relativeRoot);
  const ordinal = match?.[1] === undefined ? 0 : Number(match[1]);
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
 * The flash tool's admission for the campaign's package on `board`: a production package needs
 * none; the 601's `gamma601-mining` build, the only one that may power its BM1370 before
 * promotion, names itself. A package without a variant reads as production (older packages, such
 * as the 205's attempt-005 one). Any other test build is refused: an overheat-trip build would trip
 * at the warm-up target.
 */
export function testFirmwareArgs(
  board: SelfTestCampaignBoard,
  manifest: Readonly<Record<string, unknown>>,
): readonly string[] | null {
  const maybeVariant = manifest["firmware_variant"];
  if (maybeVariant === undefined || maybeVariant === "production") return [];
  if (board === 601 && maybeVariant === "gamma601-mining") {
    return ["--allow-test-firmware", "gamma601-mining"];
  }
  return null;
}
