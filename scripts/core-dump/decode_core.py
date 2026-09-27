"""Offline-only adapter to the pinned Espressif raw core loader.

All output (including vendor diagnostics) belongs in a private directory.
"""
import hashlib
import importlib.metadata
import json
import os
from pathlib import Path
import re
import sys


def inspect(dump, elf, expected, verify_cutoff=False):
    from esp_coredump.corefile.loader import ESPCoreDumpFileLoader
    from esp_coredump.corefile.elf import ESPCoreDumpElfFile

    if importlib.metadata.version('esp-coredump') != '1.17.2':
        raise ValueError('decoder_version')
    if hashlib.sha256(Path(elf).read_bytes()).hexdigest() != expected:
        raise ValueError('elf_identity')
    loader = ESPCoreDumpFileLoader(dump, False)
    if loader.target != 'esp32s3' or loader.dump_ver != loader.ELF_SHA256_V2_1:
        raise ValueError('dump_format')
    loader.create_corefile(exe_name=elf)
    core = ESPCoreDumpElfFile(loader.core_elf_file)
    notes = [note for segment in core.note_segments for note in segment.note_secs
             if note.name == b'ESP_CORE_DUMP_INFO' and note.type == ESPCoreDumpElfFile.PT_ESP_INFO]
    if len(notes) != 1 or len(notes[0].desc) != 72:
        raise ValueError('dump_identity')
    # IDF 5.5.4: u32 version + char[66] + ABI tail padding = 72 bytes.
    # strlcpy terminates at byte 68; spare/padding bytes are not identity.
    if notes[0].desc[68] != 0:
        raise ValueError('dump_identity')
    identity = notes[0].desc[4:68]
    if not re.fullmatch(b'[0-9a-f]{64}', identity) or identity.decode() != expected:
        raise ValueError('dump_identity')
    result = {'schema': 'bitaxe-private-core-inspection/1', 'chip': 'esp32s3',
            'elf_sha256': expected, 'dump_sha256': hashlib.sha256(Path(dump).read_bytes()).hexdigest(),
            'decoder_version': '1.17.2', 'checksum_verified': True, 'full_elf_identity_verified': True,
            'tasks_declared': int(loader.header.task_num) if 'task_num' in loader.header else None, 'cause_proven': False}
    if verify_cutoff:
        from cutoff import verify
        result.update(verify(elf, loader.core_elf_file))
    return result


if __name__ == '__main__':
    os.umask(0o077)
    result = inspect(*sys.argv[1:4], verify_cutoff=len(sys.argv) == 6 and sys.argv[5] == "verify-cutoff")
    with open(sys.argv[4], 'x', encoding='utf-8') as output:
        json.dump(result, output, sort_keys=True)
