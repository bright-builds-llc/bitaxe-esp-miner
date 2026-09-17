//! The existing pool lane owns V2 crypto and socket I/O; the owner keeps safety.
use super::*;
use bitaxe_stratum::v2::frame::Frame;

#[inline(never)]
pub(super) fn run(
    pool: ProductionPool,
    command: &PoolTransportCommand,
    receiver: &Receiver<PoolTransportCommand>,
    close: &AtomicU32,
    borrow: &NoiseBorrowWorker,
    emit: &impl Fn(PoolTransportEvent),
) -> bool {
    let PoolTransportCommand::ConnectV2 {
        transport_epoch: epoch,
        endpoint,
        authority,
        permit,
        generation,
    } = command
    else {
        return false;
    };
    let mut io = OwnerIo {
        pool,
        epoch: *epoch,
        receiver,
        close,
        borrow,
        emit,
        open: true,
        keep_running: true,
    };
    let result = crate::v2_serial_runtime::run_share_transport(
        *endpoint,
        *authority,
        *permit,
        *generation,
        *epoch,
        &mut io,
    );
    if result.is_err() && revocation::permits_work(*permit) {
        (io.emit)(PoolTransportEvent::Failed {
            pool,
            transport_epoch: *epoch,
            failure: ProductionTransportFailure::Read,
        });
    }
    drain_completed(&mut io)
}
#[inline(never)]
fn drain_completed<F: Fn(PoolTransportEvent)>(io: &mut OwnerIo<'_, F>) -> bool {
    let epoch = io.epoch;
    // All socket/crypto/input scopes returned. Commands already stamped by a
    // revoked generation cannot become work on the persistent lane afterward.
    loop {
        match io.receiver.try_recv() {
            Ok(command) => {
                io.borrow.begin_command();
                match command {
                    PoolTransportCommand::WriteFrame {
                        transport_epoch, ..
                    }
                    | PoolTransportCommand::Close { transport_epoch }
                        if transport_epoch == epoch => {}
                    PoolTransportCommand::Shutdown => io.keep_running = false,
                    _ => {
                        io.keep_running = false;
                        crate::v2_serial_runtime::capture_failure();
                    }
                }
            }
            Err(mpsc::TryRecvError::Empty) => break,
            Err(mpsc::TryRecvError::Disconnected) => {
                io.keep_running = false;
                break;
            }
        }
    }
    io.keep_running
}
struct OwnerIo<'a, F> {
    pool: ProductionPool,
    epoch: ProductionTransportEpoch,
    receiver: &'a Receiver<PoolTransportCommand>,
    close: &'a AtomicU32,
    borrow: &'a NoiseBorrowWorker,
    emit: &'a F,
    open: bool,
    keep_running: bool,
}
impl<F: Fn(PoolTransportEvent)> crate::v2_serial_runtime::ShareIo for OwnerIo<'_, F> {
    fn authenticated(&mut self) {
        (self.emit)(PoolTransportEvent::Connected {
            pool: self.pool,
            transport_epoch: self.epoch,
        });
    }
    fn open(&self) -> bool {
        self.open && self.close.load(Ordering::Acquire) != epoch_word(self.epoch)
    }
    fn next_write(&mut self) -> Result<Option<(Frame, WorkPermit)>, ()> {
        match self.receiver.try_recv() {
            Ok(command) => {
                self.borrow.begin_command();
                match command {
                    PoolTransportCommand::WriteFrame {
                        transport_epoch,
                        frame,
                        permit,
                    } if transport_epoch == self.epoch => Ok(Some((frame, permit))),
                    PoolTransportCommand::Close { transport_epoch }
                        if transport_epoch == self.epoch =>
                    {
                        self.open = false;
                        Ok(None)
                    }
                    PoolTransportCommand::Shutdown => {
                        self.open = false;
                        self.keep_running = false;
                        Ok(None)
                    }
                    _ => Err(()),
                }
            }
            Err(mpsc::TryRecvError::Empty) => Ok(None),
            Err(mpsc::TryRecvError::Disconnected) => {
                self.open = false;
                self.keep_running = false;
                Err(())
            }
        }
    }
    fn frame(&mut self, frame: Frame) {
        (self.emit)(PoolTransportEvent::Frame {
            pool: self.pool,
            transport_epoch: self.epoch,
            frame,
        });
    }
    fn written(&mut self, sequence: u32) {
        (self.emit)(PoolTransportEvent::FrameWritten {
            pool: self.pool,
            transport_epoch: self.epoch,
            sequence,
        });
    }
}
