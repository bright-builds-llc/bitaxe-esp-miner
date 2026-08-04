use super::ScoreboardEntry;

/// Exact upstream scoreboard capacity.
pub const MAX_SCOREBOARD_ENTRIES: usize = 20;
const MAX_TEXT_FIELD_BYTES: usize = 31;
const MAX_PERSISTED_BYTES: usize = 127;

/// Closed failures admitted by the pure scoreboard contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScoreboardError {
    InvalidDifficulty,
    InvalidJobId,
    InvalidExtranonce2,
    MalformedPersistence,
    PersistenceTooLong,
}

/// Result of adding one valid-nonce scoreboard entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScoreboardMutation {
    IgnoredNotBetter,
    Inserted { index: usize },
}

/// Failure from validating a candidate or confirming its persistence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScoreboardOwnerError<E> {
    InvalidEntry(ScoreboardError),
    Persistence(E),
}

/// Bounded scoreboard sorted by descending nonce difficulty.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Scoreboard {
    entries: Vec<ScoreboardEntry>,
}

impl Scoreboard {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn from_entries(
        entries: impl IntoIterator<Item = ScoreboardEntry>,
    ) -> Result<Self, ScoreboardError> {
        let mut scoreboard = Self::new();
        for entry in entries {
            let _mutation = scoreboard.insert(entry)?;
        }
        Ok(scoreboard)
    }

    #[must_use]
    pub fn entries(&self) -> &[ScoreboardEntry] {
        &self.entries
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn insert(
        &mut self,
        entry: ScoreboardEntry,
    ) -> Result<ScoreboardMutation, ScoreboardError> {
        validate_entry(&entry)?;
        if self.entries.len() == MAX_SCOREBOARD_ENTRIES
            && self
                .entries
                .last()
                .is_some_and(|last| entry.difficulty <= last.difficulty)
        {
            return Ok(ScoreboardMutation::IgnoredNotBetter);
        }

        let index = self
            .entries
            .iter()
            .position(|existing| entry.difficulty > existing.difficulty)
            .unwrap_or(self.entries.len());
        self.entries.insert(index, entry);
        self.entries.truncate(MAX_SCOREBOARD_ENTRIES);
        Ok(ScoreboardMutation::Inserted { index })
    }

    /// Projects the exact values recoverable from the upstream persistence codec.
    pub fn persisted_projection(&self) -> Result<Self, ScoreboardError> {
        Self::from_entries(
            self.entries
                .iter()
                .map(ScoreboardEntry::to_persisted)
                .map(|encoded| encoded.and_then(|value| ScoreboardEntry::from_persisted(&value)))
                .collect::<Result<Vec<_>, _>>()?,
        )
    }
}

/// Transactional owner that publishes only persistence-confirmed mutations.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ScoreboardOwner {
    confirmed: Scoreboard,
}

impl ScoreboardOwner {
    #[must_use]
    pub const fn new(confirmed: Scoreboard) -> Self {
        Self { confirmed }
    }

    #[must_use]
    pub fn entries(&self) -> &[ScoreboardEntry] {
        self.confirmed.entries()
    }

    pub fn record_with<E>(
        &mut self,
        entry: ScoreboardEntry,
        persist_and_confirm: impl FnOnce(&Scoreboard, usize) -> Result<(), E>,
    ) -> Result<ScoreboardMutation, ScoreboardOwnerError<E>> {
        let mut candidate = self.confirmed.clone();
        let mutation = candidate
            .insert(entry)
            .map_err(ScoreboardOwnerError::InvalidEntry)?;
        let ScoreboardMutation::Inserted { index } = mutation else {
            return Ok(mutation);
        };

        persist_and_confirm(&candidate, index).map_err(ScoreboardOwnerError::Persistence)?;
        self.confirmed = candidate;
        Ok(mutation)
    }
}

pub(super) fn parse_persisted_entry(value: &str) -> Result<ScoreboardEntry, ScoreboardError> {
    if value.len() > MAX_PERSISTED_BYTES {
        return Err(ScoreboardError::PersistenceTooLong);
    }
    let mut fields = value.split(';');
    let entry = ScoreboardEntry::new(
        parse_field(&mut fields)?
            .parse()
            .map_err(|_| ScoreboardError::MalformedPersistence)?,
        parse_field(&mut fields)?,
        parse_field(&mut fields)?,
        parse_field(&mut fields)?
            .parse()
            .map_err(|_| ScoreboardError::MalformedPersistence)?,
        parse_field(&mut fields)?
            .parse()
            .map_err(|_| ScoreboardError::MalformedPersistence)?,
        parse_field(&mut fields)?
            .parse()
            .map_err(|_| ScoreboardError::MalformedPersistence)?,
    );
    if fields.next().is_some() {
        return Err(ScoreboardError::MalformedPersistence);
    }
    validate_entry(&entry)?;
    Ok(entry)
}

pub(super) fn format_persisted_entry(entry: &ScoreboardEntry) -> Result<String, ScoreboardError> {
    validate_entry(entry)?;
    let value = format!(
        "{:.1};{};{};{};{};{}",
        entry.difficulty,
        entry.job_id,
        entry.extranonce2,
        entry.ntime,
        entry.nonce,
        entry.version_bits
    );
    if value.len() > MAX_PERSISTED_BYTES {
        return Err(ScoreboardError::PersistenceTooLong);
    }
    Ok(value)
}

fn parse_field<'a>(fields: &mut impl Iterator<Item = &'a str>) -> Result<&'a str, ScoreboardError> {
    fields
        .next()
        .filter(|field| !field.is_empty())
        .ok_or(ScoreboardError::MalformedPersistence)
}

fn validate_entry(entry: &ScoreboardEntry) -> Result<(), ScoreboardError> {
    if !entry.difficulty.is_finite() || entry.difficulty <= 0.0 {
        return Err(ScoreboardError::InvalidDifficulty);
    }
    validate_text(&entry.job_id, ScoreboardError::InvalidJobId)?;
    validate_text(&entry.extranonce2, ScoreboardError::InvalidExtranonce2)
}

fn validate_text(value: &str, failure: ScoreboardError) -> Result<(), ScoreboardError> {
    if value.is_empty() || value.len() > MAX_TEXT_FIELD_BYTES || value.contains(';') {
        return Err(failure);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(difficulty: f64, suffix: usize) -> ScoreboardEntry {
        ScoreboardEntry::new(
            difficulty,
            format!("job-{suffix}"),
            format!("en2-{suffix}"),
            suffix as u32,
            suffix as u32 + 10,
            suffix as u32 + 20,
        )
    }

    #[test]
    fn persistence_round_trip_matches_upstream_decimal_shape() {
        // Arrange
        let entry = ScoreboardEntry::new(42.54, "job", "abcdef", 7, 8, 9);

        // Act
        let encoded = entry.to_persisted().expect("entry should encode");
        let decoded = ScoreboardEntry::from_persisted(&encoded).expect("entry should decode");

        // Assert
        assert_eq!(encoded, "42.5;job;abcdef;7;8;9");
        assert_eq!(
            decoded,
            ScoreboardEntry::new(42.5, "job", "abcdef", 7, 8, 9)
        );
    }

    #[test]
    fn persisted_projection_rounds_only_the_durable_difficulty() {
        // Arrange
        let scoreboard =
            Scoreboard::from_entries([entry(42.54, 1)]).expect("scoreboard should be valid");

        // Act
        let persisted = scoreboard
            .persisted_projection()
            .expect("projection should round trip");

        // Assert
        assert_eq!(scoreboard.entries()[0].difficulty, 42.54);
        assert_eq!(persisted.entries()[0].difficulty, 42.5);
        assert_eq!(
            persisted.entries()[0].job_id,
            scoreboard.entries()[0].job_id
        );
    }

    #[test]
    fn malformed_and_oversized_fields_fail_closed() {
        // Arrange
        let oversized = "x".repeat(MAX_TEXT_FIELD_BYTES + 1);

        // Act
        let malformed = ScoreboardEntry::from_persisted("1.0;job;en2;1;2");
        let invalid_job = entry(1.0, 1)
            .to_persisted()
            .and_then(|_| ScoreboardEntry::new(1.0, oversized, "en2", 1, 2, 3).to_persisted());
        let invalid_extranonce =
            ScoreboardEntry::new(1.0, "job", "bad;en2", 1, 2, 3).to_persisted();

        // Assert
        assert_eq!(malformed, Err(ScoreboardError::MalformedPersistence));
        assert_eq!(invalid_job, Err(ScoreboardError::InvalidJobId));
        assert_eq!(invalid_extranonce, Err(ScoreboardError::InvalidExtranonce2));
    }

    #[test]
    fn insertion_is_descending_and_stable_for_equal_difficulty() {
        // Arrange
        let mut scoreboard = Scoreboard::new();

        // Act
        scoreboard.insert(entry(10.0, 1)).expect("insert");
        scoreboard.insert(entry(20.0, 2)).expect("insert");
        scoreboard.insert(entry(20.0, 3)).expect("insert");
        scoreboard.insert(entry(15.0, 4)).expect("insert");

        // Assert
        let jobs: Vec<&str> = scoreboard
            .entries()
            .iter()
            .map(|entry| entry.job_id.as_str())
            .collect();
        assert_eq!(jobs, ["job-2", "job-3", "job-4", "job-1"]);
    }

    #[test]
    fn full_scoreboard_ignores_last_or_worse_and_evicts_for_better() {
        // Arrange
        let mut scoreboard = Scoreboard::new();
        for suffix in 0..MAX_SCOREBOARD_ENTRIES {
            scoreboard
                .insert(entry((MAX_SCOREBOARD_ENTRIES - suffix) as f64, suffix))
                .expect("initial insert");
        }

        // Act
        let equal_last = scoreboard.insert(entry(1.0, 30)).expect("ignore");
        let worse = scoreboard.insert(entry(0.5, 31)).expect("ignore");
        let better = scoreboard.insert(entry(10.5, 32)).expect("insert");

        // Assert
        assert_eq!(equal_last, ScoreboardMutation::IgnoredNotBetter);
        assert_eq!(worse, ScoreboardMutation::IgnoredNotBetter);
        assert_eq!(better, ScoreboardMutation::Inserted { index: 10 });
        assert_eq!(scoreboard.len(), MAX_SCOREBOARD_ENTRIES);
        assert_eq!(scoreboard.entries()[10].job_id, "job-32");
        assert_eq!(
            scoreboard.entries().last().map(|entry| entry.difficulty),
            Some(2.0)
        );
    }

    #[test]
    fn mutation_reports_first_changed_suffix() {
        // Arrange
        let mut scoreboard =
            Scoreboard::from_entries([entry(30.0, 1), entry(20.0, 2), entry(10.0, 3)])
                .expect("initial scoreboard");

        // Act
        let mutation = scoreboard.insert(entry(25.0, 4));

        // Assert
        assert_eq!(mutation, Ok(ScoreboardMutation::Inserted { index: 1 }));
        assert_eq!(scoreboard.entries()[1].job_id, "job-4");
    }

    #[test]
    fn failed_persistence_does_not_publish_candidate() {
        // Arrange
        let initial = Scoreboard::from_entries([entry(10.0, 1)]).expect("initial scoreboard");
        let mut owner = ScoreboardOwner::new(initial.clone());

        // Act
        let result = owner.record_with(entry(20.0, 2), |_candidate, _index| Err("write_failed"));

        // Assert
        assert_eq!(
            result,
            Err(ScoreboardOwnerError::Persistence("write_failed"))
        );
        assert_eq!(owner.entries(), initial.entries());
    }

    #[test]
    fn ignored_candidate_does_not_open_persistence() {
        // Arrange
        let mut scoreboard = Scoreboard::new();
        for suffix in 0..MAX_SCOREBOARD_ENTRIES {
            scoreboard
                .insert(entry((MAX_SCOREBOARD_ENTRIES - suffix) as f64, suffix))
                .expect("initial insert");
        }
        let mut owner = ScoreboardOwner::new(scoreboard);
        let mut persistence_called = false;

        // Act
        let result: Result<_, ScoreboardOwnerError<()>> =
            owner.record_with(entry(1.0, 30), |_candidate, _index| {
                persistence_called = true;
                Ok(())
            });

        // Assert
        assert_eq!(result, Ok(ScoreboardMutation::IgnoredNotBetter));
        assert!(!persistence_called);
    }
}
