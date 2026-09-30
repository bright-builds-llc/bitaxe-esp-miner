/** Decode only the retained fixed ABI; entered records do not contain measured facts. */
export function parseCheckpoints(raw) {
  if (raw.length !== 384) throw Error('checkpoint_length');
  const records = [];
  for (let offset = 0; offset < raw.length; offset += 48) {
    const words = Array.from({ length: 12 }, (_, index) => raw.readUInt32LE(offset + index * 4));
    if (!words[0]) continue;
    if (words[0] !== 0x56484331 || words[1] < 1 || words[1] > 4 || ![1, 2].includes(words[2])) throw Error('checkpoint_format');
    const complete = words[2] === 2;
    records.push({ phase: words[1], stage: words[2], integrity: complete ? words[3] === 1 : null,
      core: complete ? words[4] : null, stack_bytes: complete ? words[6] : null,
      stack_pointer_inside: complete ? words[11] === 1 : null,
      stack_free_at_checkpoint: complete ? words[7] - words[5] : null,
      stack_low_water: complete ? words[10] : null, internal_free: complete ? words[8] : null,
      internal_largest: complete ? words[9] : null });
  }
  return records;
}
