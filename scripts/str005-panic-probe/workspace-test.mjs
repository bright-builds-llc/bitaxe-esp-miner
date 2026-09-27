import { spawnSync } from 'node:child_process';
import { resolve } from 'node:path';

// Canonical artifacts are workspace outputs; the explicit local test never opens USB.
const repo = process.env.BUILD_WORKSPACE_DIRECTORY;
if (!repo) throw Error('panic_command_test_workspace_required');
const result = spawnSync(process.execPath, ['--test', resolve(repo, 'scripts/str005-panic-probe/command-check.integration.mjs')], {
  cwd: repo, env: process.env, stdio: 'inherit', timeout: 120000,
});
if (result.error || result.signal || result.status !== 0) process.exitCode = 1;
