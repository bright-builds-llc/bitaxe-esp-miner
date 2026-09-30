fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("espidf") {
        println!("cargo:rerun-if-env-changed=BITAXE_VIRTUAL_SOURCE_DIGEST");
        let digest = std::env::var("BITAXE_VIRTUAL_SOURCE_DIGEST")
            .expect("virtual build requires compiled source digest");
        assert!(digest.len() == 64 && digest.bytes().all(|byte| byte.is_ascii_hexdigit()));
        println!("cargo:rustc-env=BITAXE_VIRTUAL_SOURCE_DIGEST={digest}");
        embuild::espidf::sysenv::output();
        println!("cargo:rustc-link-arg=-Wl,--wrap=adc_hal_self_calibration");
        println!("cargo:rustc-link-arg=-Wl,-u,BITAXE_VIRTUAL_EXECUTION_PROFILE");
    }
}
