use super::super::StratumV2Error;

pub(super) struct Writer {
    bytes: Vec<u8>,
}

impl Writer {
    pub(super) fn new() -> Self {
        Self { bytes: Vec::new() }
    }

    pub(super) fn u8(&mut self, value: u8) {
        self.bytes.push(value);
    }

    pub(super) fn u16(&mut self, value: u16) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    pub(super) fn u32(&mut self, value: u32) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    pub(super) fn f32(&mut self, value: f32) {
        self.u32(value.to_bits());
    }

    pub(super) fn fixed(&mut self, value: &[u8]) {
        self.bytes.extend_from_slice(value);
    }

    pub(super) fn str0255(
        &mut self,
        field: &'static str,
        value: &str,
    ) -> Result<(), StratumV2Error> {
        let len = u8::try_from(value.len()).map_err(|_| StratumV2Error::InvalidField {
            field,
            reason: "exceeds 255 bytes",
        })?;
        self.u8(len);
        self.fixed(value.as_bytes());
        Ok(())
    }

    pub(super) fn bytes0255(
        &mut self,
        field: &'static str,
        value: &[u8],
    ) -> Result<(), StratumV2Error> {
        let len = u8::try_from(value.len()).map_err(|_| StratumV2Error::InvalidField {
            field,
            reason: "exceeds 255 bytes",
        })?;
        self.u8(len);
        self.fixed(value);
        Ok(())
    }

    pub(super) fn finish(self) -> Vec<u8> {
        self.bytes
    }
}

pub(super) struct Reader<'a> {
    bytes: &'a [u8],
    cursor: usize,
}

impl<'a> Reader<'a> {
    pub(super) const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, cursor: 0 }
    }

    pub(super) fn u8(&mut self, field: &'static str) -> Result<u8, StratumV2Error> {
        let bytes = self.take(field, 1)?;
        Ok(bytes[0])
    }

    pub(super) fn u16(&mut self, field: &'static str) -> Result<u16, StratumV2Error> {
        let bytes = self.take(field, 2)?;
        Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
    }

    pub(super) fn u32(&mut self, field: &'static str) -> Result<u32, StratumV2Error> {
        let bytes = self.take(field, 4)?;
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    pub(super) fn u64(&mut self, field: &'static str) -> Result<u64, StratumV2Error> {
        let bytes = self.take(field, 8)?;
        Ok(u64::from_le_bytes(
            bytes
                .try_into()
                .expect("bounded eight-byte field must convert"),
        ))
    }

    pub(super) fn fixed<const N: usize>(
        &mut self,
        field: &'static str,
    ) -> Result<[u8; N], StratumV2Error> {
        let bytes = self.take(field, N)?;
        Ok(bytes
            .try_into()
            .expect("bounded fixed field must have requested length"))
    }

    pub(super) fn str0255(&mut self, field: &'static str) -> Result<String, StratumV2Error> {
        let len = usize::from(self.u8(field)?);
        let bytes = self.take(field, len)?;
        String::from_utf8(bytes.to_vec()).map_err(|_| StratumV2Error::InvalidField {
            field,
            reason: "is not UTF-8",
        })
    }

    pub(super) fn bytes0255(
        &mut self,
        field: &'static str,
        maximum: usize,
    ) -> Result<Vec<u8>, StratumV2Error> {
        let len = usize::from(self.u8(field)?);
        if len > maximum {
            return Err(StratumV2Error::InvalidField {
                field,
                reason: "exceeds the field bound",
            });
        }
        Ok(self.take(field, len)?.to_vec())
    }

    pub(super) fn bytes064k(
        &mut self,
        field: &'static str,
        maximum: usize,
    ) -> Result<Vec<u8>, StratumV2Error> {
        let len = usize::from(self.u16(field)?);
        if len > maximum {
            return Err(StratumV2Error::InvalidField {
                field,
                reason: "exceeds the field bound",
            });
        }
        Ok(self.take(field, len)?.to_vec())
    }

    pub(super) fn finish(self) -> Result<(), StratumV2Error> {
        if self.cursor == self.bytes.len() {
            Ok(())
        } else {
            Err(StratumV2Error::TrailingPayload)
        }
    }

    fn take(&mut self, field: &'static str, len: usize) -> Result<&'a [u8], StratumV2Error> {
        let end = self
            .cursor
            .checked_add(len)
            .ok_or(StratumV2Error::TruncatedField { field })?;
        let maybe_bytes = self.bytes.get(self.cursor..end);
        let Some(bytes) = maybe_bytes else {
            return Err(StratumV2Error::TruncatedField { field });
        };
        self.cursor = end;
        Ok(bytes)
    }
}
