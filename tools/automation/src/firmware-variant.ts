/** Firmware variants selected by `//firmware/bitaxe:firmware_variant` (ADR-0040). */
export const firmwareVariants = ["production", "overheat-trip-test"] as const;

export type FirmwareVariant = (typeof firmwareVariants)[number];

/** Boot-line literal compiled into the ELF only by the overheat-trip-test variant. */
export const OVERHEAT_TRIP_TEST_MARKER = "firmware_variant=overheat-trip-test";

/** The ELF disagrees with the variant the package is about to record: a typed `package_invalid`. */
export class FirmwareVariantMarkerError extends Error {
  readonly category = "package_invalid" as const;
  readonly publicValue: Readonly<Record<string, unknown>>;

  constructor(readonly failure: "firmware_variant_marker_missing" | "firmware_variant_marker_in_production") {
    super(failure);
    this.name = "FirmwareVariantMarkerError";
    this.publicValue = { failure };
  }
}

/** Fails closed unless the test marker is present exactly when the requested variant is the test variant. */
export function requireFirmwareVariantMarker(elf: Buffer, variant: FirmwareVariant): void {
  const hasMarker = elf.includes(OVERHEAT_TRIP_TEST_MARKER);
  if (variant === "overheat-trip-test" && !hasMarker) {
    throw new FirmwareVariantMarkerError("firmware_variant_marker_missing");
  }
  if (variant === "production" && hasMarker) {
    throw new FirmwareVariantMarkerError("firmware_variant_marker_in_production");
  }
}
