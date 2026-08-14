export type CommandEffectsBudgetComponents = {
  readonly versionProbeMillis: number;
  readonly usbCommandCount: number;
  readonly usbCommandAttemptCount: number;
  readonly usbCommandMillis: number;
  readonly usbRetryRecoveryMillis: number;
  readonly usbRecoveryMillis: number;
  readonly activationMillis: number;
  readonly observationMillis: number;
  readonly terminalGraceMillis: number;
  readonly finalCleanupMillis: number;
  readonly processTerminationMillis: number;
  readonly fixtureStopMarginMillis: number;
};

export type CommandEffectsTransactionBudget = CommandEffectsBudgetComponents & {
  readonly childMaximumMillis: number;
  readonly parentTimeoutMillis: number;
  readonly fixtureDurationSeconds: number;
  readonly fixtureTimeoutMillis: number;
};

const commandEffectsPhaseMillis = 600_000;

const productionComponents: CommandEffectsBudgetComponents = {
  versionProbeMillis: 10_000,
  usbCommandCount: 3,
  usbCommandAttemptCount: 2,
  usbCommandMillis: 360_000,
  usbRetryRecoveryMillis: 30_000,
  usbRecoveryMillis: 150_000,
  activationMillis: commandEffectsPhaseMillis,
  observationMillis: commandEffectsPhaseMillis,
  terminalGraceMillis: 180_000,
  finalCleanupMillis: 60_000,
  processTerminationMillis: 5_000,
  fixtureStopMarginMillis: 10_000,
};

function checkedInteger(value: number, label: string): number {
  if (!Number.isSafeInteger(value) || value <= 0) {
    throw new Error(`${label} must be a positive safe integer`);
  }
  return value;
}

function checkedAdd(values: readonly number[]): number {
  let total = 0;
  for (const value of values) {
    total += value;
    if (!Number.isSafeInteger(total)) throw new Error("command effects budget overflow");
  }
  return total;
}

export function deriveCommandEffectsTransactionBudget(
  components: CommandEffectsBudgetComponents,
): CommandEffectsTransactionBudget {
  const versionProbeMillis = checkedInteger(components.versionProbeMillis, "version probe budget");
  const usbCommandCount = checkedInteger(components.usbCommandCount, "USB command count");
  const usbCommandAttemptCount = checkedInteger(
    components.usbCommandAttemptCount,
    "USB command attempt count",
  );
  const usbCommandMillis = checkedInteger(components.usbCommandMillis, "USB command budget");
  const usbRetryRecoveryMillis = checkedInteger(
    components.usbRetryRecoveryMillis,
    "USB retry recovery budget",
  );
  const usbRecoveryMillis = checkedInteger(components.usbRecoveryMillis, "USB recovery budget");
  const activationMillis = checkedInteger(components.activationMillis, "activation budget");
  const observationMillis = checkedInteger(components.observationMillis, "observation budget");
  const terminalGraceMillis = checkedInteger(components.terminalGraceMillis, "terminal grace budget");
  const finalCleanupMillis = checkedInteger(components.finalCleanupMillis, "final cleanup budget");
  const processTerminationMillis = checkedInteger(
    components.processTerminationMillis,
    "process termination budget",
  );
  const fixtureStopMarginMillis = checkedInteger(
    components.fixtureStopMarginMillis,
    "fixture stop margin",
  );
  const usbCommandsMillis = usbCommandCount * usbCommandAttemptCount * usbCommandMillis;
  if (!Number.isSafeInteger(usbCommandsMillis)) throw new Error("command effects budget overflow");
  const usbRetryCount = usbCommandCount * (usbCommandAttemptCount - 1);
  const usbRetryRecoveriesMillis = usbRetryCount * usbRetryRecoveryMillis;
  if (!Number.isSafeInteger(usbRetryRecoveriesMillis)) {
    throw new Error("command effects budget overflow");
  }
  const childMaximumMillis = checkedAdd([
    versionProbeMillis,
    usbCommandsMillis,
    usbRetryRecoveriesMillis,
    usbRecoveryMillis,
    activationMillis,
    observationMillis,
    terminalGraceMillis,
    finalCleanupMillis,
  ]);
  const parentTimeoutMillis = checkedAdd([childMaximumMillis, processTerminationMillis]);
  const fixtureDurationMillis = checkedAdd([parentTimeoutMillis, fixtureStopMarginMillis]);
  if (fixtureDurationMillis % 1_000 !== 0) {
    throw new Error("fixture duration must resolve to whole seconds");
  }
  return {
    versionProbeMillis,
    usbCommandCount,
    usbCommandAttemptCount,
    usbCommandMillis,
    usbRetryRecoveryMillis,
    usbRecoveryMillis,
    activationMillis,
    observationMillis,
    terminalGraceMillis,
    finalCleanupMillis,
    processTerminationMillis,
    fixtureStopMarginMillis,
    childMaximumMillis,
    parentTimeoutMillis,
    fixtureDurationSeconds: fixtureDurationMillis / 1_000,
    fixtureTimeoutMillis: checkedAdd([fixtureDurationMillis, processTerminationMillis]),
  };
}

export const COMMAND_EFFECTS_TRANSACTION_BUDGET = deriveCommandEffectsTransactionBudget(
  productionComponents,
);
