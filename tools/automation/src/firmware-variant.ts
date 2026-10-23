import type { FirmwareBoard } from "./firmware-board.js";

/** Firmware variants selected by `//firmware/bitaxe:firmware_variant` (ADR-0040, ADR-0041). */
export const firmwareVariants = ["production", "overheat-trip-test", "gamma601-vout-check"] as const;

export type FirmwareVariant = (typeof firmwareVariants)[number];

type TestVariant = Exclude<FirmwareVariant, "production">;

/** Boot-line literal compiled into the ELF only by the overheat-trip-test variant. */
export const OVERHEAT_TRIP_TEST_MARKER = "firmware_variant=overheat-trip-test";
/** Boot-line literal compiled into the ELF only by the Gamma 601 core-voltage check variant. */
export const GAMMA601_VOUT_CHECK_MARKER = "firmware_variant=gamma601-vout-check";

const testVariantMarkers: Readonly<Record<TestVariant, string>> = {
  "overheat-trip-test": OVERHEAT_TRIP_TEST_MARKER,
  "gamma601-vout-check": GAMMA601_VOUT_CHECK_MARKER,
};

/** The board each test variant is built for; production builds for every board. */
const testVariantBoards: Readonly<Record<TestVariant, FirmwareBoard>> = {
  "overheat-trip-test": "205",
  "gamma601-vout-check": "601",
};

/** The ELF disagrees with the variant the package is about to record: a typed `package_invalid`. */
export class FirmwareVariantMarkerError extends Error {
  readonly category = "package_invalid" as const;
  readonly publicValue: Readonly<Record<string, unknown>>;

  constructor(
    readonly failure:
      | "firmware_variant_marker_missing"
      | "firmware_variant_marker_in_production"
      | "firmware_variant_marker_foreign"
      | "firmware_variant_board_mismatch",
  ) {
    super(failure);
    this.name = "FirmwareVariantMarkerError";
    this.publicValue = { failure };
  }
}

/**
 * Fails closed unless the ELF carries exactly the requested test variant's marker, and no
 * marker at all for production.
 */
export function requireFirmwareVariantMarker(elf: Buffer, variant: FirmwareVariant): void {
  const present = (Object.keys(testVariantMarkers) as TestVariant[])
    .filter((testVariant) => elf.includes(testVariantMarkers[testVariant]));
  if (variant === "production") {
    if (present.length > 0) throw new FirmwareVariantMarkerError("firmware_variant_marker_in_production");
    return;
  }
  if (!present.includes(variant)) throw new FirmwareVariantMarkerError("firmware_variant_marker_missing");
  if (present.length > 1) throw new FirmwareVariantMarkerError("firmware_variant_marker_foreign");
}

/** A test variant is packaged only for the board it was built for. */
export function requireFirmwareVariantBoard(variant: FirmwareVariant, board: FirmwareBoard): void {
  if (variant !== "production" && testVariantBoards[variant] !== board) {
    throw new FirmwareVariantMarkerError("firmware_variant_board_mismatch");
  }
}
