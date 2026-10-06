//! ESP-IDF `www` partition adapter for `/api/system/OTAWWW`.
//!
//! Reference breadcrumbs:
//! - `reference/esp-miner/main/http_server/http_server.c:POST_WWW_update`
//! - `crates/bitaxe-api/src/www_update.rs` for every admission and offset decision

use std::{thread, time::Duration};

use bitaxe_api::www_update::{
    admit_www_body, WwwReceive, WwwStep, WwwUpdateStatus, WwwWritePlan, WWW_RECEIVE_CHUNK_BYTES,
    WWW_YIELD_MILLIS,
};
use bitaxe_api::PublicHttpResponse;
use esp_idf_svc::sys;

/// Result of an attempted `www` partition update.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WwwUpdateResult {
    /// The body was refused before any erase.
    Rejected(PublicHttpResponse),
    /// Every byte was written over the erased partition.
    Complete { bytes_written: usize },
    /// The body stream failed after the erase; the partition is partial.
    ProtocolError { code: i32 },
    /// ESP-IDF rejected an erase or write; the partition is partial.
    WriteError { esp_err: sys::esp_err_t },
}

/// Replaces the mounted `www` SPIFFS partition with the HTTP request body.
///
/// Like upstream, the partition stays mounted: static requests served while
/// this runs may fail, and nothing restages the old image on failure.
pub fn stream_www_update(
    request: *mut sys::httpd_req_t,
    mut status_sink: impl FnMut(WwwUpdateStatus),
) -> WwwUpdateResult {
    status_sink(WwwUpdateStatus::Starting);

    let partition = unsafe {
        sys::esp_partition_find_first(
            sys::esp_partition_type_t_ESP_PARTITION_TYPE_DATA,
            sys::esp_partition_subtype_t_ESP_PARTITION_SUBTYPE_DATA_SPIFFS,
            c"www".as_ptr(),
        )
    };
    let maybe_partition_size =
        (!partition.is_null()).then(|| unsafe { (*partition).size } as usize);
    let content_len = unsafe { (*request).content_len };
    let plan = match admit_www_body(content_len, maybe_partition_size) {
        Ok(plan) => plan,
        Err(response) => return WwwUpdateResult::Rejected(response),
    };

    if let Err(esp_err) = erase_partition(partition, plan) {
        status_sink(WwwUpdateStatus::WriteError);
        return WwwUpdateResult::WriteError { esp_err };
    }

    stream_body(request, partition, plan, &mut status_sink)
}

fn erase_partition(
    partition: *const sys::esp_partition_t,
    plan: WwwWritePlan,
) -> Result<(), sys::esp_err_t> {
    for (offset, len) in plan.erase_ranges() {
        let result = unsafe { sys::esp_partition_erase_range(partition, offset, len) };
        if result != sys::ESP_OK {
            return Err(result);
        }
        thread::sleep(Duration::from_millis(WWW_YIELD_MILLIS));
    }
    Ok(())
}

fn stream_body(
    request: *mut sys::httpd_req_t,
    partition: *const sys::esp_partition_t,
    plan: WwwWritePlan,
    status_sink: &mut impl FnMut(WwwUpdateStatus),
) -> WwwUpdateResult {
    let mut cursor = plan.cursor();
    let mut buffer = [0_u8; WWW_RECEIVE_CHUNK_BYTES];

    while !cursor.is_complete() {
        let recv_len = unsafe {
            sys::httpd_req_recv(request, buffer.as_mut_ptr().cast(), cursor.next_read_len())
        };
        let (offset, len) = match cursor.on_receive(classify_receive(recv_len)) {
            WwwStep::Retry => continue,
            WwwStep::ProtocolError { code } => {
                status_sink(WwwUpdateStatus::ProtocolError);
                return WwwUpdateResult::ProtocolError { code };
            }
            WwwStep::Write { offset, len } => (offset, len),
        };

        let result =
            unsafe { sys::esp_partition_write(partition, offset, buffer.as_ptr().cast(), len) };
        if result != sys::ESP_OK {
            status_sink(WwwUpdateStatus::WriteError);
            return WwwUpdateResult::WriteError { esp_err: result };
        }

        let written = cursor.wrote(len);
        status_sink(written.status);
        if written.yield_now {
            thread::sleep(Duration::from_millis(WWW_YIELD_MILLIS));
        }
    }

    WwwUpdateResult::Complete {
        bytes_written: plan.size(),
    }
}

fn classify_receive(recv_len: i32) -> WwwReceive {
    if recv_len == sys::HTTPD_SOCK_ERR_TIMEOUT {
        return WwwReceive::Timeout;
    }
    match usize::try_from(recv_len) {
        Ok(len) => WwwReceive::Data(len),
        Err(_) => WwwReceive::Failed(recv_len),
    }
}
