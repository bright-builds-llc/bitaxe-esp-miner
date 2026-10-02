#![cfg_attr(target_os = "espidf", feature(asm_experimental_arch))]

//! Conspicuous virtual guest. It has no GPIO, USB, Wi-Fi or real board adapters.
#[used]
#[no_mangle]
pub static BITAXE_VIRTUAL_EXECUTION_PROFILE: [u8; 44] =
    *b"BITAXE_EXECUTION_PROFILE=virtual-ultra205\0\0\0";

#[cfg(not(target_os = "espidf"))]
fn main() {}

#[cfg(target_os = "espidf")]
mod checkpoint;
#[cfg(target_os = "espidf")]
mod guest;
#[cfg(target_os = "espidf")]
mod noise_probe;

#[cfg(target_os = "espidf")]
fn main() {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
    if let Err(error) = guest::run() {
        log::error!("virtual_guest_failed: {error:#}");
    }
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
