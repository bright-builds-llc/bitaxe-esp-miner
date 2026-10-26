// The local mining stopped proof (ADR-0038) that admits `just core-dump-read` for the local mining
// owner: within 120 s of a reset-capable read, two HTTP samples in a row show the exact installed
// firmware paused with the ASIC powered down and mining on boot off, a fresh detector names the
// physical device, and nothing holds its serial node. It reads only; it never pauses or resumes.
import { GAMMA601_LIMITS, hardwareStoppedUnder, LIMITS, sampleFromInfo, SAMPLE_INTERVAL_MS } from "./judge.mjs";
import { requireCondition } from "../fixed-usb-qualification/contract.mjs";

export const STOPPED_PROOF_SCHEMA = "local-mining-stopped-proof-v1";
/** The detector must be this fresh, like the Gate soak's preflight. */
export const DETECTOR_FRESH_MS = 60_000;
/** The limits whose idle power shows the installed package's board powered down. */
export const limitsForBoard = (maybeBoard) => (maybeBoard === "601" ? GAMMA601_LIMITS : LIMITS);

/**
 * Builds the proof from two samples `SAMPLE_INTERVAL_MS` apart. `installed` names the firmware the
 * device must run; `bootOrdinal` is the boot the same-session handoff saw over USB, which binds the
 * HTTP device to the detected one; `detector` is the parsed fresh detector; `requireReleased`
 * checks the serial node. `limits` are the installed board's: a powered-down 601 reads about 5.0 W.
 */
export async function stoppedProof({ client, installed, bootOrdinal, detector, sourceCommit, requireReleased, sleep, now,
  limits = LIMITS }) {
  const hardwareStopped = hardwareStoppedUnder(limits);
  const firstInfo = await client.info();
  const first = sampleFromInfo(firstInfo, now());
  await sleep(SAMPLE_INTERVAL_MS);
  const info = await client.info();
  const second = sampleFromInfo(info, now());
  requireCondition([firstInfo, info].every((value) => value.sourceCommit === installed.firmwareCommit &&
    value.appElfSha256 === installed.appElfSha256), "stopped_proof_identity");
  requireCondition([firstInfo, info].every((value) => value.bootOrdinal === bootOrdinal), "stopped_proof_boot_ordinal");
  requireCondition(typeof second.bootSession === "string" && /^[0-9a-f]{32}$/u.test(second.bootSession) &&
    first.bootSession === second.bootSession, "stopped_proof_boot");
  requireCondition(hardwareStopped(first) && hardwareStopped(second), "stopped_proof_not_stopped");
  requireCondition(!first.startMiningOnBoot && !second.startMiningOnBoot, "stopped_proof_mine_on_boot");
  requireReleased(detector.port);
  return { schema: STOPPED_PROOF_SCHEMA, source_commit: sourceCommit, firmware_commit: installed.firmwareCommit,
    app_elf_sha256: installed.appElfSha256, physical_identity_sha256: detector.physical, observed_at_unix_ms: now(),
    boot_session: second.bootSession, mining_paused: true, asic_stopped: true, stopped_samples: 2, mine_on_boot: false,
    serial_ownership_released: true };
}
