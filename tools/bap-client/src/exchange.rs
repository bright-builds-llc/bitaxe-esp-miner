//! The pure BAP host exchange: the frame an operation sends, and how each inbound
//! frame settles it. Time and bytes come from a [`Link`], so the whole exchange runs
//! the same over a serial adapter or a scripted test link.

use bitaxe_core::bap::{BapCommand, BapFrame, BapFrameError, BapParameter, BAP_MAX_MESSAGE_LEN};
use std::io;

/// One host operation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Operation {
    Request(BapParameter),
    /// Subscribes, listens, then unsubscribes so no subscription outlives the client.
    Subscribe {
        parameter: BapParameter,
        maybe_interval_ms: Option<u32>,
    },
    Unsubscribe(BapParameter),
    Set {
        parameter: BapParameter,
        value: String,
    },
}

impl Operation {
    #[must_use]
    pub const fn parameter(&self) -> BapParameter {
        match self {
            Self::Request(parameter) | Self::Unsubscribe(parameter) => *parameter,
            Self::Subscribe { parameter, .. } | Self::Set { parameter, .. } => *parameter,
        }
    }

    /// The canonical frame this operation sends first.
    pub fn frame(&self) -> Result<BapFrame, BapFrameError> {
        match self {
            Self::Request(parameter) => BapFrame::new(BapCommand::Request, *parameter, None),
            Self::Subscribe {
                parameter,
                maybe_interval_ms,
            } => BapFrame::new(
                BapCommand::Subscribe,
                *parameter,
                maybe_interval_ms.map(|interval| interval.to_string()),
            ),
            Self::Unsubscribe(parameter) => {
                BapFrame::new(BapCommand::Unsubscribe, *parameter, None)
            }
            Self::Set { parameter, value } => {
                BapFrame::new(BapCommand::Set, *parameter, Some(value.clone()))
            }
        }
    }
}

/// What one inbound frame means for a pending operation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Disposition {
    /// The device acknowledged the operation.
    Confirmed,
    /// The device refused it with this error code.
    Rejected(String),
    /// A value the operation asked for.
    Data,
    /// Traffic that does not settle the operation, such as a status or log line.
    Other,
}

#[must_use]
pub fn classify(operation: &Operation, frame: &BapFrame) -> Disposition {
    let same = frame.parameter_token() == operation.parameter().token();
    let value = frame.value().unwrap_or_default();
    match (operation, frame.command()) {
        (_, BapCommand::Error) if same => Disposition::Rejected(value.to_owned()),
        // A request may answer with several tokens, as systemInfo does.
        (Operation::Request(_) | Operation::Subscribe { .. }, BapCommand::Response) => {
            Disposition::Data
        }
        (Operation::Subscribe { .. }, BapCommand::Acknowledge) if same && value == "subscribed" => {
            Disposition::Confirmed
        }
        (Operation::Unsubscribe(_), BapCommand::Acknowledge) if same => Disposition::Confirmed,
        (Operation::Set { .. }, BapCommand::Acknowledge) if same => Disposition::Confirmed,
        _ => Disposition::Other,
    }
}

/// Bytes and time from the adapter, or from a test script.
pub trait Link {
    fn send(&mut self, bytes: &[u8]) -> io::Result<()>;
    /// Bytes that arrive within `wait_ms`; empty when none did.
    fn receive(&mut self, wait_ms: u64) -> io::Result<Vec<u8>>;
    fn now_ms(&self) -> u64;
}

/// Bounds for one operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Timing {
    /// How long to wait for a confirmation, rejection or first value.
    pub answer_ms: u64,
    /// A request is complete once no further value arrives for this long.
    pub quiet_ms: u64,
    /// How long a subscription listens before it unsubscribes.
    pub listen_ms: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    Confirmed,
    Answered,
    Rejected(String),
    TimedOut,
    /// The subscription ran, but its unsubscribe was not acknowledged.
    UnsubscribeUnconfirmed,
}

/// Every inbound frame, in arrival order, and how the operation ended.
#[derive(Debug)]
pub struct Report {
    pub outcome: Outcome,
    pub frames: Vec<BapFrame>,
    pub malformed_lines: u32,
}

/// Runs one operation to its outcome.
pub fn run(operation: &Operation, link: &mut impl Link, timing: Timing) -> io::Result<Report> {
    let mut session = Session {
        link,
        lines: Lines::default(),
        frames: Vec::new(),
        malformed_lines: 0,
    };
    let outcome = match operation {
        Operation::Request(_) => {
            session.send(operation)?;
            session.collect(operation, timing)?
        }
        Operation::Subscribe { parameter, .. } => {
            session.send(operation)?;
            match session.settle(operation, timing.answer_ms)? {
                Outcome::Confirmed => {
                    session.listen(timing.listen_ms)?;
                    let unsubscribe = Operation::Unsubscribe(*parameter);
                    session.send(&unsubscribe)?;
                    match session.settle(&unsubscribe, timing.answer_ms)? {
                        Outcome::Confirmed => Outcome::Confirmed,
                        _ => Outcome::UnsubscribeUnconfirmed,
                    }
                }
                other => other,
            }
        }
        Operation::Unsubscribe(_) => {
            session.send(operation)?;
            session.settle(operation, timing.answer_ms)?
        }
        Operation::Set { .. } => {
            session.send(operation)?;
            let outcome = session.settle(operation, timing.answer_ms)?;
            // A credential change announces a restart right after its acknowledgement.
            session.listen(timing.quiet_ms)?;
            outcome
        }
    };
    Ok(Report {
        outcome,
        frames: session.frames,
        malformed_lines: session.malformed_lines,
    })
}

struct Session<'a, L: Link> {
    link: &'a mut L,
    lines: Lines,
    frames: Vec<BapFrame>,
    malformed_lines: u32,
}

impl<L: Link> Session<'_, L> {
    fn send(&mut self, operation: &Operation) -> io::Result<()> {
        let encoded = operation
            .frame()
            .and_then(|frame| frame.encode())
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "bap_frame_invalid"))?;
        self.link.send(encoded.as_bytes())
    }

    /// The next inbound frames within `wait_ms`, recorded as they arrive.
    fn next(&mut self, wait_ms: u64) -> io::Result<Vec<BapFrame>> {
        let bytes = self.link.receive(wait_ms)?;
        let mut frames = Vec::new();
        for line in self.lines.push(&bytes) {
            match line.and_then(|line| BapFrame::parse(&line).ok()) {
                Some(frame) => {
                    self.frames.push(frame.clone());
                    frames.push(frame);
                }
                None => self.malformed_lines += 1,
            }
        }
        Ok(frames)
    }

    /// Waits for a confirmation or rejection.
    fn settle(&mut self, operation: &Operation, answer_ms: u64) -> io::Result<Outcome> {
        let deadline = self.link.now_ms().saturating_add(answer_ms);
        while self.link.now_ms() < deadline {
            let wait = deadline - self.link.now_ms();
            for frame in self.next(wait)? {
                match classify(operation, &frame) {
                    Disposition::Confirmed => return Ok(Outcome::Confirmed),
                    Disposition::Rejected(code) => return Ok(Outcome::Rejected(code)),
                    Disposition::Data | Disposition::Other => {}
                }
            }
        }
        Ok(Outcome::TimedOut)
    }

    /// A request's values: complete once they stop arriving for the quiet period.
    fn collect(&mut self, operation: &Operation, timing: Timing) -> io::Result<Outcome> {
        let started = self.link.now_ms();
        let mut maybe_last_data: Option<u64> = None;
        loop {
            let now = self.link.now_ms();
            let deadline = match maybe_last_data {
                Some(last) => last.saturating_add(timing.quiet_ms),
                None => started.saturating_add(timing.answer_ms),
            };
            if now >= deadline {
                return Ok(match maybe_last_data {
                    Some(_) => Outcome::Answered,
                    None => Outcome::TimedOut,
                });
            }
            for frame in self.next(deadline - now)? {
                match classify(operation, &frame) {
                    Disposition::Data => maybe_last_data = Some(self.link.now_ms()),
                    Disposition::Rejected(code) => return Ok(Outcome::Rejected(code)),
                    Disposition::Confirmed | Disposition::Other => {}
                }
            }
        }
    }

    /// Records everything that arrives for `listen_ms`.
    fn listen(&mut self, listen_ms: u64) -> io::Result<()> {
        let deadline = self.link.now_ms().saturating_add(listen_ms);
        while self.link.now_ms() < deadline {
            let wait = deadline - self.link.now_ms();
            self.next(wait)?;
        }
        Ok(())
    }
}

/// Splits received bytes into lines, bounded as the device bounds a message.
#[derive(Default)]
struct Lines {
    pending: Vec<u8>,
    oversized: bool,
}

impl Lines {
    /// Complete lines; `None` stands for one discarded oversized line.
    fn push(&mut self, bytes: &[u8]) -> Vec<Option<Vec<u8>>> {
        let mut lines = Vec::new();
        for byte in bytes {
            if *byte == b'\n' {
                let line = std::mem::take(&mut self.pending);
                if std::mem::take(&mut self.oversized) {
                    lines.push(None);
                } else if !line.iter().all(|byte| byte.is_ascii_whitespace()) {
                    lines.push(Some(line));
                }
                continue;
            }
            if self.pending.len() >= BAP_MAX_MESSAGE_LEN {
                self.oversized = true;
                continue;
            }
            self.pending.push(*byte);
        }
        lines
    }
}

#[cfg(test)]
mod tests;
