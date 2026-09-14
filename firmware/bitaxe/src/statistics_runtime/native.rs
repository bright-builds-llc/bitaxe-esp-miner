//! Read the exact pthread stack capability class without changing task configuration.
use super::diagnostics::HeapObservation;
use esp_idf_svc::sys;

pub(super) fn stack_capabilities() -> Option<u32> {
    let mut config = unsafe { sys::esp_pthread_get_default_config() };
    let result = unsafe { sys::esp_pthread_get_cfg(&mut config) };
    if result == sys::ESP_ERR_NOT_FOUND {
        config = unsafe { sys::esp_pthread_get_default_config() };
    } else if result != sys::ESP_OK {
        return None;
    }
    let caps = if config.stack_alloc_caps == 0 {
        sys::MALLOC_CAP_INTERNAL | sys::MALLOC_CAP_8BIT
    } else {
        config.stack_alloc_caps
    };
    (caps & sys::MALLOC_CAP_8BIT != 0).then_some(caps)
}

pub(super) fn heap(caps: u32) -> HeapObservation {
    let mut info = std::mem::MaybeUninit::<sys::multi_heap_info_t>::uninit();
    // ESP-IDF initializes the whole output and derives each heap's free/largest
    // pair together; separate queries could produce an impossible mixed pair.
    let info = unsafe {
        sys::heap_caps_get_info(info.as_mut_ptr(), caps);
        info.assume_init()
    };
    HeapObservation {
        free_bytes: info.total_free_bytes as u32,
        largest_block_bytes: info.largest_free_block as u32,
    }
}
