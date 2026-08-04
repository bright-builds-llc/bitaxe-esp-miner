//! Deterministic host ownership for an ESP device across application restart.

mod evidence;
mod fixture;
mod live;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(not(target_os = "macos"))]
#[path = "macos_unsupported.rs"]
mod macos;
mod model;
mod platform;
mod usb;

pub use evidence::{validate_private_input, SessionArtifacts};
pub use fixture::{run_fixture_session, FixtureTranscript, FIXTURE_SCHEMA};
pub use live::{run_admitted_live_session, run_live_session};
pub use model::{
    BaselineApplication, DevicePhase, ExpectedPostcondition, PhysicalMatch, PlatformCategory,
    PrivateBootB, PrivateSessionResult, PublicProjection, RebootIntent, RequestOutcome,
    SerialDelivery, SerialPhase, SessionEvent, SessionRequest, SessionState, TerminalCategory,
    PRIVATE_RESULT_SCHEMA, PUBLIC_PROJECTION_SCHEMA, REBOOT_INTENT_SCHEMA, REQUEST_SCHEMA,
};
pub use platform::current_platform;
pub use usb::{
    discover_usb_ports, reduce_lifecycle, retry_is_eligible, MonitorOutput, ReflashReady,
    RetryContext, SupervisedOutput, SupervisedTermination, UsbDeviceEffectState, UsbLifecycleEvent,
    UsbLifecycleState, UsbOperation, UsbSession, UsbSessionError, UsbTerminalCategory,
};
