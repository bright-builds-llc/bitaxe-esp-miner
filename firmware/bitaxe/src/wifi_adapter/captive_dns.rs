use std::{
    net::{Ipv4Addr, SocketAddrV4, UdpSocket},
    thread,
};

use bitaxe_api::{build_captive_dns_response, CAPTIVE_DNS_PACKET_BYTES, CAPTIVE_DNS_PORT};

const THREAD_STACK_BYTES: usize = 8 * 1024;
const THREAD_NAME: &str = "captive-dns";

pub(super) fn start(ap_ipv4: Ipv4Addr) -> anyhow::Result<()> {
    let socket = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, CAPTIVE_DNS_PORT))?;
    thread::Builder::new()
        .name(THREAD_NAME.to_owned())
        .stack_size(THREAD_STACK_BYTES)
        .spawn(move || run(socket, ap_ipv4))
        .map_err(|error| anyhow::anyhow!("failed to start captive DNS owner: {error}"))?;
    Ok(())
}

fn run(socket: UdpSocket, ap_ipv4: Ipv4Addr) {
    let mut request = [0_u8; CAPTIVE_DNS_PACKET_BYTES];
    loop {
        let (request_len, peer) = match socket.recv_from(&mut request) {
            Ok(received) => received,
            Err(error) => {
                log::warn!(
                    "captive_dns=stopped reason=receive_failed kind={:?}",
                    error.kind()
                );
                return;
            }
        };

        let response = match build_captive_dns_response(&request[..request_len], ap_ipv4) {
            Ok(Some(response)) => response,
            Ok(None) => continue,
            Err(error) => {
                log::debug!("captive_dns=request_rejected category={error}");
                continue;
            }
        };
        if let Err(error) = socket.send_to(&response, peer) {
            log::warn!("captive_dns=response_failed kind={:?}", error.kind());
        }
    }
}
