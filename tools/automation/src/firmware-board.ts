/** Boards a package can target (ADR-0041); selected by the Bazel package target. */
export const firmwareBoards = ["205", "601"] as const;

export type FirmwareBoard = (typeof firmwareBoards)[number];

/** File and release names each board's package uses. */
export const boardPackageNames: Readonly<Record<FirmwareBoard, {
  readonly elf: string;
  readonly factoryImage: string;
  readonly releaseName: string;
  readonly installNotes: string;
}>> = {
  "205": {
    elf: "bitaxe-ultra205.elf",
    factoryImage: "bitaxe-ultra205-factory.bin",
    releaseName: "bitaxe-ultra205",
    installNotes: "docs/release/ultra-205.md",
  },
  "601": {
    elf: "bitaxe-gamma601.elf",
    factoryImage: "bitaxe-gamma601-factory.bin",
    releaseName: "bitaxe-gamma601",
    installNotes: "docs/release/gamma-601.md",
  },
};

const marker = (board: FirmwareBoard): string => `firmware_board=${board}`;

/** The ELF disagrees with the board the package is about to record: a typed `package_invalid`. */
export class FirmwareBoardMarkerError extends Error {
  readonly category = "package_invalid" as const;
  readonly publicValue: Readonly<Record<string, unknown>>;

  constructor(readonly failure: "firmware_board_marker_missing" | "firmware_board_mismatch") {
    super(failure);
    this.name = "FirmwareBoardMarkerError";
    this.publicValue = { failure };
  }
}

/**
 * Fails closed when the ELF carries another board's marker, or a 601 ELF lacks its own.
 * A 205 ELF may lack the marker: historical commits predate it, and the flash tool requires it
 * for every package it installs.
 */
export function requireFirmwareBoardMarker(elf: Buffer, board: FirmwareBoard): void {
  const other: FirmwareBoard = board === "205" ? "601" : "205";
  if (elf.includes(marker(other))) throw new FirmwareBoardMarkerError("firmware_board_mismatch");
  if (board === "601" && !elf.includes(marker(board))) {
    throw new FirmwareBoardMarkerError("firmware_board_marker_missing");
  }
}
