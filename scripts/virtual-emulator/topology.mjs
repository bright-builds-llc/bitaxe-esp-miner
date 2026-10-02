import { readFile, writeFile } from 'node:fs/promises';
import { join } from 'node:path';
import { managedPaths } from './setup.mjs';

/** Pinned QEMU topology; no physical transport, networking, monitor socket or credentials. */
export function qemuArguments(image, efuse) {
  return ['-M', 'esp32s3', '-smp', '2', '-m', '8M',
    '-global', 'driver=ssi_psram,property=is_octal,value=true',
    '-drive', `file=${image},if=mtd,format=raw`,
    '-drive', `file=${efuse},if=none,format=raw,id=efuse`,
    '-global', 'driver=nvram.esp32s3.efuse,property=drive,value=efuse',
    '-global', 'driver=timer.esp32s3.timg,property=wdt_disable,value=true',
    '-nographic', '-serial', 'mon:stdio', '-nic', 'none'];
}

/** Use the exact pinned SDK default virtual eFuses, never physical identities. */
export async function writeSdkEfuse(repo, evidence) {
  const tools = managedPaths(repo);
  const extension = await readFile(join(tools.idf, 'tools/idf_py_actions/qemu_ext.py'), 'utf8');
  const section = extension.slice(extension.indexOf("'esp32s3': QemuTarget("));
  const hex = section.slice(section.indexOf('binascii.unhexlify('), section.indexOf("'-global driver=esp32s3.gpio"))
    .match(/'[a-f0-9]{40,}'/g)?.map(value => value.slice(1, -1)).join('');
  if (!hex || hex.length !== 2048) throw Error('guest_sdk_efuse_shape');
  const efuse = join(evidence, 'virtual-efuse.bin');
  await writeFile(efuse, Buffer.from(hex, 'hex'), { flag: 'wx', mode: 0o600 });
  return efuse;
}
