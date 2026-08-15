#![allow(dead_code)]

use core::convert::Infallible;
use core::marker::PhantomData;

use embedded_hal::i2c::{ErrorType, I2c, Operation};

struct NoopI2c;

impl ErrorType for NoopI2c {
    type Error = Infallible;
}

impl I2c for NoopI2c {
    fn transaction(
        &mut self,
        _address: u8,
        operations: &mut [Operation<'_>],
    ) -> Result<(), Self::Error> {
        for operation in operations {
            if let Operation::Read(bytes) = operation {
                bytes.fill(0);
            }
        }
        Ok(())
    }
}

mod safety_adapter {
    use super::{NoopI2c, PhantomData};

    pub(crate) struct BitaxeI2cBus<'d> {
        _lifetime: PhantomData<&'d ()>,
    }

    impl BitaxeI2cBus<'_> {
        pub(crate) fn startup_display(&mut self) -> NoopI2c {
            NoopI2c
        }
    }

    pub(crate) struct RuntimeI2cOwner<'d> {
        _lifetime: PhantomData<&'d ()>,
    }

    pub(crate) struct RuntimeI2cBudget;

    impl RuntimeI2cOwner<'_> {
        pub(crate) fn display(&mut self, _budget: &mut RuntimeI2cBudget) -> NoopI2c {
            NoopI2c
        }
    }
}

#[path = "display_adapter.rs"]
mod display_adapter;
