/** Immutable SDK-selected emulator download identities, not a newest-version lookup. */
export const EMULATOR = Object.freeze({
  schema: 'bitaxe-virtual-emulator-lock-v1', sdk: 'v5.5.4', tool: 'qemu-xtensa',
  version: 'esp_develop_9.2.2_20250817',
  platforms: Object.freeze({
    'darwin-arm64': { platform: 'macos-arm64', sha256: 'aa92e337461d482f5d9f31cd8efc0bd67b3de8fcfcfb567289cb43a59c184651', size: 3882404 },
    'darwin-x64': { platform: 'macos', sha256: '00b9dbc2124cf7633cb86f264fbc524226ad4001bce68bbdba43c9bdc4eb026e', size: 4092932 },
    'linux-arm64': { platform: 'linux-arm64', sha256: '317f6e0fd1dba0886d8110709823d909593ef29438822a14f81ebe19d72ce7cd', size: 15123084 },
    'linux-x64': { platform: 'linux-amd64', sha256: '588bfaccd0f929650655d10a580f020c6ba9c131712d8fa519280081b8d126eb', size: 15648448 },
  }),
});

/** Resolve only the exact recommended release from the pinned local SDK manifest. */
export function selectRelease(manifest, platform = process.platform, arch = process.arch) {
  const pinned = EMULATOR.platforms[`${platform}-${arch}`];
  if (!pinned) throw Error('emulator_platform_unsupported');
  const tools = manifest.tools.filter(tool => tool.name === EMULATOR.tool);
  if (tools.length !== 1) throw Error('emulator_manifest_tool');
  const versions = tools[0].versions.filter(version => version.name === EMULATOR.version && version.status === 'recommended');
  if (versions.length !== 1) throw Error('emulator_manifest_version');
  const selected = versions[0][pinned.platform];
  if (!selected || selected.sha256 !== pinned.sha256 || selected.size !== pinned.size ||
      !selected.url.startsWith('https://github.com/espressif/qemu/releases/download/esp-develop-9.2.2-20250817/')) {
    throw Error('emulator_manifest_digest');
  }
  return { ...selected, platform: pinned.platform };
}
