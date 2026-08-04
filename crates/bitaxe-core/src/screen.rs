//! Pure Ultra 205 priority-screen, overlay, and carousel flow.
//!
//! Reference breadcrumb: `reference/esp-miner/main/screen.c`.

use core::fmt;

/// Pinned screen-state evaluation cadence.
pub const SCREEN_UPDATE_MS: u64 = 500;
/// Pinned intro-page dwell before the next 500 ms evaluation advances it.
pub const INTRO_DELAY_MS: u64 = 3_000;
/// Pinned carousel-page dwell before the next 500 ms evaluation advances it.
pub const CAROUSEL_DELAY_MS: u64 = 10_000;
/// Four text rows fit the Ultra 205 panel.
pub const SCREEN_LINE_COUNT: usize = 4;
/// `FONT_5X7` advances six pixels per glyph on a 128-pixel row.
pub const SCREEN_MAX_LINE_CHARS: usize = 128 / 6;

/// One page in the pinned priority, intro, and carousel vocabulary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenPage {
    SelfTest,
    FirmwareUpdate,
    AsicStatus,
    Overheat,
    Welcome,
    Connection,
    BitaxeIntro,
    OpenSourceIntro,
    Urls,
    Statistics,
    Mining,
    Wifi,
}

impl ScreenPage {
    const fn next_normal(self) -> Self {
        match self {
            Self::BitaxeIntro => Self::OpenSourceIntro,
            Self::OpenSourceIntro => Self::Urls,
            Self::Urls => Self::Statistics,
            Self::Statistics => Self::Mining,
            Self::Mining => Self::Wifi,
            Self::Wifi => Self::Urls,
            _ => Self::BitaxeIntro,
        }
    }

    const fn is_intro(self) -> bool {
        matches!(self, Self::BitaxeIntro | Self::OpenSourceIntro)
    }
}

mod frame;

pub use frame::ScreenFrame;
use frame::{frame, page_frame};

/// Private runtime facts used only to render the physical screen.
#[derive(Clone, Default, PartialEq)]
pub struct ScreenSnapshot {
    pub maybe_self_test: Option<[String; 3]>,
    pub maybe_firmware_update: Option<[String; 2]>,
    pub maybe_asic_status: Option<String>,
    pub overheat: bool,
    pub identify_active: bool,
    pub wifi_connected: bool,
    pub ap_enabled: bool,
    pub ssid: String,
    pub ap_ssid: String,
    pub wifi_status: String,
    pub ipv4: String,
    pub model: String,
    pub board: String,
    pub version: String,
    pub pool_host: String,
    pub maybe_hashrate_ghs: Option<f64>,
    pub maybe_power_watts: Option<f64>,
    pub maybe_best_difficulty: Option<f64>,
    pub maybe_temperature_celsius: Option<f64>,
    pub maybe_block_height: Option<u32>,
    pub maybe_network_difficulty: Option<f64>,
    pub maybe_scriptsig: Option<String>,
    pub maybe_rssi_dbm: Option<i16>,
    pub uptime_seconds: u64,
    pub shares_accepted: u64,
    pub shares_rejected: u64,
    pub work_received: u64,
    pub mining_paused: bool,
    pub show_new_block: bool,
}

impl fmt::Debug for ScreenSnapshot {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ScreenSnapshot")
            .field("private_text", &"redacted")
            .field("wifi_connected", &self.wifi_connected)
            .field("ap_enabled", &self.ap_enabled)
            .field("overheat", &self.overheat)
            .field("identify_active", &self.identify_active)
            .field("show_new_block", &self.show_new_block)
            .finish()
    }
}

/// Closed screen-flow failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenFlowError {
    ClockRegressed,
    DeadlineOverflow,
}

impl fmt::Display for ScreenFlowError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::ClockRegressed => "screen clock regressed",
            Self::DeadlineOverflow => "screen deadline overflow",
        })
    }
}

impl std::error::Error for ScreenFlowError {}

/// One pure screen decision at the pinned update cadence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScreenDecision {
    pub page: ScreenPage,
    pub priority_visible: bool,
    pub frame: ScreenFrame,
}

/// Retained owner of page dwell and notification-delta state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScreenFlow {
    page: ScreenPage,
    page_started_at_ms: u64,
    last_update_ms: u64,
    intro_complete: bool,
    maybe_priority_page: Option<ScreenPage>,
    block_pinned: bool,
    shares_accepted: u64,
    shares_rejected: u64,
    work_received: u64,
}

impl ScreenFlow {
    /// Starts on the highest current priority page or the first intro page.
    #[must_use]
    pub fn new(started_at_ms: u64, snapshot: &ScreenSnapshot) -> Self {
        let maybe_priority_page = priority_page(snapshot);
        Self {
            page: maybe_priority_page.unwrap_or(ScreenPage::BitaxeIntro),
            page_started_at_ms: started_at_ms,
            last_update_ms: started_at_ms,
            intro_complete: false,
            maybe_priority_page,
            block_pinned: false,
            shares_accepted: snapshot.shares_accepted,
            shares_rejected: snapshot.shares_rejected,
            work_received: snapshot.work_received,
        }
    }

    /// Evaluates priorities, overlays, and exact page dwell at `now_ms`.
    pub fn update(
        &mut self,
        now_ms: u64,
        snapshot: &ScreenSnapshot,
    ) -> Result<ScreenDecision, ScreenFlowError> {
        if now_ms < self.last_update_ms {
            return Err(ScreenFlowError::ClockRegressed);
        }
        self.last_update_ms = now_ms;
        let notification = self.notification(snapshot);

        if let Some(priority) = priority_page(snapshot) {
            if self.maybe_priority_page != Some(priority) {
                self.page = priority;
                self.page_started_at_ms = now_ms;
            }
            self.maybe_priority_page = Some(priority);
            self.block_pinned = false;
        } else if self.maybe_priority_page.take().is_some() {
            self.page = if self.intro_complete {
                ScreenPage::Urls
            } else {
                ScreenPage::BitaxeIntro
            };
            self.page_started_at_ms = now_ms;
        } else if snapshot.show_new_block {
            if !self.block_pinned {
                self.page = ScreenPage::Statistics;
                self.page_started_at_ms = now_ms;
                self.intro_complete = true;
                self.block_pinned = true;
            }
        } else {
            if self.block_pinned {
                self.block_pinned = false;
                self.page_started_at_ms = now_ms;
            }
            self.advance_normal_pages(now_ms)?;
        }

        let priority_visible =
            self.maybe_priority_page.is_some() || self.block_pinned || snapshot.identify_active;
        let frame = if snapshot.identify_active {
            frame(["", "BITAXE IDENTIFY", "Hello!", ""])
        } else {
            page_frame(self.page, snapshot, notification)
        };
        Ok(ScreenDecision {
            page: self.page,
            priority_visible,
            frame,
        })
    }

    /// Advances once in pinned screen order for an admitted short click.
    pub fn advance_by_input(
        &mut self,
        now_ms: u64,
        snapshot: &ScreenSnapshot,
    ) -> Result<ScreenDecision, ScreenFlowError> {
        if now_ms < self.last_update_ms {
            return Err(ScreenFlowError::ClockRegressed);
        }
        self.last_update_ms = now_ms;
        self.page = manual_successor(self.page);
        self.page_started_at_ms = now_ms;
        self.intro_complete |= !self.page.is_intro();
        self.maybe_priority_page = None;
        self.block_pinned = false;
        Ok(ScreenDecision {
            page: self.page,
            priority_visible: false,
            frame: page_frame(self.page, snapshot, Notification::default()),
        })
    }

    fn advance_normal_pages(&mut self, now_ms: u64) -> Result<(), ScreenFlowError> {
        for _ in 0..2 {
            if !self.page.is_intro() {
                break;
            }
            let dwell_ms = INTRO_DELAY_MS
                .checked_add(SCREEN_UPDATE_MS)
                .ok_or(ScreenFlowError::DeadlineOverflow)?;
            let elapsed_ms = now_ms
                .checked_sub(self.page_started_at_ms)
                .ok_or(ScreenFlowError::ClockRegressed)?;
            if elapsed_ms < dwell_ms {
                return Ok(());
            }
            self.page = self.page.next_normal();
            self.page_started_at_ms = self
                .page_started_at_ms
                .checked_add(dwell_ms)
                .ok_or(ScreenFlowError::DeadlineOverflow)?;
            if !self.page.is_intro() {
                self.intro_complete = true;
            }
        }

        let dwell_ms = CAROUSEL_DELAY_MS
            .checked_add(SCREEN_UPDATE_MS)
            .ok_or(ScreenFlowError::DeadlineOverflow)?;
        let elapsed_ms = now_ms
            .checked_sub(self.page_started_at_ms)
            .ok_or(ScreenFlowError::ClockRegressed)?;
        let advances = elapsed_ms / dwell_ms;
        if advances == 0 {
            return Ok(());
        }
        self.page = carousel_page(carousel_index(self.page) + advances % 4);
        self.page_started_at_ms = self
            .page_started_at_ms
            .checked_add(
                advances
                    .checked_mul(dwell_ms)
                    .ok_or(ScreenFlowError::DeadlineOverflow)?,
            )
            .ok_or(ScreenFlowError::DeadlineOverflow)?;
        Ok(())
    }

    fn notification(&mut self, snapshot: &ScreenSnapshot) -> Notification {
        let accepted = snapshot.shares_accepted > self.shares_accepted;
        let rejected = snapshot.shares_rejected > self.shares_rejected;
        let work = snapshot.work_received > self.work_received;
        self.shares_accepted = snapshot.shares_accepted;
        self.shares_rejected = snapshot.shares_rejected;
        self.work_received = snapshot.work_received;
        Notification {
            accepted,
            rejected,
            work,
            paused: !accepted && !rejected && !work && snapshot.mining_paused,
        }
    }
}

const fn manual_successor(page: ScreenPage) -> ScreenPage {
    match page {
        ScreenPage::SelfTest => ScreenPage::Overheat,
        ScreenPage::Overheat => ScreenPage::AsicStatus,
        ScreenPage::AsicStatus => ScreenPage::Welcome,
        ScreenPage::Welcome => ScreenPage::FirmwareUpdate,
        ScreenPage::FirmwareUpdate => ScreenPage::Connection,
        ScreenPage::Connection => ScreenPage::BitaxeIntro,
        ScreenPage::BitaxeIntro => ScreenPage::OpenSourceIntro,
        ScreenPage::OpenSourceIntro => ScreenPage::Urls,
        ScreenPage::Urls => ScreenPage::Statistics,
        ScreenPage::Statistics => ScreenPage::Mining,
        ScreenPage::Mining => ScreenPage::Wifi,
        ScreenPage::Wifi => ScreenPage::Urls,
    }
}

const fn carousel_index(page: ScreenPage) -> u64 {
    match page {
        ScreenPage::Urls => 0,
        ScreenPage::Statistics => 1,
        ScreenPage::Mining => 2,
        ScreenPage::Wifi => 3,
        _ => 0,
    }
}

const fn carousel_page(index: u64) -> ScreenPage {
    match index % 4 {
        0 => ScreenPage::Urls,
        1 => ScreenPage::Statistics,
        2 => ScreenPage::Mining,
        _ => ScreenPage::Wifi,
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) struct Notification {
    accepted: bool,
    rejected: bool,
    work: bool,
    paused: bool,
}

impl Notification {
    pub(super) fn label(self) -> &'static str {
        match (self.accepted, self.rejected, self.work, self.paused) {
            (false, false, false, false) => "",
            (true, false, false, _) => "A",
            (false, true, false, _) => "X",
            (true, true, false, _) => "XA",
            (false, false, true, _) => "W",
            (true, false, true, _) => "WA",
            (false, true, true, _) => "XW",
            (true, true, true, _) => "XWA",
            (false, false, false, true) => "||",
        }
    }
}

fn priority_page(snapshot: &ScreenSnapshot) -> Option<ScreenPage> {
    if snapshot.maybe_self_test.is_some() {
        Some(ScreenPage::SelfTest)
    } else if snapshot.maybe_firmware_update.is_some() {
        Some(ScreenPage::FirmwareUpdate)
    } else if snapshot.maybe_asic_status.is_some() {
        Some(ScreenPage::AsicStatus)
    } else if snapshot.overheat {
        Some(ScreenPage::Overheat)
    } else if snapshot.ssid.is_empty() {
        Some(ScreenPage::Welcome)
    } else if snapshot.ap_enabled || !snapshot.wifi_connected {
        Some(ScreenPage::Connection)
    } else {
        None
    }
}

#[cfg(test)]
#[path = "screen/tests.rs"]
mod tests;
