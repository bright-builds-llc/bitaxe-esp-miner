import { readFile, stat } from 'node:fs/promises';
import { protectedPath } from '../fixed-usb-qualification/contract.mjs';
import { parseDetector } from '../str005-panic-probe/detector.mjs';
import { check } from '../str005-v2-serial/values.mjs';

/** Reads the pre-Start detector the operator writes beside the attempt root; a missing file is a named failure. */
export async function readStartupDetector(path, physical) {
  try { await protectedPath(path); } catch (error) { check(error.code !== 'ENOENT', 'startup_detector_missing'); throw error; }
  const bytes = await readFile(path), age = Date.now() - (await stat(path)).mtimeMs;
  try { return { bytes, detector: parseDetector(bytes.toString('utf8'), physical, age) }; }
  catch (error) { if (error.code === 'panic_detector_stale') error.detectorAgeMs = Math.floor(age); throw error; }
}
