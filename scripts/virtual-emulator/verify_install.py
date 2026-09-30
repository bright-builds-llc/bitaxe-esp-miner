#!/usr/bin/env python3
"""Verify every managed emulator file against the SDK-checksummed official archive."""
import hashlib
from pathlib import Path, PurePosixPath
import sys
import tarfile


def verify(archive: Path, install: Path, expected_sha: str, expected_size: int) -> None:
    if archive.stat().st_size != expected_size:
        raise ValueError("emulator_archive_size")
    if hashlib.sha256(archive.read_bytes()).hexdigest() != expected_sha:
        raise ValueError("emulator_archive_digest")
    with tarfile.open(archive, "r:xz") as contents:
        for member in contents:
            relative = PurePosixPath(member.name)
            if relative.is_absolute() or ".." in relative.parts:
                raise ValueError("emulator_archive_path")
            destination = install.joinpath(*relative.parts)
            if member.isdir():
                if not destination.is_dir():
                    raise ValueError("emulator_installed_directory")
            elif member.isfile():
                source = contents.extractfile(member)
                if source is None or destination.is_symlink():
                    raise ValueError("emulator_installed_file")
                if hashlib.sha256(source.read()).digest() != hashlib.sha256(destination.read_bytes()).digest():
                    raise ValueError("emulator_installed_digest")
            else:
                raise ValueError("emulator_archive_entry_unsupported")


if __name__ == "__main__":
    try:
        verify(Path(sys.argv[1]), Path(sys.argv[2]), sys.argv[3], int(sys.argv[4]))
    except (ValueError, OSError, tarfile.TarError) as error:
        print(str(error), file=sys.stderr)
        sys.exit(1)
