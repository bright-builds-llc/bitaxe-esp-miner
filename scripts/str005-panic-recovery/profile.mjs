/** Current recovery after the idle-review panic on the installed step-5 image (heartbeat005, boot 15). */
export const ENABLED = true;
export const PANIC_RECOVERY = Object.freeze({
  enabled: ENABLED,
  task: 'task-str005-start-panic-diagnosis',
  contract: 'docs/hardware/str005-idle-panic-capture-amendment.md',
  lines: ['Idle panic recovery hardware: enabled.'],
  // Heartbeat002: the last sealed Start on this image, supplying the attempt ID and Gate assets.
  seal: 'aad964919c38207cd5890011fad2599c9063a5b8f0c551f6523e53c9e361fd83',
  rootOption: '--predecessor-root',
  // The origin that holds the Ultra 205 Web Serial grant; another port would show the chooser.
  port: 48765,
  failedBootOrdinal: 15,
  contextSchema: 'str005-heartbeat-shutdown-context-v1',
  admitResult: result => result.schema === 'str005-heartbeat-result-v1' && result.complete === false,
  identity: { firmware_commit: '654338d0101521490d90330c5a4a10e5ec32e5c2',
    app_elf_sha256: '2641c24fc3f4fc3a80a4bcb8bfd389b2d70d3e048d9e18a5771b14309b588193', gate_commit: '86fc62d7a9d75da1affa2d51bc3b9eab41d86031' },
});
