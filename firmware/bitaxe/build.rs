use bitaxe_api::BuildProvenance;
use std::env;
use std::fs;

fn main() {
    embuild::espidf::sysenv::output();
    assert_sdkconfig_contract();
    println!("cargo:rerun-if-env-changed=BITAXE_BUILD_PROVENANCE_STAMP");
    println!("cargo:rerun-if-env-changed=BITAXE_BUILD_TIMESTAMP_UTC_FILE");
    println!("cargo:rerun-if-env-changed=BITAXE_HARDWARE_EVIDENCE_ACK");
    println!("cargo:rerun-if-env-changed=BITAXE_WORK_RESULT_INVESTIGATION");
    println!("cargo:rerun-if-env-changed=BITAXE_CHIP_DETECT_INVESTIGATION");
    println!("cargo:rerun-if-env-changed=BITAXE_OTA_ROLLBACK_PROBE");
    let provenance = required_build_provenance();
    let identity = provenance.build_identity();
    println!(
        "cargo:rustc-env=BITAXE_SEMANTIC_VERSION={}",
        provenance.semantic_version()
    );
    println!(
        "cargo:rustc-env=BITAXE_FIRMWARE_COMMIT={}",
        identity.source_commit()
    );
    println!(
        "cargo:rustc-env=BITAXE_BUILD_LABEL={}",
        identity.build_label()
    );
    println!(
        "cargo:rustc-env=BITAXE_BUILD_CHANNEL={}",
        identity.build_channel().as_str()
    );
    println!(
        "cargo:rustc-env=BITAXE_SOURCE_DIRTY={}",
        identity.source_dirty()
    );
    println!(
        "cargo:rustc-env=BITAXE_RELEASE_TAG={}",
        identity.maybe_release_tag().unwrap_or("unavailable")
    );
    println!(
        "cargo:rustc-env=BITAXE_REFERENCE_COMMIT={}",
        provenance.reference_commit()
    );
    println!(
        "cargo:rustc-env=BITAXE_RUNTIME_BUILD_IDENTITY={}",
        provenance.runtime_identity_record()
    );
    println!(
        "cargo:rustc-env=BITAXE_BUILD_TIMESTAMP_UTC={}",
        required_build_timestamp_utc()
    );
    println!(
        "cargo:rustc-env=BITAXE_OTA_ROLLBACK_PROBE={}",
        rollback_probe_enabled()
    );
}

fn rollback_probe_enabled() -> bool {
    match env::var("BITAXE_OTA_ROLLBACK_PROBE").as_deref() {
        Ok("1") => true,
        Ok("0") | Err(_) => false,
        Ok(_) => panic!("BITAXE_OTA_ROLLBACK_PROBE must be 0 or 1"),
    }
}

fn assert_sdkconfig_contract() {
    const REQUIRED_DEFAULTS: [&str; 5] = [
        "CONFIG_ESP_CONSOLE_UART_DEFAULT=y",
        "CONFIG_ESP_CONSOLE_UART_BAUDRATE=115200",
        "CONFIG_ESP_CONSOLE_SECONDARY_USB_SERIAL_JTAG=y",
        "CONFIG_TINYUSB_TASK_STACK_SIZE=3072",
        "CONFIG_SPIRAM_MALLOC_RESERVE_INTERNAL=98304",
    ];

    println!("cargo:rerun-if-changed=sdkconfig.defaults");
    let Ok(defaults) = fs::read_to_string("sdkconfig.defaults") else {
        panic!("firmware build requires readable sdkconfig.defaults");
    };
    for required in REQUIRED_DEFAULTS {
        if !defaults.lines().any(|line| line == required) {
            panic!("firmware sdkconfig contract missing {required}");
        }
    }
}

fn required_build_provenance() -> BuildProvenance {
    let stamp_path = env::var("BITAXE_BUILD_PROVENANCE_STAMP").unwrap_or_else(|_| {
        panic!("canonical firmware build requires build provenance; run `just build`")
    });
    println!("cargo:rerun-if-changed={stamp_path}");
    let stamp = fs::read_to_string(&stamp_path).unwrap_or_else(|error| {
        panic!("failed to read canonical build provenance {stamp_path}: {error}")
    });
    BuildProvenance::parse_stamp(&stamp).unwrap_or_else(|error| {
        panic!("invalid canonical build provenance; run `just build`: {error}")
    })
}

fn required_build_timestamp_utc() -> String {
    let timestamp_path = env::var("BITAXE_BUILD_TIMESTAMP_UTC_FILE").unwrap_or_else(|_| {
        panic!("canonical firmware build requires build timestamp; run `just build`")
    });
    println!("cargo:rerun-if-changed={timestamp_path}");
    let timestamp = fs::read_to_string(&timestamp_path)
        .unwrap_or_else(|error| panic!("failed to read build timestamp {timestamp_path}: {error}"));
    let timestamp = timestamp.trim();
    if !is_valid_build_timestamp_utc(timestamp) {
        panic!("invalid canonical UTC build timestamp; run `just build`");
    }
    timestamp.to_owned()
}

fn is_valid_build_timestamp_utc(timestamp: &str) -> bool {
    let bytes = timestamp.as_bytes();
    bytes.len() == 20
        && matches!(
            bytes,
            [
                b'0'..=b'9',
                b'0'..=b'9',
                b'0'..=b'9',
                b'0'..=b'9',
                b'-',
                b'0'..=b'9',
                b'0'..=b'9',
                b'-',
                b'0'..=b'9',
                b'0'..=b'9',
                b'T',
                b'0'..=b'9',
                b'0'..=b'9',
                b':',
                b'0'..=b'9',
                b'0'..=b'9',
                b':',
                b'0'..=b'9',
                b'0'..=b'9',
                b'Z'
            ]
        )
        && timestamp[5..7]
            .parse::<u8>()
            .is_ok_and(|month| (1..=12).contains(&month))
        && timestamp[8..10]
            .parse::<u8>()
            .is_ok_and(|day| (1..=31).contains(&day))
        && timestamp[11..13].parse::<u8>().is_ok_and(|hour| hour <= 23)
        && timestamp[14..16]
            .parse::<u8>()
            .is_ok_and(|minute| minute <= 59)
        && timestamp[17..19]
            .parse::<u8>()
            .is_ok_and(|second| second <= 59)
}
