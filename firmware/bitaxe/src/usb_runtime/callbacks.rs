use std::ptr;

use esp_idf_sys as sys;

use super::tinyusb;

const WORKER_INTERFACE: u8 = 0;

#[no_mangle]
extern "C" fn tud_mount_cb() {
    crate::bwg_worker_usb::enqueue_attached();
}

#[no_mangle]
extern "C" fn tud_umount_cb() {
    crate::bwg_worker_usb::enqueue_detached();
}

#[no_mangle]
unsafe extern "C" fn tud_vendor_rx_cb(interface: u8, bytes: *const u8, length: u16) {
    if interface != WORKER_INTERFACE {
        return;
    }
    if length > 0 {
        if bytes.is_null() {
            crate::bwg_worker_usb::note_ingress_lost();
            return;
        }
        let bytes = unsafe { std::slice::from_raw_parts(bytes, usize::from(length)) };
        crate::bwg_worker_usb::enqueue_vendor_bytes(bytes);
        return;
    }
    let mut chunk = [0_u8; 256];
    while tinyusb::vendor_available() > 0 {
        let received = tinyusb::read_vendor(&mut chunk);
        if received == 0 || received > chunk.len() {
            crate::bwg_worker_usb::note_ingress_lost();
            return;
        }
        crate::bwg_worker_usb::enqueue_vendor_bytes(&chunk[..received]);
    }
}

#[no_mangle]
extern "C" fn tud_cdc_rx_cb(interface: u8) {
    let mut discarded = [0_u8; 64];
    tinyusb::discard_cdc(interface, &mut discarded);
}

#[no_mangle]
unsafe extern "C" fn tud_cdc_line_coding_cb(interface: u8, coding: *const sys::cdc_line_coding_t) {
    if interface != 0 || coding.is_null() {
        return;
    }
    let bit_rate = unsafe { ptr::addr_of!((*coding).bit_rate).read_unaligned() };
    crate::bwg_worker_usb::enqueue_line_coding(bit_rate);
}

#[no_mangle]
extern "C" fn tud_cdc_line_state_cb(interface: u8, dtr: bool, rts: bool) {
    if interface == 0 {
        crate::bwg_worker_usb::enqueue_line_state(dtr, rts);
    }
}
