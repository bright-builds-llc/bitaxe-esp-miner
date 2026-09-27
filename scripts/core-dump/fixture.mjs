import { createHash } from 'node:crypto';
const hash = bytes => createHash('sha256').update(bytes).digest();
const words = values => { const data = Buffer.alloc(values.length * 4); values.forEach((value, index) => data.writeUInt32LE(value, index * 4)); return data; };
function elf(payload, note = false) {
  const header = Buffer.alloc(52); Buffer.from([0x7f, 69, 76, 70, 1, 1, 1]).copy(header);
  header.writeUInt16LE(note ? 4 : 2, 16); header.writeUInt16LE(94, 18); header.writeUInt32LE(1, 20);
  header.writeUInt32LE(52, 28); header.writeUInt16LE(52, 40); header.writeUInt16LE(32, 42); header.writeUInt16LE(1, 44);
  return Buffer.concat([header, words([note ? 4 : 1, 84, note ? 0 : 0x42000000, 0, payload.length, payload.length, 5, 4]), payload]);
}
/** Synthetic ELF and raw SHA256 V2.1 fixture; no captured device memory. */
export function fixture(kind = 'valid') {
  const program = elf(Buffer.from('synthetic-program'));
  const identity = hash(program).toString('hex');
  const name = Buffer.from((kind === 'absent' ? 'UNRELATED_PADDING' : 'ESP_CORE_DUMP_INFO') + '\0');
  const identityBytes = Buffer.alloc(64);
  Buffer.from(kind === 'wrong' ? 'f'.repeat(64) : kind === 'truncated' ? identity.slice(0, 9) : identity).copy(identityBytes);
  const version = (9 << 16) | 0x103;
  const tail = Buffer.alloc(4);
  if (kind === 'padding') tail.fill(0xa5, 1);
  if (kind === 'unterminated') tail[0] = 0x61;
  const descriptor = Buffer.concat([words([version]), identityBytes, kind === 'short-abi' ? Buffer.alloc(0) : tail]);
  const note = Buffer.concat([words([name.length, descriptor.length, 8266]), name, Buffer.alloc((4 - name.length % 4) % 4), descriptor]);
  const core = elf(kind === 'duplicate' ? Buffer.concat([note, note]) : note, true);
  const body = Buffer.concat([words([24 + core.length + 32, version, 0, 0, 0, 0]), core]);
  const dump = Buffer.concat([body, hash(body)]);
  if (kind === 'checksum') dump[dump.length - 1] ^= 1;
  return { program, dump, identity };
}
