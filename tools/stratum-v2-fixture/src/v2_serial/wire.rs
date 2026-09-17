use super::{
    input::Input,
    io::SecretFrame,
    model::{Error, Result, Submission},
    oracle::TARGET,
};
use std::net::SocketAddr;
pub(super) struct Cursor<'a> {
    bytes: &'a [u8],
    at: usize,
    stage: &'static str,
}
impl<'a> Cursor<'a> {
    fn bytes(&mut self, n: usize) -> Result<&'a [u8]> {
        let end = self
            .at
            .checked_add(n)
            .ok_or(Error::new(self.stage, "protocol"))?;
        let v = self
            .bytes
            .get(self.at..end)
            .ok_or(Error::new(self.stage, "protocol"))?;
        self.at = end;
        Ok(v)
    }
    fn byte(&mut self) -> Result<u8> {
        Ok(self.bytes(1)?[0])
    }
    fn u16(&mut self) -> Result<u16> {
        Ok(u16::from_le_bytes(
            self.bytes(2)?.try_into().expect("length checked"),
        ))
    }
    fn u32(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(
            self.bytes(4)?.try_into().expect("length checked"),
        ))
    }
    fn string(&mut self) -> Result<&'a str> {
        let n = usize::from(self.byte()?);
        std::str::from_utf8(self.bytes(n)?).map_err(|_| Error::new(self.stage, "protocol"))
    }
    fn finish(self) -> Result<()> {
        if self.at != self.bytes.len() {
            Err(Error::new(self.stage, "extra"))
        } else {
            Ok(())
        }
    }
}
fn cursor<'a>(
    frame: &'a SecretFrame,
    extension: u16,
    message: u8,
    stage: &'static str,
) -> Result<Cursor<'a>> {
    if frame.header.extension_type != extension || frame.header.message_type != message {
        return Err(Error::new(stage, "protocol"));
    }
    Ok(Cursor {
        bytes: &frame.payload,
        at: 0,
        stage,
    })
}
pub(super) fn setup(frame: &SecretFrame, address: SocketAddr) -> Result<()> {
    let mut c = cursor(frame, 0, 0, "setup_received")?;
    if c.byte()? != 0
        || c.u16()? != 2
        || c.u16()? != 2
        || c.u32()? != 5
        || c.string()? != address.ip().to_string()
        || c.u16()? != address.port()
    {
        return Err(Error::new("setup_received", "protocol"));
    }
    for _ in 0..4 {
        c.string()?;
    }
    c.finish()
}
pub(super) fn open(frame: &SecretFrame, input: &Input) -> Result<u32> {
    let mut c = cursor(frame, 0, 0x10, "channel_open_received")?;
    let request = c.u32()?;
    if c.string()? != input.user_identity.as_str() {
        return Err(Error::new("channel_open_received", "authentication"));
    }
    let rate = f32::from_bits(c.u32()?);
    let maximum: [u8; 32] = c.bytes(32)?.try_into().expect("length checked");
    if !rate.is_finite() || rate < 0.0 || !super::oracle::meets_target(&TARGET, &maximum) {
        return Err(Error::new("channel_open_received", "protocol"));
    }
    c.finish()?;
    Ok(request)
}
pub(super) fn share(frame: &SecretFrame) -> Result<Submission> {
    let mut c = cursor(frame, 0x8000, 0x1a, "share_received")?;
    let value = Submission {
        channel_id: c.u32()?,
        sequence_number: c.u32()?,
        job_id: c.u32()?,
        nonce: c.u32()?,
        ntime: c.u32()?,
        version: c.u32()?,
    };
    c.finish()?;
    Ok(value)
}
