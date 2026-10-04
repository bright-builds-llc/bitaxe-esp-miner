//! Pure update rule for the control owner's core-dump stack trace record.

/// Committed last, so a reader can tell a complete record from a torn one.
pub const MAGIC: u32 = 0x4353_5431;
pub const WORDS: usize = 8;
const COMMANDS: usize = 1;
const MIN_FREE: usize = 2;
const MIN_AT: usize = 3;
const LAST_FREE: usize = 4;
const CHECKS: usize = 5;
const STACK_BYTES: usize = 6;
const VERSION: usize = 7;

/// Folds one completed command into the record.
///
/// `free_bytes` is the FreeRTOS high-water mark, the least free stack since the
/// task started, so the minimum only moves when a command went deeper.
pub fn record_command(
    previous: [u32; WORDS],
    free_bytes: u32,
    stack_bytes: u32,
    integrity_checks: u32,
) -> [u32; WORDS] {
    let fresh = previous[0] != MAGIC;
    let commands = if fresh {
        1
    } else {
        previous[COMMANDS].saturating_add(1)
    };
    let deeper = fresh || free_bytes < previous[MIN_FREE];
    let mut next = [0; WORDS];
    next[0] = MAGIC;
    next[COMMANDS] = commands;
    next[MIN_FREE] = if deeper {
        free_bytes
    } else {
        previous[MIN_FREE]
    };
    next[MIN_AT] = if deeper { commands } else { previous[MIN_AT] };
    next[LAST_FREE] = free_bytes;
    next[CHECKS] = integrity_checks;
    next[STACK_BYTES] = stack_bytes;
    next[VERSION] = 1;
    next
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_command_starts_the_record() {
        // Arrange
        let empty = [0; WORDS];
        // Act
        let record = record_command(empty, 5_000, 16_384, 2);
        // Assert
        assert_eq!(record, [MAGIC, 1, 5_000, 1, 5_000, 2, 16_384, 1]);
    }

    #[test]
    fn a_deeper_command_moves_the_minimum_and_its_index() {
        // Arrange
        let first = record_command([0; WORDS], 5_000, 16_384, 2);
        // Act
        let record = record_command(first, 900, 16_384, 4);
        // Assert
        assert_eq!(
            (record[MIN_FREE], record[MIN_AT], record[COMMANDS]),
            (900, 2, 2)
        );
    }

    #[test]
    fn a_shallower_command_keeps_the_minimum() {
        // Arrange
        let deep = record_command(record_command([0; WORDS], 900, 16_384, 2), 900, 16_384, 4);
        // Act
        let record = record_command(deep, 4_000, 16_384, 6);
        // Assert
        assert_eq!(
            (record[MIN_FREE], record[MIN_AT], record[LAST_FREE]),
            (900, 1, 4_000)
        );
    }

    #[test]
    fn a_torn_record_restarts_rather_than_trusting_stale_words() {
        // Arrange
        let torn = [0, 7, 1, 3, 1, 9, 16_384, 1];
        // Act
        let record = record_command(torn, 3_000, 16_384, 1);
        // Assert
        assert_eq!((record[COMMANDS], record[MIN_FREE]), (1, 3_000));
    }
}
