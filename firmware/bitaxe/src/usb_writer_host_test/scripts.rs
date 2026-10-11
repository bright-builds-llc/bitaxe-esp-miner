//! One script per writer behavior; each transcript is pinned byte for byte.
use super::fakes::{Effect, Fault, Step};
use super::transcript;

const S1: &str = "AAAAAAAAAAAAAAAAAAAAAA";
const S2: &str = "AQEBAQEBAQEBAQEBAQEBAQ";

/// A JSON object payload of exactly `bytes` bytes.
fn payload(bytes: usize) -> Vec<u8> {
    format!(r#"{{"p":"{}"}}"#, "x".repeat(bytes - 8)).into_bytes()
}

fn open(epoch: u32, session: &'static str) -> Vec<Step> {
    vec![
        Step::Link(Effect::Begin(epoch)),
        Step::Hello { epoch, session },
    ]
}

fn control(epoch: u32, request_sequence: u32, payload: Vec<u8>) -> Step {
    Step::Control {
        epoch,
        request_sequence,
        payload,
    }
}

fn arm(write: u32, check: u32, fault: Fault) -> Step {
    Step::Arm {
        write,
        check,
        fault,
    }
}

fn script(parts: Vec<Vec<Step>>) -> Vec<Step> {
    parts.into_iter().flatten().collect()
}

macro_rules! pinned {
    ($name:ident, $steps:expr) => {
        #[test]
        fn $name() {
            // Arrange
            let steps = $steps;
            // Act
            let actual = transcript(steps);
            // Assert
            if let Ok(directory) = std::env::var("WRITER_TRANSCRIPT_BLESS_DIR") {
                let path = format!("{directory}/{}.txt", stringify!($name));
                std::fs::write(path, &actual).expect("bless transcript");
                return;
            }
            let expected = include_str!(concat!("transcripts/", stringify!($name), ".txt"));
            assert_eq!(actual, expected);
        }
    };
}

pinned!(
    hello_ack_then_heartbeat_at_1000_not_999,
    script(vec![
        open(1, S1),
        vec![Step::At(999), Step::Timeout, Step::At(1000), Step::Timeout],
    ])
);

pinned!(
    stale_hello_is_dropped_without_output,
    vec![
        Step::Link(Effect::Begin(2)),
        Step::Hello {
            epoch: 1,
            session: S1
        },
        Step::Timeout,
    ]
);

pinned!(
    hello_resynchronizes_partial_output_before_its_ack,
    script(vec![
        open(1, S1),
        vec![
            arm(1, 2, Fault::Effect(Effect::Revoke)),
            control(1, 1, payload(1500)),
        ],
        open(2, S2),
        vec![Step::Timeout],
    ])
);

pinned!(
    resynchronization_failure_revokes_the_new_epoch,
    script(vec![
        open(1, S1),
        vec![
            arm(1, 2, Fault::Effect(Effect::Revoke)),
            control(1, 1, payload(1500)),
            Step::Link(Effect::Begin(2)),
            arm(1, 1, Fault::Rejected),
            Step::Hello {
                epoch: 2,
                session: S2
            },
            Step::Timeout,
        ],
    ])
);

pinned!(
    second_hello_restarts_sequence_and_credit,
    script(vec![
        open(1, S1),
        vec![
            Step::Link(Effect::Publish(10)),
            Step::Timeout,
            control(1, 1, payload(20)),
        ],
        open(2, S2),
        vec![Step::Link(Effect::Publish(4)), Step::Timeout],
    ])
);

pinned!(
    receive_credit_is_sent_only_when_it_advances,
    script(vec![
        open(1, S1),
        vec![
            Step::Link(Effect::Publish(5)),
            Step::Timeout,
            Step::Timeout,
            Step::Link(Effect::Publish(9)),
            Step::Timeout,
        ],
    ])
);

pinned!(
    idle_credit_failure_revokes_and_skips_diagnostics,
    script(vec![
        open(1, S1),
        vec![
            Step::Link(Effect::Publish(5)),
            arm(1, 1, Fault::Rejected),
            Step::Timeout,
            Step::Timeout,
        ],
    ])
);

pinned!(
    heartbeat_failure_revokes_the_epoch,
    script(vec![
        open(1, S1),
        // The idle startup marker is the first write after arming; the
        // heartbeat due at the next loop top is the second.
        vec![
            Step::At(1000),
            arm(2, 1, Fault::Rejected),
            Step::Timeout,
            Step::Timeout,
        ],
    ])
);

pinned!(
    terminal_credit_follows_a_validated_close,
    script(vec![
        open(1, S1),
        vec![
            Step::Link(Effect::Publish(20)),
            Step::Timeout,
            Step::At(100),
            Step::Link(Effect::CloseRecord { received: 21 }),
            Step::Timeout,
            Step::Timeout,
        ],
    ])
);

pinned!(
    partial_output_blocks_the_terminal_credit,
    script(vec![
        open(1, S1),
        vec![
            Step::Link(Effect::Publish(20)),
            Step::Timeout,
            arm(1, 2, Fault::Effect(Effect::CloseRecord { received: 21 })),
            control(1, 1, payload(1500)),
            Step::Timeout,
        ],
    ])
);

pinned!(
    terminal_credit_expires_after_two_seconds,
    script(vec![
        open(1, S1),
        vec![
            Step::Link(Effect::Publish(20)),
            Step::Timeout,
            Step::At(100),
            Step::Link(Effect::CloseRecord { received: 21 }),
            Step::At(2100),
            Step::Timeout,
        ],
    ])
);

pinned!(
    failed_terminal_credit_still_finishes_the_receipt,
    script(vec![
        open(1, S1),
        vec![
            Step::Link(Effect::Publish(20)),
            Step::Timeout,
            Step::Link(Effect::CloseRecord { received: 21 }),
            arm(1, 1, Fault::Rejected),
            Step::Timeout,
            Step::Timeout,
        ],
    ])
);

pinned!(
    control_reply_follows_its_credit,
    script(vec![
        open(1, S1),
        vec![
            Step::Link(Effect::Publish(30)),
            control(1, 5, br#"{"ok":true}"#.to_vec()),
            Step::Timeout,
        ],
    ])
);

pinned!(
    stale_control_is_rejected_and_revokes_its_epoch,
    script(vec![
        open(1, S1),
        vec![control(7, 1, payload(20)), Step::Timeout]
    ])
);

pinned!(
    long_control_reply_is_preceded_by_a_heartbeat,
    script(vec![
        open(1, S1),
        vec![
            control(1, 1, payload(4096)),
            Step::At(10),
            control(1, 2, payload(4097)),
        ],
    ])
);

pinned!(
    control_cancelled_mid_record_leaves_partial_output,
    script(vec![
        open(1, S1),
        vec![
            arm(1, 2, Fault::Effect(Effect::Revoke)),
            control(1, 1, payload(1500)),
            Step::Timeout,
        ],
    ])
);

pinned!(
    invalid_control_json_is_rejected,
    script(vec![
        open(1, S1),
        vec![control(1, 1, b"not json".to_vec()), Step::Timeout]
    ])
);

pinned!(
    admission_lost_after_flush_is_unsent,
    script(vec![
        open(1, S1),
        vec![
            arm(1, 3, Fault::Effect(Effect::Revoke)),
            control(1, 1, payload(20)),
            Step::Timeout,
        ],
    ])
);

pinned!(
    bootstrap_diagnostics_are_plain_lines,
    vec![
        Step::Timeout,
        Step::Diagnostic("bwg_worker event=restoration_pending"),
        Step::At(250),
        Step::Timeout,
        arm(1, 1, Fault::Rejected),
        Step::At(500),
        Step::Timeout,
        Step::Timeout,
    ]
);

pinned!(
    session_diagnostics_are_framed_records,
    script(vec![
        open(1, S1),
        vec![
            Step::Diagnostic("invalid_frame"),
            Step::Timeout,
            Step::At(250),
            Step::Timeout,
            Step::At(500),
            Step::Timeout,
            Step::At(750),
            Step::Timeout,
        ],
    ])
);

pinned!(write_failure_marker_replays_in_slot_fifteen, {
    let mut steps = script(vec![
        open(1, S1),
        vec![
            arm(1, 2, Fault::Effect(Effect::Revoke)),
            control(1, 1, payload(1500)),
        ],
    ]);
    for tick in 1..=80 {
        steps.push(Step::At(tick * 100));
        steps.push(Step::Timeout);
    }
    steps
});

pinned!(
    statistics_startup_failure_skips_the_iteration,
    vec![
        Step::StartupMarker(Some(
            "statistics_startup schema=v1 state=failed redacted=true"
        )),
        arm(1, 1, Fault::Rejected),
        Step::Timeout,
        Step::At(250),
        Step::Timeout,
    ]
);
