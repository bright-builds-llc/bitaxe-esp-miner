//! Deterministic host ownership for an ESP device across application restart.

mod display_uat;
mod evidence;
mod fixture;
mod inspection;
mod live;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(not(target_os = "macos"))]
#[path = "macos_unsupported.rs"]
mod macos;
mod model;
mod platform;
mod transaction;
mod usb;

pub use display_uat::{
    finalize_display_uat, run_display_uat_live, DisplayUatIntent, DisplayUatProjection,
    DISPLAY_UAT_INTENT_SCHEMA, DISPLAY_UAT_MACHINE_SCHEMA, DISPLAY_UAT_PROJECTION_SCHEMA,
};
pub use evidence::{validate_private_input, InspectionArtifacts, SessionArtifacts};
pub use fixture::{run_fixture_session, FixtureTranscript, FIXTURE_SCHEMA};
pub use inspection::{
    run_admitted_inspection, DeviceInspectionIntent, DeviceInspectionProjection,
    INSPECTION_INTENT_SCHEMA, INSPECTION_PROJECTION_SCHEMA,
};
pub use live::{run_admitted_live_session, run_admitted_ota_session, run_live_session};
pub use model::{
    BaselineApplication, DevicePhase, ExpectedPostcondition, OtaIntent, PhysicalMatch,
    PlatformCategory, PrivateBootB, PrivateSessionResult, PublicProjection, RebootIntent,
    RequestOutcome, SerialDelivery, SerialPhase, SessionEvent, SessionRequest, SessionState,
    TerminalCategory, OTA_INTENT_SCHEMA, PRIVATE_RESULT_SCHEMA, PUBLIC_PROJECTION_SCHEMA,
    REBOOT_INTENT_SCHEMA, REQUEST_SCHEMA,
};
pub use platform::current_platform;
pub use transaction::{
    run_admitted_transaction, DeviceTransactionIntent, TransactionGoal, TRANSACTION_INTENT_SCHEMA,
};
pub use usb::{
    discover_usb_ports, reduce_lifecycle, retry_is_eligible, MonitorOutput, ReflashReady,
    RetryContext, SupervisedOutput, SupervisedTermination, UsbCommandDiagnostic,
    UsbCommandTermination, UsbConnectionSignature, UsbDeviceEffectState, UsbLifecycleEvent,
    UsbLifecycleState, UsbOperation, UsbSession, UsbSessionError, UsbTerminalCategory,
};
