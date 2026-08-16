import type { AutomationCommand } from "./contracts.generated.js";

type FlagRule = {
  readonly kind: "value" | "boolean";
  readonly required?: true;
  readonly values?: readonly string[];
  readonly positiveInteger?: true;
};

type CommandRule = Readonly<Record<string, FlagRule>>;

const value = (options: Omit<FlagRule, "kind"> = {}): FlagRule => ({ kind: "value", ...options });
const boolean = (): FlagRule => ({ kind: "boolean" });

const flashCommon: CommandRule = {
  "--board": value({ values: ["205"] }),
  "--port": value(),
  "--dry-run": boolean(),
  "--redact-evidence": boolean(),
  "--evidence-dir": value(),
  "--capture-timeout-seconds": value({ positiveInteger: true }),
};

const rules: Record<AutomationCommand, CommandRule> = {
  doctor: {},
  "bootstrap-esp": {},
  "build-firmware": {
    "--output-dir": value({ required: true }),
    "--build-provenance-stamp": value({ required: true }),
    "--identity-sdkconfig-defaults": value({ required: true }),
    "--build-timestamp-utc": value({ required: true }),
    "--build-mode": value({ required: true, values: ["normal", "rollback-probe"] }),
  },
  "package-rollback-probe": {
    "--firmware-elf": value({ required: true }),
    "--build-provenance-stamp": value({ required: true }),
    "--output-image": value({ required: true }),
    "--metadata": value({ required: true }),
  },
  "package-firmware": {
    "--firmware-elf": value({ required: true }),
    "--build-provenance-stamp": value({ required: true }),
    "--esp-idf-sdkconfig": value({ required: true }),
    "--bootloader-bin": value({ required: true }),
    "--partition-table-bin": value({ required: true }),
    "--otadata-initial-bin": value({ required: true }),
    "--out-dir": value({ required: true }),
    "--manifest": value({ required: true }),
  },
  "verify-reference": {},
  "verify-redaction": {
    "--evidence-root": value(),
  },
  "verify-production-session": {},
  "observe-serial": flashCommon,
  "verify-flash-durability": {
    ...flashCommon,
    "--image": value(),
    "--manifest": value(),
    "--wifi-credentials": value(),
  },
  "verify-firmware-ota": { "--request": value({ required: true }) },
  "verify-web-assets-ota": { "--request": value({ required: true }) },
  "verify-recovery": { "--request": value({ required: true }) },
  "verify-http-api": {
    "--device-url": value({ required: true }),
    "--output": value({ required: true }),
    "--route": value(),
  },
  "verify-hardware-surface": {
    "--surface": value({
      required: true,
      values: ["power", "voltage", "thermal", "fan", "watchdog", "display", "telemetry", "failure-paths"],
    }),
    "--request": value({ required: true }),
  },
  "verify-mining": { "--request": value({ required: true }) },
  "capture-operator-evidence": {
    "--profile": value({ required: true, values: ["release"] }),
    "--evidence-root": value({ required: true }),
    "--require-redaction-passed": boolean(),
    "--require-operator-snapshot-coherence": boolean(),
  },
  "verify-settings-durability": {
    "--trace": value(),
    "--mode": value({ required: true, values: ["baseline", "delivery", "post-restart", "capture"] }),
    "--start-byte": value({ positiveInteger: true }),
    "--expected-session": value(),
    "--expected-ordinal": value({ positiveInteger: true }),
    "--private-root": value(),
    "--package-manifest": value(),
    "--wifi-credentials": value(),
    "--detector-output": value(),
    "--projection": value(),
    "--capture-timeout-seconds": value({ positiveInteger: true }),
  },
  "api-command-effects-campaign": {
    "--private-root": value({ required: true }),
    "--package-manifest": value({ required: true }),
    "--wifi-credentials": value({ required: true }),
    "--detector-output": value({ required: true }),
    "--projection": value({ required: true }),
    "--duration-seconds": value({ required: true, values: ["600"] }),
  },
  "verify-theme-durability": {
    "--private-root": value({ required: true }),
    "--package-manifest": value({ required: true }),
    "--wifi-credentials": value({ required: true }),
    "--detector-output": value({ required: true }),
    "--projection": value({ required: true }),
    "--capture-timeout-seconds": value({ required: true, positiveInteger: true }),
  },
  "capture-correlated-runtime-evidence": {
    "--root": value({ required: true }),
    "--staging": value({ required: true }),
  },
  "capture-version-evidence": {
    "--private-root": value({ required: true }),
    "--package-manifest": value({ required: true }),
    "--wifi-credentials": value({ required: true }),
    "--port": value(),
    "--detector-output": value(),
    "--projection": value({ required: true }),
    "--capture-timeout-seconds": value({ required: true, positiveInteger: true }),
  },
  "capture-operator-snapshot-evidence": {
    "--private-root": value({ required: true }),
    "--package-manifest": value({ required: true }),
    "--wifi-credentials": value({ required: true }),
    "--port": value(),
    "--detector-output": value(),
    "--projection": value({ required: true }),
    "--capture-timeout-seconds": value({ required: true, positiveInteger: true }),
  },
  "capture-runtime-health-evidence": {
    "--private-root": value({ required: true }),
    "--package-manifest": value({ required: true }),
    "--wifi-credentials": value({ required: true }),
    "--detector-output": value({ required: true }),
    "--projection": value({ required: true }),
    "--capture-timeout-seconds": value({ required: true, positiveInteger: true }),
  },
  "capture-system-info-evidence": {
    "--private-root": value({ required: true }),
    "--package-manifest": value({ required: true }),
    "--wifi-credentials": value({ required: true }),
    "--detector-output": value({ required: true }),
    "--projection": value({ required: true }),
    "--capture-timeout-seconds": value({ required: true, positiveInteger: true }),
  },
  "capture-adc-observation-evidence": {
    "--private-root": value({ required: true }),
    "--package-manifest": value({ required: true }),
    "--wifi-credentials": value({ required: true }),
    "--detector-output": value({ required: true }),
    "--projection": value({ required: true }),
    "--capture-timeout-seconds": value({ required: true, positiveInteger: true }),
  },
  "capture-emc2101-thermal-evidence": {
    "--private-root": value({ required: true }),
    "--package-manifest": value({ required: true }),
    "--wifi-credentials": value({ required: true }),
    "--detector-output": value({ required: true }),
    "--projection": value({ required: true }),
    "--capture-timeout-seconds": value({ required: true, positiveInteger: true }),
  },
  "capture-emc2101-thermal-fault-evidence": {
    "--private-root": value({ required: true }),
    "--package-manifest": value({ required: true }),
    "--wifi-credentials": value({ required: true }),
    "--detector-output": value({ required: true }),
    "--projection": value({ required: true }),
    "--capture-timeout-seconds": value({ required: true, positiveInteger: true }),
  },
  "capture-ultra205-defaults-evidence": {
    "--private-root": value({ required: true }),
    "--package-manifest": value({ required: true }),
    "--wifi-credentials": value({ required: true }),
    "--detector-output": value({ required: true }),
    "--projection": value({ required: true }),
    "--capture-timeout-seconds": value({ required: true, positiveInteger: true }),
  },
  "capture-settings-patch-evidence": {
    "--private-root": value({ required: true }),
    "--package-manifest": value({ required: true }),
    "--wifi-credentials": value({ required: true }),
    "--detector-output": value({ required: true }),
    "--projection": value({ required: true }),
    "--capture-timeout-seconds": value({ required: true, positiveInteger: true }),
  },
  "capture-log-buffer-evidence": {
    "--private-root": value({ required: true }),
    "--package-manifest": value({ required: true }),
    "--wifi-credentials": value({ required: true }),
    "--detector-output": value({ required: true }),
    "--projection": value({ required: true }),
    "--capture-timeout-seconds": value({ required: true, positiveInteger: true }),
  },
  "capture-partition-layout-evidence": {
    "--private-root": value({ required: true }),
    "--package-manifest": value({ required: true }),
    "--wifi-credentials": value({ required: true }),
    "--detector-output": value({ required: true }),
    "--projection": value({ required: true }),
    "--capture-timeout-seconds": value({ required: true, positiveInteger: true }),
  },
  "capture-sdkconfig-rollback-evidence": {
    "--private-root": value({ required: true }),
    "--package-manifest": value({ required: true }),
    "--rollback-probe-image": value({ required: true }),
    "--rollback-probe-metadata": value({ required: true }),
    "--wifi-credentials": value({ required: true }),
    "--detector-output": value({ required: true }),
    "--projection": value({ required: true }),
    "--capture-timeout-seconds": value({ required: true, positiveInteger: true }),
  },
  "capture-network-reconnect-evidence": {
    "--private-root": value({ required: true }),
    "--package-manifest": value({ required: true }),
    "--wifi-credentials": value({ required: true }),
    "--detector-output": value({ required: true }),
    "--projection": value({ required: true }),
    "--capture-timeout-seconds": value({ required: true, positiveInteger: true }),
  },
  "capture-network-scan-evidence": {
    "--private-root": value({ required: true }),
    "--package-manifest": value({ required: true }),
    "--wifi-credentials": value({ required: true }),
    "--detector-output": value({ required: true }),
    "--projection": value({ required: true }),
    "--capture-timeout-seconds": value({ required: true, positiveInteger: true }),
  },
  "project-asic-initialization-evidence": {
    "--attempt-root": value({ required: true }),
    "--attempt-source-commit": value({ required: true }),
    "--projection": value({ required: true }),
  },
  "project-asic-power-initialization-evidence": {
    "--source-projection": value({ required: true }),
    "--attempt-source-commit": value({ required: true }),
    "--projection": value({ required: true }),
  },
  "project-core-voltage-control-evidence": {
    "--source-projection": value({ required: true }),
    "--attempt-source-commit": value({ required: true }),
    "--projection": value({ required: true }),
  },
  "project-ina260-evidence": {
    "--attempt-root": value({ required: true }),
    "--source-projection": value({ required: true }),
    "--attempt-source-commit": value({ required: true }),
    "--projection": value({ required: true }),
  },
  "project-asic-reset-evidence": {
    "--source-projection": value({ required: true }),
    "--attempt-source-commit": value({ required: true }),
    "--projection": value({ required: true }),
  },
  "project-asic-frequency-transition-evidence": {
    "--source-projection": value({ required: true }),
    "--attempt-source-commit": value({ required: true }),
    "--projection": value({ required: true }),
  },
  "project-stratum-socket-evidence": {
    "--source-projection": value({ required: true }),
    "--attempt-source-commit": value({ required: true }),
    "--projection": value({ required: true }),
  },
  "project-protocol-coordinator-evidence": {
    "--initialization-projection": value({ required: true }),
    "--work-send-projection": value({ required: true }),
    "--result-parsing-projection": value({ required: true }),
    "--socket-projection": value({ required: true }),
    "--attempt-source-commit": value({ required: true }),
    "--projection": value({ required: true }),
  },
  "project-mining-criteria-evidence": {
    "--summary": value({ required: true }),
    "--smoke": value({ required: true }),
    "--soak": value({ required: true }),
    "--coordinator-projection": value({ required: true }),
    "--projection": value({ required: true }),
  },
  "project-asic-work-send-evidence": {
    "--source-projection": value({ required: true }),
    "--attempt-source-commit": value({ required: true }),
    "--projection": value({ required: true }),
  },
  "project-asic-result-parsing-evidence": {
    "--source-projection": value({ required: true }),
    "--attempt-source-commit": value({ required: true }),
    "--projection": value({ required: true }),
  },
  "project-asic-serial-transport-evidence": {
    "--work-send-projection": value({ required: true }),
    "--result-parsing-projection": value({ required: true }),
    "--attempt-source-commit": value({ required: true }),
    "--projection": value({ required: true }),
  },
  "capture-provisioning-network-evidence": {
    "--private-root": value({ required: true }),
    "--package-manifest": value({ required: true }),
    "--wifi-credentials": value({ required: true }),
    "--detector-output": value({ required: true }),
    "--projection": value({ required: true }),
    "--capture-timeout-seconds": value({ required: true, positiveInteger: true }),
  },
  "project-ui-workflow-evidence": {
    "--private-root": value({ required: true }),
    "--attempt-source-commit": value({ required: true }),
    "--operator-snapshot-projection": value({ required: true }),
    "--browser-attestation": value({ required: true }),
    "--projection": value({ required: true }),
  },
};

const commands = new Set(Object.keys(rules) as AutomationCommand[]);

export class InvocationError extends Error {}

export type ParsedInvocation = {
  readonly command: AutomationCommand;
  readonly args: readonly string[];
  readonly values: ReadonlyMap<string, string | true>;
};

export function parseInvocation(argv: readonly string[]): ParsedInvocation {
  const [maybeCommand, ...args] = argv;
  if (maybeCommand === undefined || !commands.has(maybeCommand as AutomationCommand)) {
    throw new InvocationError("unknown or missing semantic subcommand");
  }
  const command = maybeCommand as AutomationCommand;
  const commandRules = rules[command];
  const values = new Map<string, string | true>();
  for (let index = 0; index < args.length; index += 1) {
    const flag = args[index];
    if (flag === undefined || !flag.startsWith("--") || flag.includes("=") || flag.includes("_")) {
      throw new InvocationError("arguments must use canonical --kebab-case flags with separate values");
    }
    const rule = commandRules[flag];
    if (rule === undefined) throw new InvocationError(`unknown option ${flag} for ${command}`);
    if (values.has(flag)) throw new InvocationError(`duplicate option ${flag}`);
    if (rule.kind === "boolean") {
      values.set(flag, true);
      continue;
    }
    const maybeValue = args[index + 1];
    if (maybeValue === undefined || maybeValue.startsWith("--")) {
      throw new InvocationError(`missing value for ${flag}`);
    }
    if (rule.values !== undefined && !rule.values.includes(maybeValue)) {
      throw new InvocationError(`invalid value ${maybeValue} for ${flag}`);
    }
    if (rule.positiveInteger === true && (!/^\d+$/u.test(maybeValue) || Number(maybeValue) <= 0)) {
      throw new InvocationError(`${flag} must be a positive integer`);
    }
    values.set(flag, maybeValue);
    index += 1;
  }
  for (const [flag, rule] of Object.entries(commandRules)) {
    if (rule.required === true && !values.has(flag)) {
      throw new InvocationError(`missing required option ${flag}`);
    }
  }
  if (command === "verify-flash-durability" && values.has("--image") && !values.has("--manifest")) {
    throw new InvocationError("--image requires --manifest");
  }
  if (
    command === "capture-version-evidence"
    && values.has("--port") === values.has("--detector-output")
  ) {
    throw new InvocationError("capture-version-evidence requires exactly one of --port or --detector-output");
  }
  if (
    command === "capture-operator-snapshot-evidence"
    && values.has("--port") === values.has("--detector-output")
  ) {
    throw new InvocationError(
      "capture-operator-snapshot-evidence requires exactly one of --port or --detector-output",
    );
  }
  if (command === "verify-settings-durability") {
    const capture = values.get("--mode") === "capture";
    const captureFlags = [
      "--private-root", "--package-manifest", "--wifi-credentials", "--detector-output",
      "--projection", "--capture-timeout-seconds",
    ];
    if (capture) {
      for (const flag of captureFlags) {
        if (!values.has(flag)) throw new InvocationError(`capture mode requires ${flag}`);
      }
      if (values.has("--trace") || values.has("--start-byte") || values.has("--expected-session") || values.has("--expected-ordinal")) {
        throw new InvocationError("capture mode rejects trace-classification options");
      }
    } else if (!values.has("--trace")) {
      throw new InvocationError("classification mode requires --trace");
    } else if (captureFlags.some((flag) => values.has(flag))) {
      throw new InvocationError("classification mode rejects capture options");
    }
  }
  return { command, args, values };
}

export function optionValue(invocation: ParsedInvocation, flag: string): string {
  const maybeValue = invocation.values.get(flag);
  if (typeof maybeValue !== "string") throw new InvocationError(`missing required option ${flag}`);
  return maybeValue;
}

export function maybeOptionValue(invocation: ParsedInvocation, flag: string): string | undefined {
  const maybeValue = invocation.values.get(flag);
  return typeof maybeValue === "string" ? maybeValue : undefined;
}

export function hasFlag(invocation: ParsedInvocation, flag: string): boolean {
  return invocation.values.get(flag) === true;
}
