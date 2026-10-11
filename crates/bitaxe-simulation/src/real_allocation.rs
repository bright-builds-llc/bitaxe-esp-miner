//! A capability-aware test allocator that fails real allocations by site.
//!
//! Production code marks each recoverable reservation with a named site
//! (`bitaxe_core::fallible`). This allocator routes every attempt made inside an armed
//! site as firmware routes a default-capability heap allocation (PSRAM first, then
//! internal; `bitaxe_virtual_board::memory::route`) and fails it only when every
//! capability on that route is marked exhausted, as ESP-IDF malloc does. Attempts are
//! logged, so a test proves the armed site was actually reached. Arming is per thread;
//! everything else passes through to the system allocator.

use bitaxe_core::fallible::{self, Hooks, Site};
use bitaxe_virtual_board::memory::{route, Capability, FIRMWARE_DEFAULT_INTERNAL_THRESHOLD};
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::sync::Once;

/// Which capabilities an armed site finds exhausted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Exhausted {
    pub psram: bool,
    pub internal: bool,
}

impl Exhausted {
    pub(crate) const HEAP: Self = Self {
        psram: true,
        internal: true,
    };
    pub(crate) const PSRAM: Self = Self {
        psram: true,
        internal: false,
    };

    fn covers(self, capability: Capability) -> bool {
        match capability {
            Capability::Psram => self.psram,
            Capability::Internal | Capability::Dma | Capability::Default => self.internal,
        }
    }
}

/// One allocation attempt inside an armed site.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Attempt {
    pub site: &'static str,
    pub bytes: usize,
    /// The capability that served it, or `None` when every routed capability was exhausted.
    pub maybe_served_by: Option<Capability>,
}

const LOG_CAPACITY: usize = 16;

thread_local! {
    static SITE: Cell<Option<&'static str>> = const { Cell::new(None) };
    static ARMED: Cell<Option<(&'static str, Exhausted)>> = const { Cell::new(None) };
    static LOG: [Cell<Option<Attempt>>; LOG_CAPACITY] = const { [const { Cell::new(None) }; LOG_CAPACITY] };
    static LOGGED: Cell<usize> = const { Cell::new(0) };
}

pub(crate) struct Intercepting;

// SAFETY: every path delegates to `System` or returns null, which callers of
// `try_reserve` handle; the bookkeeping touches only thread-local cells.
unsafe impl GlobalAlloc for Intercepting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if intercepted(layout.size()) {
            return std::ptr::null_mut();
        }
        System.alloc(layout)
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        System.dealloc(pointer, layout);
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        if intercepted(size) {
            return std::ptr::null_mut();
        }
        System.realloc(pointer, layout, size)
    }
}

#[global_allocator]
static ALLOCATOR: Intercepting = Intercepting;

/// True when an armed site is active on this thread and its route is exhausted.
fn intercepted(bytes: usize) -> bool {
    let Ok(Some(site)) = SITE.try_with(Cell::get) else {
        return false;
    };
    let Ok(Some((armed_site, exhausted))) = ARMED.try_with(Cell::get) else {
        return false;
    };
    if site != armed_site {
        return false;
    }
    let maybe_served_by = route(
        Capability::Default,
        bytes,
        FIRMWARE_DEFAULT_INTERNAL_THRESHOLD,
    )
    .iter()
    .copied()
    .find(|capability| !exhausted.covers(*capability));
    record(Attempt {
        site,
        bytes,
        maybe_served_by,
    });
    maybe_served_by.is_none()
}

fn record(attempt: Attempt) {
    let _ = LOGGED.try_with(|logged| {
        let index = logged.get();
        if index < LOG_CAPACITY {
            let _ = LOG.try_with(|log| log[index].set(Some(attempt)));
            logged.set(index + 1);
        }
    });
}

/// Fails real allocations at `site` on this thread while the guard lives.
#[must_use]
pub(crate) fn arm(site: Site, exhausted: Exhausted) -> Armed {
    static INSTALL: Once = Once::new();
    INSTALL.call_once(|| {
        fallible::install(Hooks {
            begin: |site| SITE.with(|current| current.set(Some(site.0))),
            end: || SITE.with(|current| current.set(None)),
        });
    });
    LOGGED.with(|logged| logged.set(0));
    ARMED.with(|armed| armed.set(Some((site.0, exhausted))));
    Armed
}

pub(crate) struct Armed;

impl Armed {
    /// The attempts made at the armed site so far.
    pub(crate) fn attempts(&self) -> Vec<Attempt> {
        let count = LOGGED.with(Cell::get);
        LOG.with(|log| log[..count].iter().filter_map(Cell::get).collect())
    }
}

impl Drop for Armed {
    fn drop(&mut self) {
        ARMED.with(|armed| armed.set(None));
    }
}

#[cfg(test)]
mod tests;
