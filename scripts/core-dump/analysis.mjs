/** Offline bounded debugger commands; never print locals, arguments, memory, or connect remotely. */
export function analysisArguments(elf, core) {
  return ['--nx', '--batch', '-iex', 'set print frame-arguments none', '-iex', 'set pagination off',
    '-iex', 'set auto-load python-scripts off', '-iex', 'set debuginfod enabled off', elf, `--core=${core}`,
    '-ex', 'thread apply all bt 40'];
}
