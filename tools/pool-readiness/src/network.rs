use std::{
    net::{Ipv4Addr, SocketAddr, TcpStream, ToSocketAddrs},
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

use sha2::{Digest, Sha256};

use super::{ReadinessCategory, ReadinessError};

pub(super) fn resolve_private_addresses(
    host: &str,
    port: u16,
    timeout: Duration,
) -> Result<Vec<SocketAddr>, ReadinessError> {
    let (sender, receiver) = mpsc::sync_channel(1);
    let owned_host = host.to_owned();
    thread::spawn(move || {
        let result = (owned_host.as_str(), port)
            .to_socket_addrs()
            .map(|addresses| addresses.collect::<Vec<_>>());
        let _ = sender.send(result);
    });
    let addresses = match receiver.recv_timeout(timeout) {
        Ok(Ok(addresses)) if !addresses.is_empty() => addresses,
        Ok(_) => return Err(ReadinessError::new(ReadinessCategory::ResolutionFailed)),
        Err(mpsc::RecvTimeoutError::Timeout) => {
            return Err(ReadinessError::new(ReadinessCategory::Timeout));
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            return Err(ReadinessError::new(ReadinessCategory::ResolutionFailed));
        }
    };
    if !addresses.iter().all(nonpublic_address) {
        return Err(ReadinessError::new(ReadinessCategory::EndpointNotPrivate));
    }
    Ok(addresses)
}

pub(super) fn rfc1918_only(addresses: &[SocketAddr]) -> bool {
    addresses.iter().all(|address| {
        let SocketAddr::V4(address) = address else {
            return false;
        };
        private_ipv4(*address.ip())
    })
}

pub(super) fn endpoint_set_sha256(addresses: &[SocketAddr]) -> String {
    let mut values = addresses
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    values.sort();
    values.dedup();
    Sha256::digest(values.join("\n"))
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

pub(super) fn connect(
    addresses: &[SocketAddr],
    deadline: Instant,
) -> Result<TcpStream, ReadinessError> {
    for address in addresses {
        let remaining = deadline
            .checked_duration_since(Instant::now())
            .filter(|remaining| !remaining.is_zero())
            .ok_or_else(|| ReadinessError::new(ReadinessCategory::Timeout))?;
        if let Ok(stream) = TcpStream::connect_timeout(address, remaining) {
            return Ok(stream);
        }
    }
    Err(ReadinessError::new(ReadinessCategory::ConnectionFailed))
}

fn nonpublic_address(address: &SocketAddr) -> bool {
    let SocketAddr::V4(address) = address else {
        return false;
    };
    let ip = address.ip();
    ip.is_loopback() || private_ipv4(*ip)
}

fn private_ipv4(ip: Ipv4Addr) -> bool {
    let [first, second, _, _] = ip.octets();
    first == 10 || (first == 172 && (16..=31).contains(&second)) || (first == 192 && second == 168)
}
