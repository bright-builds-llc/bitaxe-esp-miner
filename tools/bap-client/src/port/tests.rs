use super::*;
use crate::exchange::{run, Operation, Outcome, Timing};
use bitaxe_core::bap::{BapCommand, BapFrame, BapParameter};
use std::ffi::CStr;
use std::os::fd::FromRawFd;

#[test]
fn an_adapter_callout_node_is_admitted() {
    assert_eq!(admit_adapter_path("/dev/cu.usbserial-1410"), Ok(()));
    assert_eq!(admit_adapter_path("/dev/cu.wchusbserial1420"), Ok(()));
}

#[test]
fn the_board_usb_node_is_refused() {
    assert_eq!(
        admit_adapter_path("/dev/cu.usbmodem1101"),
        Err(PathRefusal::BoardUsbNode)
    );
}

#[test]
fn a_non_callout_path_is_refused() {
    for path in [
        "/dev/tty.usbserial-1410",
        "/tmp/cu.fake",
        "/dev/cu.",
        "/dev/cu.a/b",
    ] {
        assert_eq!(
            admit_adapter_path(path),
            Err(PathRefusal::NotCalloutNode),
            "{path}"
        );
    }
}

/// A pseudo-terminal pair: the client opens the device side by path while the test
/// holds both ends open and plays the board on the primary.
fn pseudo_terminal() -> (File, File, String) {
    let mut primary = -1;
    let mut secondary = -1;
    let mut name = [0 as libc::c_char; 1024];
    // SAFETY: valid out-pointers; the null termios/winsize arguments are permitted.
    let opened = unsafe {
        libc::openpty(
            &mut primary,
            &mut secondary,
            name.as_mut_ptr(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    };
    assert_eq!(opened, 0, "pseudo-terminal pair must open");
    // SAFETY: openpty wrote a NUL-terminated device name.
    let path = unsafe { CStr::from_ptr(name.as_ptr()) }
        .to_str()
        .expect("utf8")
        .to_owned();
    // SAFETY: openpty returned two open descriptors this test now owns.
    let (board, device) = unsafe { (File::from_raw_fd(primary), File::from_raw_fd(secondary)) };
    (board, device, path)
}

#[test]
fn a_request_round_trips_over_a_serial_line() {
    // Arrange
    let (mut board, _device, path) = pseudo_terminal();
    let mut link = AdapterLink::open(&path).expect("adapter");
    let answer = BapFrame::new(
        BapCommand::Response,
        BapParameter::Hashrate,
        Some("512.3".into()),
    )
    .expect("frame")
    .encode()
    .expect("encode");
    let responder = std::thread::spawn(move || {
        let mut request = Vec::new();
        let mut byte = [0_u8; 1];
        while !request.ends_with(b"\n") {
            board.read_exact(&mut byte).expect("request byte");
            request.extend_from_slice(&byte);
        }
        board.write_all(answer.as_bytes()).expect("answer");
        // Closing the board end could discard the unread answer, so it stays open.
        (request, board)
    });
    let timing = Timing {
        answer_ms: 2_000,
        quiet_ms: 100,
        listen_ms: 0,
    };
    // Act
    let report = run(
        &Operation::Request(BapParameter::Hashrate),
        &mut link,
        timing,
    )
    .expect("run");
    // Assert
    let (request, _board) = responder.join().expect("responder");
    let sent = BapFrame::parse(&request).expect("request frame");
    assert_eq!(sent.command(), BapCommand::Request);
    assert_eq!(report.outcome, Outcome::Answered);
    assert_eq!(report.frames[0].value(), Some("512.3"));
}

#[test]
fn a_hung_up_adapter_is_an_error_not_a_quiet_line() {
    // Arrange
    let (board, _device, path) = pseudo_terminal();
    let mut link = AdapterLink::open(&path).expect("adapter");
    // Act
    drop(board);
    let result = link.receive(500);
    // Assert
    assert_eq!(
        result.expect_err("hang-up").kind(),
        io::ErrorKind::UnexpectedEof
    );
}
