use bitaxe_api::BuildProvenance;
use std::env;
use std::fs;

fn main() {
    embuild::espidf::sysenv::output();
    assert_console_contract();
    println!("cargo:rerun-if-env-changed=BITAXE_BUILD_PROVENANCE_STAMP");
    println!("cargo:rerun-if-env-changed=BITAXE_MINING_EVIDENCE_MODE");
    println!("cargo:rerun-if-env-changed=BITAXE_HARDWARE_EVIDENCE_ACK");
    println!("cargo:rerun-if-env-changed=BITAXE_WORK_RESULT_INVESTIGATION");
    println!("cargo:rerun-if-env-changed=BITAXE_CHIP_DETECT_INVESTIGATION");
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
}

fn assert_console_contract() {
    const REQUIRED_DEFAULTS: [&str; 3] = [
        "CONFIG_ESP_CONSOLE_UART_DEFAULT=y",
        "CONFIG_ESP_CONSOLE_UART_BAUDRATE=115200",
        "CONFIG_ESP_CONSOLE_SECONDARY_USB_SERIAL_JTAG=y",
    ];

    println!("cargo:rerun-if-changed=sdkconfig.defaults");
    let Ok(defaults) = fs::read_to_string("sdkconfig.defaults") else {
        panic!("firmware build requires readable sdkconfig.defaults");
    };
    for required in REQUIRED_DEFAULTS {
        if !defaults.lines().any(|line| line == required) {
            panic!("firmware console contract missing {required}");
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
