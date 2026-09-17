use super::io::pause;
use super::model::{CandidateReceipt, Cause, Terminal};
use std::io::Read;
use std::net::{IpAddr, TcpListener, TcpStream};
use std::time::{Duration, Instant};

pub(crate) fn select(
    listener: &TcpListener,
    expected: IpAddr,
    accept_deadline: Instant,
    lifetime_deadline: Instant,
    read_timeout: Duration,
    receipt: &mut Terminal,
) -> Result<(TcpStream, [u8; 64]), Cause> {
    let mut peers: Vec<(TcpStream, [u8; 65], usize)> = Vec::new();
    let mut maybe_read_deadline = None;
    let mut maybe_selected_at = None;
    loop {
        let now = Instant::now();
        if now >= lifetime_deadline
            || (maybe_selected_at.is_none()
                && now >= maybe_read_deadline.unwrap_or(accept_deadline))
        {
            for candidate in &mut receipt.candidates {
                candidate.read_outcome = "timeout";
            }
            return Err(Cause::new("act_one_written", "read", "timeout"));
        }
        // A bounded accept slice prevents a connection flood starving the clock.
        for _ in 0..4 {
            if Instant::now() >= lifetime_deadline {
                return Err(Cause::new("candidate_inventory", "read", "timeout"));
            }
            match listener.accept() {
                Ok((_stream, peer)) if peer.ip() != expected => {
                    receipt.unexpected_peer_count += 1;
                    return Err(Cause::new(
                        "candidate_inventory",
                        "authentication",
                        "peer_conflict",
                    ));
                }
                Ok((stream, peer)) => {
                    receipt.expected_peer_connection_count += 1;
                    if peers.len() == 3 {
                        receipt.candidate_overflow = true;
                        return Err(Cause::new("candidate_inventory", "read", "overflow"));
                    }
                    stream
                        .set_nonblocking(true)
                        .map_err(|_| Cause::new("candidate_inventory", "read", "io"))?;
                    let accepted_at = Instant::now();
                    if maybe_read_deadline.is_none() && accepted_at >= accept_deadline {
                        return Err(Cause::new("candidate_inventory", "read", "timeout"));
                    }
                    maybe_read_deadline.get_or_insert(accepted_at + read_timeout);
                    receipt.candidates.push(CandidateReceipt {
                        remote_port: peer.port(),
                        act_one_bytes: 0,
                        read_outcome: "partial",
                    });
                    peers.push((stream, [0; 65], 0));
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(_) => return Err(Cause::new("candidate_inventory", "read", "io")),
            }
        }
        for (index, (stream, bytes, received)) in peers.iter_mut().enumerate() {
            let candidate = &mut receipt.candidates[index];
            if Instant::now() >= lifetime_deadline
                || (maybe_selected_at.is_none()
                    && Instant::now() >= maybe_read_deadline.unwrap_or(accept_deadline))
            {
                candidate.read_outcome = "timeout";
                return Err(Cause::new("act_one_written", "read", "timeout"));
            }
            match stream.read(&mut bytes[*received..]) {
                Ok(0) => {
                    candidate.read_outcome = "eof";
                    return Err(Cause::new("act_one_written", "read", "eof"));
                }
                Ok(count) => {
                    *received += count;
                    candidate.act_one_bytes = (*received)
                        .try_into()
                        .map_err(|_| Cause::new("act_one_written", "read", "overflow"))?;
                    if *received > 64 {
                        candidate.read_outcome = "malformed";
                        return Err(Cause::new("act_one_written", "read", "extra"));
                    }
                    if maybe_selected_at.is_none()
                        && Instant::now() >= maybe_read_deadline.unwrap_or(accept_deadline)
                    {
                        candidate.read_outcome = "timeout";
                        return Err(Cause::new("act_one_written", "read", "timeout"));
                    }
                    if *received == 64 {
                        candidate.read_outcome = "complete";
                        if receipt.selected_index.is_none() {
                            receipt.selected_index = Some(index);
                            maybe_selected_at = Some(Instant::now());
                        }
                    }
                }
                Err(error)
                    if matches!(
                        error.kind(),
                        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::Interrupted
                    ) => {}
                Err(_) => {
                    candidate.read_outcome = "io";
                    return Err(Cause::new("act_one_written", "read", "io"));
                }
            }
        }
        let now = Instant::now();
        if now >= lifetime_deadline {
            return Err(Cause::new("candidate_inventory", "read", "timeout"));
        }
        if maybe_selected_at.is_some_and(|at| now >= at + Duration::from_millis(500)) {
            if peers.len() != 1 {
                return Err(Cause::new(
                    "candidate_inventory",
                    "authentication",
                    "peer_conflict",
                ));
            }
            let Some((stream, bytes, _)) = peers.pop() else {
                return Err(Cause::new(
                    "candidate_inventory",
                    "evidence_incomplete",
                    "missing",
                ));
            };
            let mut act_one = [0; 64];
            act_one.copy_from_slice(&bytes[..64]);
            return Ok((stream, act_one));
        }
        // A selected complete candidate gets the full post-selection observation,
        // even if it arrived immediately before the read deadline.
        if maybe_selected_at.is_none() && now >= maybe_read_deadline.unwrap_or(accept_deadline) {
            for candidate in &mut receipt.candidates {
                candidate.read_outcome = "timeout";
            }
            return Err(Cause::new("act_one_written", "read", "timeout"));
        }
        pause(lifetime_deadline, "candidate_inventory")?;
    }
}

/// Includes connections arriving during responder/proof I/O in ownership rejection.
pub(crate) fn reject_later_peers(
    listener: &TcpListener,
    expected: IpAddr,
    receipt: &mut Terminal,
) -> Result<(), Cause> {
    match listener.accept() {
        Ok((_stream, peer)) => {
            if peer.ip() == expected {
                receipt.expected_peer_connection_count += 1;
            } else {
                receipt.unexpected_peer_count += 1;
            }
            Err(Cause::new(
                "candidate_inventory",
                "authentication",
                "peer_conflict",
            ))
        }
        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => Ok(()),
        Err(_) => Err(Cause::new("candidate_inventory", "read", "io")),
    }
}
