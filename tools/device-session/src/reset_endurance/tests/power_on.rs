use super::*;

fn counting(cycles: u32) -> ResetEnduranceConfig {
    config(cycles).with_count_power_on(true)
}

#[test]
fn power_on_without_the_opt_in_still_stops() {
    // Arrange / Act
    let (run, _fake) = run_with(5, Some((3, Fault::PowerOnFresh)));

    // Assert
    let failure = run.maybe_first_failure.expect("failure");
    assert_eq!(
        (failure.cycle, failure.category),
        (3, ResetEnduranceStop::RebootNotProven)
    );
}

#[test]
fn counted_power_on_is_recorded_and_the_run_continues() {
    // Arrange / Act
    let (run, _fake) = run_faults(counting(5), vec![(3, Fault::PowerOnFresh)]);

    // Assert
    let power_on: Vec<_> = run.rows.iter().map(|row| row.power_on).collect();
    assert_eq!(
        (run.maybe_first_failure, power_on),
        (None, vec![false, false, true, false, false])
    );
}

#[test]
fn counted_power_on_requires_the_next_ordinal_to_exceed_one() {
    // Arrange / Act
    let (run, _fake) = run_faults(
        counting(5),
        vec![(3, Fault::PowerOnFresh), (4, Fault::OrdinalOneOther)],
    );

    // Assert
    let failure = run.maybe_first_failure.expect("failure");
    assert_eq!(
        (failure.cycle, failure.category),
        (4, ResetEnduranceStop::RebootNotProven)
    );
}

#[test]
fn counted_power_on_with_a_later_ordinal_still_stops() {
    // Arrange / Act
    let (run, _fake) = run_faults(counting(5), vec![(3, Fault::PowerOnOrdinalFive)]);

    // Assert
    let failure = run.maybe_first_failure.expect("failure");
    assert_eq!(
        (failure.cycle, failure.category, run.rows[2].power_on),
        (3, ResetEnduranceStop::RebootNotProven, false)
    );
}

#[test]
fn ordinal_one_with_another_reason_still_stops_with_the_opt_in() {
    // Arrange / Act
    let (run, _fake) = run_faults(counting(5), vec![(3, Fault::OrdinalOneOther)]);

    // Assert
    let failure = run.maybe_first_failure.expect("failure");
    assert_eq!(
        (failure.cycle, failure.category),
        (3, ResetEnduranceStop::RebootNotProven)
    );
}

#[test]
fn projection_reports_power_on_counts_without_the_port() {
    // Arrange
    let config = counting(6);
    let (run, _fake) = run_faults(
        config.clone(),
        vec![(2, Fault::PowerOnFresh), (5, Fault::PowerOnFresh)],
    );
    let cleanup = FinalCleanup {
        proven: true,
        maybe_detail: Some(format!("detail for {PORT}")),
    };

    // Act
    let projection = ResetEnduranceProjection::build(&config, &run, &cleanup, 1_000);
    let encoded = serde_json::to_string(&projection).expect("projection JSON");

    // Assert
    assert_eq!(
        (
            projection.schema,
            projection.count_power_on,
            projection.power_on_cycles.clone(),
            projection.power_on_count,
            projection.passed(),
        ),
        ("usb-reset-endurance-v2", true, vec![2, 5], 2, true)
    );
    assert!(!encoded.contains(PORT) && !encoded.contains(&physical()));
}

#[test]
fn projection_always_emits_empty_power_on_fields_without_the_opt_in() {
    // Arrange
    let (run, _fake) = run_with(2, None);
    let cleanup = FinalCleanup {
        proven: true,
        maybe_detail: None,
    };

    // Act
    let projection = ResetEnduranceProjection::build(&config(2), &run, &cleanup, 1_000);
    let encoded = serde_json::to_value(&projection).expect("projection JSON");

    // Assert
    assert_eq!(
        (
            &encoded["count_power_on"],
            &encoded["power_on_cycles"],
            &encoded["power_on_count"],
        ),
        (
            &serde_json::json!(false),
            &serde_json::json!([]),
            &serde_json::json!(0)
        )
    );
}
