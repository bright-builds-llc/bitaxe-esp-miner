//! Effect-order seam for preparing Noise before opening a responder socket.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrepareBeforeConnectError<PreparationError, ConnectionError> {
    Preparation(PreparationError),
    Connection(ConnectionError),
}

pub fn prepare_before_connect<
    Prepared,
    Connected,
    PreparationError,
    ConnectionError,
    Prepare,
    Connect,
>(
    prepare: Prepare,
    connect: Connect,
) -> Result<(Prepared, Connected), PrepareBeforeConnectError<PreparationError, ConnectionError>>
where
    Prepare: FnOnce() -> Result<Prepared, PreparationError>,
    Connect: FnOnce() -> Result<Connected, ConnectionError>,
{
    let prepared = prepare().map_err(PrepareBeforeConnectError::Preparation)?;
    let connected = connect().map_err(PrepareBeforeConnectError::Connection)?;
    Ok((prepared, connected))
}
