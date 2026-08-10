use super::*;

#[test]
fn settings_persistence_exhaustive_patch_writes_once_then_confirms_before_publication() {
    // Arrange
    let case = fixture().exhaustive_valid;
    let accepted = plan_settings_patch_value(&case.body).expect("full PATCH should be accepted");
    let plan = SettingsPersistencePlan::from_accepted(&accepted);
    let expected_write_count = plan.writes().len();
    let shared = Rc::new(RefCell::new(SharedAdapterState::new("bitaxe")));
    let mut adapter = RecordingAdapter::new("writer-1", Rc::clone(&shared));

    // Act
    let success = execute_settings_persistence_plan(&plan, &mut adapter)
        .expect("every typed write must persist and reconcile");

    // Assert
    let write_count = success
        .steps()
        .iter()
        .filter(|step| matches!(step, SettingsPersistenceStep::Write { .. }))
        .count();
    assert_eq!(write_count, expected_write_count);
    assert_eq!(
        success
            .steps()
            .iter()
            .filter(|step| **step == SettingsPersistenceStep::Commit)
            .count(),
        1
    );
    assert_eq!(
        success.steps().last(),
        Some(&SettingsPersistenceStep::PublicSuccess)
    );
    assert_eq!(shared.borrow().publication_history.len(), 1);
}

#[test]
fn settings_persistence_serializes_two_writers_through_publication() {
    // Arrange
    let shared = Rc::new(RefCell::new(SharedAdapterState::new("bitaxe")));
    let first_plan = persistence_plan("writer-one");
    let second_plan = persistence_plan("writer-two");
    let mut first = RecordingAdapter::new("writer-1", Rc::clone(&shared)).probing_contention();
    let mut second = RecordingAdapter::new("writer-2", Rc::clone(&shared));

    // Act
    execute_settings_persistence_plan(&first_plan, &mut first).expect("first writer must confirm");
    execute_settings_persistence_plan(&second_plan, &mut second)
        .expect("second writer must confirm after first releases ownership");

    // Assert
    let shared = shared.borrow();
    let first_publish = shared
        .events
        .iter()
        .position(|event| {
            *event == AdapterEvent::Step("writer-1", SettingsPersistenceStep::Publish)
        })
        .expect("first publication event must exist");
    let first_end = shared
        .events
        .iter()
        .position(|event| *event == AdapterEvent::End("writer-1"))
        .expect("first transaction end must exist");
    let second_begin = shared
        .events
        .iter()
        .position(|event| *event == AdapterEvent::Begin("writer-2"))
        .expect("second transaction begin must exist");
    assert!(shared.events.contains(&AdapterEvent::Blocked("writer-2")));
    assert!(first_publish < first_end && first_end < second_begin);
    assert_eq!(
        shared.publication_history,
        [
            ("writer-1", "writer-one".to_owned()),
            ("writer-2", "writer-two".to_owned()),
        ]
    );
}
