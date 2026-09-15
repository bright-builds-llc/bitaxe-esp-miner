//! Reserve the larger HTTP task before the deferred worker can fragment its heap.

pub(super) fn initialize<S, E>(
    server: impl FnOnce() -> Result<S, E>,
    deferred: impl FnOnce() -> Result<(), E>,
) -> Result<S, E> {
    let server = server()?;
    deferred()?;
    Ok(server)
}

#[cfg(test)]
#[path = "startup_owners_tests.rs"]
mod tests;
