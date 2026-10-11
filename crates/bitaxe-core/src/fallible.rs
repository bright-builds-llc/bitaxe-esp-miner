//! Recoverable allocation regions.
//!
//! Firmware records every failed heap allocation and keeps the first one per boot as
//! the fatal-failure receipt. A reservation whose failure the caller handles is
//! recoverable: it runs inside a region, so the recorder keeps it in its history but
//! never in the fatal receipt. On firmware the region is marked natively; a host may
//! install hooks instead, for example to fail allocations by site.

use std::collections::TryReserveError;

/// One recoverable allocation site, named for hosts that observe or fail it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Site(pub &'static str);

/// Host callbacks around every recoverable reservation, in this thread.
#[derive(Clone, Copy, Debug)]
pub struct Hooks {
    pub begin: fn(Site),
    pub end: fn(),
}

/// Installs the process-wide host hooks once; false when hooks were already installed.
pub fn install(hooks: Hooks) -> bool {
    host::install(hooks)
}

/// Runs one allocation attempt whose failure the caller handles.
pub fn recoverable<R>(site: Site, attempt: impl FnOnce() -> R) -> R {
    let _region = Region::open(site);
    attempt()
}

/// `Vec::try_reserve_exact` inside a recoverable region.
pub fn reserve_exact<T>(
    vec: &mut Vec<T>,
    additional: usize,
    site: Site,
) -> Result<(), TryReserveError> {
    recoverable(site, || vec.try_reserve_exact(additional))
}

/// `Vec::try_reserve` inside a recoverable region.
pub fn reserve<T>(vec: &mut Vec<T>, additional: usize, site: Site) -> Result<(), TryReserveError> {
    recoverable(site, || vec.try_reserve(additional))
}

/// A copy of `text` whose allocation is recoverable.
pub fn try_string(text: &str, site: Site) -> Result<String, TryReserveError> {
    let mut copy = String::new();
    recoverable(site, || copy.try_reserve_exact(text.len()))?;
    copy.push_str(text);
    Ok(copy)
}

/// Closes its region even if the attempt unwinds.
struct Region;

impl Region {
    fn open(site: Site) -> Self {
        native::begin();
        host::begin(site);
        Self
    }
}

impl Drop for Region {
    fn drop(&mut self) {
        host::end();
        native::end();
    }
}

#[cfg(all(feature = "native-fault-provenance", target_os = "espidf"))]
mod native {
    extern "C" {
        fn bitaxe_allocation_recoverable_begin();
        fn bitaxe_allocation_recoverable_end();
    }

    pub(super) fn begin() {
        // SAFETY: the provenance component exports both functions; they only mark the
        // calling task in internal DRAM and touch no Rust state.
        unsafe { bitaxe_allocation_recoverable_begin() }
    }

    pub(super) fn end() {
        // SAFETY: as for `begin`; an end without a matching begin is ignored.
        unsafe { bitaxe_allocation_recoverable_end() }
    }
}

#[cfg(not(all(feature = "native-fault-provenance", target_os = "espidf")))]
mod native {
    pub(super) fn begin() {}
    pub(super) fn end() {}
}

#[cfg(not(target_os = "espidf"))]
mod host {
    use super::{Hooks, Site};
    use std::sync::OnceLock;

    static HOOKS: OnceLock<Hooks> = OnceLock::new();

    pub(super) fn install(hooks: Hooks) -> bool {
        HOOKS.set(hooks).is_ok()
    }

    pub(super) fn begin(site: Site) {
        if let Some(hooks) = HOOKS.get() {
            (hooks.begin)(site);
        }
    }

    pub(super) fn end() {
        if let Some(hooks) = HOOKS.get() {
            (hooks.end)();
        }
    }
}

/// Firmware has no host hooks, so no indirect call joins an audited path.
#[cfg(target_os = "espidf")]
mod host {
    use super::{Hooks, Site};

    pub(super) fn install(_: Hooks) -> bool {
        false
    }

    pub(super) fn begin(_: Site) {}

    pub(super) fn end() {}
}

#[cfg(test)]
mod tests;
