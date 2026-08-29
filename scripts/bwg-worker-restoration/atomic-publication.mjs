import { randomBytes } from "node:crypto";
import { link, open, unlink } from "node:fs/promises";
import { basename, dirname, resolve } from "node:path";

export async function writeAtomicNew(path, value, mode, operations = {}) {
  const openFile = operations.openFile ?? open;
  const linkFile = operations.linkFile ?? link;
  const unlinkFile = operations.unlinkFile ?? unlink;
  const temporary = resolve(dirname(path), `.${basename(path)}.${randomBytes(16).toString("hex")}`);
  let handle;
  let maybeWriteError;
  try {
    handle = await openFile(temporary, "wx", mode);
    await handle.writeFile(value);
    await handle.sync();
  } catch (error) {
    maybeWriteError = error;
  }
  if (handle) {
    try {
      await handle.close();
    } catch (error) {
      maybeWriteError ??= error;
    }
  }
  if (maybeWriteError) {
    try {
      await unlinkFile(temporary);
    } catch (cleanupError) {
      if (cleanupError?.code !== "ENOENT") {
        throw new AggregateError(
          [maybeWriteError, cleanupError],
          "atomic_publication_cleanup_failed",
        );
      }
    }
    throw maybeWriteError;
  }
  let targetLinked = false;
  try {
    await linkFile(temporary, path);
    targetLinked = true;
    await unlinkFile(temporary);
  } catch (error) {
    const cleanupErrors = [];
    if (targetLinked) {
      try {
        await unlinkFile(path);
      } catch (cleanupError) {
        cleanupErrors.push(cleanupError);
      }
    }
    try {
      await unlinkFile(temporary);
    } catch (cleanupError) {
      if (cleanupError?.code !== "ENOENT") cleanupErrors.push(cleanupError);
    }
    if (cleanupErrors.length > 0) {
      throw new AggregateError([error, ...cleanupErrors], "atomic_publication_cleanup_failed");
    }
    throw error;
  }
}
