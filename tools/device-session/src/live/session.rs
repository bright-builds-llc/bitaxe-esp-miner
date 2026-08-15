use std::fs;
use std::thread;
use std::time::{Duration, Instant};

use anyhow::Result;
use bitaxe_http_transport::StrictHttpClient;

use super::{
    apply_event, baseline_matches, elapsed_millis, finish_failed_session, is_success_status,
    maybe_parse_boot_b, maybe_successful_http_response, next_poll_deadline, record_http,
    recovery_http_deadline, remaining, request_evidence_fields,
};
use crate::macos::{MacOsDeviceAdapter, ReceiveOnlyReader};
use crate::{
    current_platform, DevicePhase, PlatformCategory, SerialPhase, SessionArtifacts, SessionEvent,
    SessionRequest, SessionState, TerminalCategory,
};

const SAMPLE_INTERVAL: Duration = Duration::from_millis(250);
const INITIAL_DEVICE_TIMEOUT: Duration = Duration::from_secs(10);
const PRE_RESTART_DELIVERY_TIMEOUT: Duration = Duration::from_secs(30);

pub fn run_live_session(
    request: SessionRequest,
    artifacts: SessionArtifacts,
    timeout: Duration,
) -> Result<TerminalCategory> {
    run_live_session_with_action(request, artifacts, timeout, LiveAction::Restart)
}

pub(super) fn run_live_ota_session(
    request: SessionRequest,
    ota_image: Vec<u8>,
    artifacts: SessionArtifacts,
    timeout: Duration,
) -> Result<TerminalCategory> {
    run_live_session_with_action(request, artifacts, timeout, LiveAction::Ota(ota_image))
}

enum LiveAction {
    Restart,
    Ota(Vec<u8>),
}

fn run_live_session_with_action(
    request: SessionRequest,
    artifacts: SessionArtifacts,
    timeout: Duration,
    action: LiveAction,
) -> Result<TerminalCategory> {
    let mut session = LiveSession::new(request, artifacts, timeout)?;
    if !session.observe_platform()? {
        session.apply(SessionEvent::CleanupComplete)?;
        return session.finish();
    }
    if !session.observe_initial_device()? {
        return session.finish_failed();
    }
    let Some(reader) = session.arm_reader_and_confirm_delivery()? else {
        return session.finish_failed();
    };
    let http = StrictHttpClient::new(&session.request.trusted_origin)?;
    if !session.confirm_baseline(&http)? {
        return session.finish_failed();
    }
    if !session.request_action_once(&http, action)? {
        return session.finish_failed();
    }
    let reader = session.observe_recovery(Some(reader), &http)?;
    session.finish_recovery(reader)
}

struct LiveSession {
    request: SessionRequest,
    artifacts: SessionArtifacts,
    state: SessionState,
    session_started: Instant,
    deadline: Instant,
    timeout: Duration,
    selected_port: String,
}

impl LiveSession {
    fn new(
        request: SessionRequest,
        artifacts: SessionArtifacts,
        timeout: Duration,
    ) -> Result<Self> {
        if !request.schema_is_valid() {
            anyhow::bail!("device-session request schema is invalid");
        }
        let session_started = Instant::now();
        let state = SessionState::new(
            request.baseline.clone(),
            request.expected_postcondition.clone(),
            request.trusted_origin.clone(),
        );
        let selected_port = request.admitted_port.clone();
        Ok(Self {
            request,
            artifacts,
            state,
            session_started,
            deadline: session_started + timeout,
            timeout,
            selected_port,
        })
    }

    fn observe_platform(&mut self) -> Result<bool> {
        let platform = current_platform();
        self.apply(SessionEvent::PlatformObserved { category: platform })?;
        if platform != PlatformCategory::Macos {
            return Ok(false);
        }
        self.session_started = Instant::now();
        self.deadline = self.session_started + self.timeout;
        Ok(true)
    }

    fn observe_initial_device(&mut self) -> Result<bool> {
        let initial_deadline = self.session_started + INITIAL_DEVICE_TIMEOUT.min(self.timeout);
        while Instant::now() < initial_deadline && !self.state.device_ready(DevicePhase::Initial) {
            let observation = MacOsDeviceAdapter::initial_sample(
                &self.request.admitted_port,
                &self.request.physical_identity_digest,
            )?;
            if let Some(port) = observation.maybe_port {
                self.selected_port = port;
            }
            self.apply(observation.event)?;
            if self.state.terminal_category() != TerminalCategory::Incomplete {
                break;
            }
            thread::sleep(SAMPLE_INTERVAL);
        }
        Ok(self.state.device_ready(DevicePhase::Initial))
    }

    fn arm_reader_and_confirm_delivery(&mut self) -> Result<Option<ReceiveOnlyReader>> {
        let mut reader = match ReceiveOnlyReader::open(&self.selected_port) {
            Ok(reader) => {
                self.apply(SessionEvent::ReaderArmed)?;
                reader
            }
            Err(_) => {
                self.apply(SessionEvent::ReaderStartFailed)?;
                return Ok(None);
            }
        };
        let pre_delivery_deadline = Instant::now()
            + PRE_RESTART_DELIVERY_TIMEOUT.min(remaining(self.session_started, self.timeout));
        while Instant::now() < pre_delivery_deadline {
            let bytes = match reader.read_available() {
                Ok(bytes) => bytes,
                Err(_) => {
                    self.apply(SessionEvent::ReaderStartFailed)?;
                    return Ok(None);
                }
            };
            if !bytes.is_empty() {
                if !self.artifacts.record_serial(&bytes)? {
                    self.state.apply(SessionEvent::AdmissionRejected);
                    return Ok(None);
                }
                self.apply(SessionEvent::SerialBytes {
                    phase: SerialPhase::PreRestart,
                    count: u64::try_from(bytes.len()).unwrap_or(u64::MAX),
                })?;
                break;
            }
            thread::sleep(SAMPLE_INTERVAL);
        }
        Ok(self
            .state
            .projection()
            .pre_restart_serial_delivery
            .then_some(reader))
    }

    fn confirm_baseline(&mut self, http: &StrictHttpClient) -> Result<bool> {
        let baseline_http = http.get_system_info(self.deadline)?;
        record_http(
            &mut self.state,
            &mut self.artifacts,
            "baseline",
            &baseline_http,
        )?;
        let confirmed = maybe_successful_http_response(&baseline_http)
            .is_some_and(|response| baseline_matches(&self.request, response.body()));
        if !confirmed {
            self.apply(SessionEvent::BaselineMismatch)?;
            return Ok(false);
        }
        self.apply(SessionEvent::BaselineConfirmed)?;
        Ok(true)
    }

    fn request_action_once(&mut self, http: &StrictHttpClient, action: LiveAction) -> Result<bool> {
        self.apply(SessionEvent::RestartRequestStarted)?;
        let restart = match action {
            LiveAction::Restart => http.post_restart_once(self.deadline)?,
            LiveAction::Ota(image) => {
                http.post_binary_once("/api/system/OTA", &image, self.deadline)?
            }
        };
        record_http(&mut self.state, &mut self.artifacts, "restart", &restart)?;
        let (bytes_written, write_complete) = request_evidence_fields(restart.request_progress());
        if bytes_written > 0 {
            self.apply(SessionEvent::RestartRequestBytesWritten {
                count: bytes_written,
            })?;
        }
        if write_complete {
            self.apply(SessionEvent::RestartRequestWriteComplete)?;
        }
        if let Some(response) = restart.maybe_http_response() {
            let event = if is_success_status(response.status()) {
                SessionEvent::RestartResponseReceived
            } else {
                SessionEvent::RestartResponseRejected
            };
            self.apply(event)?;
        }
        Ok(self.state.terminal_category() == TerminalCategory::Incomplete)
    }

    fn observe_recovery(
        &mut self,
        mut reader: Option<ReceiveOnlyReader>,
        http: &StrictHttpClient,
    ) -> Result<Option<ReceiveOnlyReader>> {
        let mut next_http_poll = Instant::now();
        let mut service_loss_recorded = false;
        let mut disappearance_recorded = false;
        while Instant::now() < self.deadline && !self.state.authoritative_quorum_satisfied() {
            if !self.observe_recovery_serial(&mut reader, &mut disappearance_recorded)? {
                break;
            }
            if !self.observe_recovery_device(&mut reader)? {
                break;
            }
            if Instant::now() >= next_http_poll {
                let polled = http.get_system_info(recovery_http_deadline(self.deadline))?;
                record_http(&mut self.state, &mut self.artifacts, "recovery", &polled)?;
                match maybe_successful_http_response(&polled) {
                    Some(response) => {
                        if let Some(boot_b) =
                            maybe_parse_boot_b(&self.request.trusted_origin, response.body())
                        {
                            self.apply(SessionEvent::BootBObserved { boot_b })?;
                        }
                    }
                    _ if !service_loss_recorded => {
                        self.apply(SessionEvent::ServiceLossObserved)?;
                        service_loss_recorded = true;
                    }
                    _ => {}
                }
                next_http_poll = next_poll_deadline(next_http_poll, Instant::now());
            }
            thread::sleep(Duration::from_millis(100));
        }
        Ok(reader)
    }

    fn observe_recovery_serial(
        &mut self,
        reader: &mut Option<ReceiveOnlyReader>,
        disappearance_recorded: &mut bool,
    ) -> Result<bool> {
        let Some(active_reader) = reader.as_mut() else {
            return Ok(true);
        };
        if fs::metadata(active_reader.port()).is_err() {
            *reader = None;
            if !*disappearance_recorded {
                self.apply(SessionEvent::DeviceAbsent)?;
                *disappearance_recorded = true;
            }
            return Ok(true);
        }
        let bytes = match active_reader.read_available() {
            Ok(bytes) => bytes,
            Err(_) => {
                *reader = None;
                self.apply(SessionEvent::ReaderLost)?;
                return Ok(true);
            }
        };
        if bytes.is_empty() {
            return Ok(true);
        }
        if !self.artifacts.record_serial(&bytes)? {
            self.state.apply(SessionEvent::AdmissionRejected);
            return Ok(false);
        }
        self.apply(SessionEvent::SerialBytes {
            phase: SerialPhase::PostRestart,
            count: u64::try_from(bytes.len()).unwrap_or(u64::MAX),
        })?;
        Ok(true)
    }

    fn observe_recovery_device(&mut self, reader: &mut Option<ReceiveOnlyReader>) -> Result<bool> {
        if self.state.device_ready(DevicePhase::Recovery) {
            return Ok(true);
        }
        let observation = MacOsDeviceAdapter::recovery_sample(
            &self.request.physical_identity_digest,
            &self.selected_port,
        )?;
        if let Some(port) = observation.maybe_port {
            self.selected_port = port;
        }
        self.apply(observation.event)?;
        if self.state.terminal_category() != TerminalCategory::Incomplete {
            return Ok(false);
        }
        if reader.is_none() && self.state.device_ready(DevicePhase::Recovery) {
            match ReceiveOnlyReader::open(&self.selected_port) {
                Ok(new_reader) => {
                    *reader = Some(new_reader);
                    self.apply(SessionEvent::ReaderReacquired)?;
                }
                Err(_) => {
                    self.apply(SessionEvent::ReaderStartFailed)?;
                    return Ok(false);
                }
            }
        }
        Ok(true)
    }

    fn finish_recovery(mut self, reader: Option<ReceiveOnlyReader>) -> Result<TerminalCategory> {
        drop(reader);
        thread::sleep(Duration::from_millis(50));
        if MacOsDeviceAdapter::holder_count(&self.selected_port)? > 0 {
            self.apply(SessionEvent::CleanupFailed)?;
        } else if self.state.authoritative_quorum_satisfied() {
            self.apply(SessionEvent::CleanupComplete)?;
        } else {
            self.apply(SessionEvent::ObservationWindowExpired {
                duration_millis: elapsed_millis(self.session_started),
            })?;
            self.apply(SessionEvent::CleanupComplete)?;
        }
        self.finish()
    }

    fn apply(&mut self, event: SessionEvent) -> Result<()> {
        apply_event(&mut self.state, &mut self.artifacts, event)
    }

    fn finish_failed(self) -> Result<TerminalCategory> {
        finish_failed_session(self.state, self.artifacts, self.session_started)
    }

    fn finish(self) -> Result<TerminalCategory> {
        let terminal = self.state.terminal_category();
        self.artifacts.finish(&self.state)?;
        Ok(terminal)
    }
}
