use embedded_hal::blocking::i2c;

use ch32v1::ch32v103::{ RCC, I2C1 };
// use crate::time::*;
use crate::rcc::*;
use crate::gpio::*;
use crate::gpio::gpiob::{ PB6, PB7 };

// define I2C error
#[derive(Debug)]
pub enum I2cError {
    // Bus Error (BERR)
    BusError,
    // Acknowledge Failure (AF)
    Nak,
    // Arbitration Lost (ARLO)
    ArbitrationError,
    // Overrun/ Underrun Error (OVR)
    OverrunError,
    // Crc check error
    CrcError,
    // Not specify error reason
    UnknownError,
}

pub enum I2cMode {
    Standard, // 100kHz
    Fast, // 400kHz
}

pub trait SclPin<I2C> {
    fn remap(&self) -> bool;
}
pub trait SdaPin<I2C> {
    fn remap(&self) -> bool;
}

impl SclPin<I2C1> for PB6<AltOutput<OpenDrain>> {
    fn remap(&self) -> bool {
        false
    }
}
impl SdaPin<I2C1> for PB7<AltOutput<OpenDrain>> {
    fn remap(&self) -> bool {
        false
    }
}

// Serial abstraction
pub struct I2c<I2C, PINS> {
    i2c: I2C,
    pins: PINS,
}

impl<SC, SD> I2c<I2C1, (SC, SD)> {
    // init I2C1
    // TODO: I2c::new() for I2C1 and I2C2
    pub fn i2c1(i2c: I2C1, pins: (SC, SD), mode: I2cMode, clocks: &Clocks) -> Self
        where SC: SclPin<I2C1>, SD: SdaPin<I2C1>
    {
        unsafe {
            (*RCC::ptr()).apb1pcenr.modify(|_, w| w.i2c1en().set_bit());
            let base_freq = clocks.pclk1().0 / 1000_000; // by MHz
            assert!((base_freq >= 2) & (base_freq <= 36));
            (*I2C1::ptr()).ctlr2.modify(|_, w| w.freq().bits(base_freq as u8));

            match mode {
                // CCR values are referred to STM32F4xx (M0090 Rev 19) datasheet
                I2cMode::Standard => {
                    // Thigh = CCR * TPCLK1
                    // Tlow = CCR * TPCLK1
                    // (1 / 100kHz) * (1 / 2) = CCR * (1 / PCLK1)
                    let ccr = 5 * base_freq;
                    (*I2C1::ptr()).ckcfgr.modify(|_, w|
                        w
                            .f_s()
                            .clear_bit()
                            .ccr()
                            .bits(ccr as u16)
                    );
                }
                I2cMode::Fast => {
                    let ccr = if base_freq < 10 {
                        (*I2C1::ptr()).ckcfgr.modify(|_, w| w.duty().clear_bit());

                        // If DUTY = 0:
                        // Thigh = CCR * TPCLK1
                        // Tlow = 2 * CCR * TPCLK1
                        // (1 / 400kHz) * (1 / 3) = CCR * (1 / PCLK1)
                        (base_freq * 10) / 12
                    } else {
                        // If DUTY = 1:
                        (*I2C1::ptr()).ckcfgr.modify(|_, w| w.duty().set_bit());

                        // Thigh = 9 * CCR * TPCLK1
                        // Tlow = 16 * CCR * TPCLK1
                        // (1 / 400kHz) * (9 /25) = 9 * CCR * (1 / PCLK1)
                        // base_freq must be >10HHz
                        base_freq / 10
                    };
                    (*I2C1::ptr()).ckcfgr.modify(|_, w|
                        w
                            .f_s()
                            .set_bit()
                            .ccr()
                            .bits(ccr as u16)
                    );
                }
            }

            // Enable I2C
            (*I2C1::ptr()).ctlr1.modify(|_, w| w.pe().set_bit());
        }
        Self {
            i2c: i2c,
            pins: pins,
        }
    }

    pub fn status(&self) -> u16 {
        unsafe { (*I2C1::ptr()).star1.read().bits() as u16 }
    }
}

macro_rules! busy_wait {
    ($flag:ident) => {
        loop {
            let stat = (*I2C1::ptr()).star1.read();
            if stat.$flag().bit_is_set() {
                break;
            } else if stat.bits() & 0x1f00 > 0 {
                // Make STOP condition before exit
                (*I2C1::ptr()).ctlr1.modify(|_, w| w.stop().set_bit());
                return Err(
                    if stat.pecerr().bit_is_set() {
                        I2cError::CrcError
                    } else if stat.ovr().bit_is_set() {
                        I2cError::OverrunError
                    } else if stat.af().bit_is_set() {
                        I2cError::Nak
                    } else if stat.arlo().bit_is_set() {
                        I2cError::ArbitrationError
                    } else if stat.berr().bit_is_set() {
                        I2cError::BusError
                    } else {
                        I2cError::UnknownError
                    }
                );
            }
        }
    };
}

impl<I2C, PINS> i2c::Write<u8> for I2c<I2C, PINS> {
    type Error = I2cError;

    fn write(&mut self, address: u8, bytes: &[u8]) -> Result<(), Self::Error> {
        unsafe {
            // Reset status register
            (*I2C1::ptr()).star1.write(|w| w.bits(0));

            // make START condition
            (*I2C1::ptr()).ctlr1.modify(|_, w| w.start().set_bit());
            // wait start transmitted
            busy_wait!(sb);

            // send ADDRESS with WRITE flag(0)
            (*I2C1::ptr()).datar.write(|w| w.bits((address as u16) << 1));
            // Wait transmitted ADDR and RW bit.
            busy_wait!(addr);
            (*I2C1::ptr()).star2.read();

            // send all data
            for byte in bytes {
                busy_wait!(tx_e);
                (*I2C1::ptr()).datar.write(|w| w.bits(*byte as u16));
            }

            // Wait WRITE complete
            busy_wait!(btf);
            // make STOP condition
            (*I2C1::ptr()).ctlr1.modify(|_, w| w.stop().set_bit());
        }

        Ok(())
    }
}

impl<I2C, PINS> i2c::WriteIter<u8> for I2c<I2C, PINS> {
    type Error = I2cError;

    fn write<B>(&mut self, address: u8, bytes: B) -> Result<(), Self::Error>
        where B: IntoIterator<Item = u8>
    {
        unsafe {
            // Reset status register
            (*I2C1::ptr()).star1.write(|w| w.bits(0));

            // make START condition
            (*I2C1::ptr()).ctlr1.modify(|_, w| w.start().set_bit());
            // wait start transmitted
            busy_wait!(sb);

            // send ADDRESS with WRITE flag(0)
            (*I2C1::ptr()).datar.write(|w| w.bits((address as u16) << 1));
            // Wait transmitted ADDR and RW bit.
            busy_wait!(addr);
            (*I2C1::ptr()).star2.read();

            // send all data
            for byte in bytes {
                busy_wait!(tx_e);
                (*I2C1::ptr()).datar.write(|w| w.bits(byte as u16));
            }

            // Wait WRITE complete
            busy_wait!(btf);
            // make STOP condition
            (*I2C1::ptr()).ctlr1.modify(|_, w| w.stop().set_bit());
        }

        Ok(())
    }
}

impl<I2C, PINS> i2c::Read<u8> for I2c<I2C, PINS> {
    type Error = I2cError;

    fn read(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), Self::Error> {
        unsafe {
            // Reset status register
            (*I2C1::ptr()).star1.write(|w| w.bits(0));

            // make START condition
            (*I2C1::ptr()).ctlr1.modify(|_, w| w.start().set_bit());
            // wait start transmitted
            busy_wait!(sb);

            // send ADDRESS with WRITE flag
            (*I2C1::ptr()).datar.write(|w| w.bits(((address as u16) << 1) | 0b1));
            // Wait transmitted ADDR and RW bit.
            busy_wait!(addr);
            (*I2C1::ptr()).star2.read();

            // read all data
            (*I2C1::ptr()).ctlr1.modify(|_, w| w.ack().set_bit());
            let buffer_last = buffer.len() - 1;
            for (i, byte) in buffer.iter_mut().enumerate() {
                if i == buffer_last {
                    // Return NACK
                    (*I2C1::ptr()).ctlr1.modify(|_, w| w.ack().clear_bit());
                }
                // Wait ready to read
                busy_wait!(rx_ne);
                *byte = (*I2C1::ptr()).datar.read().bits() as u8;
            }

            // make STOP condition
            (*I2C1::ptr()).ctlr1.modify(|_, w| w.stop().set_bit());
        }

        Ok(())
    }
}

impl<I2C, PINS> i2c::WriteRead<u8> for I2c<I2C, PINS> {
    type Error = I2cError;

    fn write_read(
        &mut self,
        address: u8,
        bytes: &[u8],
        buffer: &mut [u8]
    ) -> Result<(), Self::Error> {
        unsafe {
            // Reset status register
            (*I2C1::ptr()).star1.write(|w| w.bits(0));

            // make START condition
            (*I2C1::ptr()).ctlr1.modify(|_, w| w.start().set_bit());
            // wait start transmitted
            busy_wait!(sb);

            // send ADDRESS with WRITE flag(0)
            (*I2C1::ptr()).datar.write(|w| w.bits((address as u16) << 1));
            // Wait transmitted ADDR and RW bit.
            busy_wait!(addr);
            (*I2C1::ptr()).star2.read();

            // send all data
            for byte in bytes {
                busy_wait!(tx_e);
                (*I2C1::ptr()).datar.write(|w| w.bits(*byte as u16));
            }

            // Wait WRITE complete
            busy_wait!(tx_e);

            // make START again
            (*I2C1::ptr()).ctlr1.modify(|_, w| w.start().set_bit());
            // wait start transmitted
            busy_wait!(sb);

            // send ADDRESS with WRITE flag
            (*I2C1::ptr()).datar.write(|w| w.bits(((address as u16) << 1) | 0b1));
            // Wait transmitted ADDR and RW bit.
            busy_wait!(addr);
            (*I2C1::ptr()).star2.read();

            // read all data
            (*I2C1::ptr()).ctlr1.modify(|_, w| w.ack().set_bit());
            let buffer_last = buffer.len() - 1;
            for (i, byte) in buffer.iter_mut().enumerate() {
                if i == buffer_last {
                    // Return NACK
                    (*I2C1::ptr()).ctlr1.modify(|_, w| w.ack().clear_bit());
                }
                // Wait ready to read
                busy_wait!(rx_ne);
                *byte = (*I2C1::ptr()).datar.read().bits() as u8;
            }

            // make STOP condition
            (*I2C1::ptr()).ctlr1.modify(|_, w| w.stop().set_bit());
        }

        Ok(())
    }
}

impl<I2C, PINS> i2c::WriteIterRead<u8> for I2c<I2C, PINS> {
    type Error = I2cError;

    fn write_iter_read<B>(
        &mut self,
        address: u8,
        bytes: B,
        buffer: &mut [u8]
    ) -> Result<(), Self::Error>
        where B: IntoIterator<Item = u8>
    {
        unsafe {
            // Reset status register
            (*I2C1::ptr()).star1.write(|w| w.bits(0));

            // make START condition
            (*I2C1::ptr()).ctlr1.modify(|_, w| w.start().set_bit());
            // wait start transmitted
            busy_wait!(sb);

            // send ADDRESS with WRITE flag(0)
            (*I2C1::ptr()).datar.write(|w| w.bits((address as u16) << 1));
            // Wait transmitted ADDR and RW bit.
            busy_wait!(addr);
            (*I2C1::ptr()).star2.read();

            // send all data
            for byte in bytes {
                busy_wait!(tx_e);
                (*I2C1::ptr()).datar.write(|w| w.bits(byte as u16));
            }

            // Wait WRITE complete
            busy_wait!(tx_e);

            // make START again
            (*I2C1::ptr()).ctlr1.modify(|_, w| w.start().set_bit());
            // wait start transmitted
            busy_wait!(sb);

            // send ADDRESS with WRITE flag
            (*I2C1::ptr()).datar.write(|w| w.bits(((address as u16) << 1) | 0b1));
            // Wait transmitted ADDR and RW bit.
            busy_wait!(addr);
            (*I2C1::ptr()).star2.read();

            // read all data
            (*I2C1::ptr()).ctlr1.modify(|_, w| w.ack().set_bit());
            let buffer_last = buffer.len() - 1;
            for (i, byte) in buffer.iter_mut().enumerate() {
                if i == buffer_last {
                    // Return NACK
                    (*I2C1::ptr()).ctlr1.modify(|_, w| w.ack().clear_bit());
                }
                // Wait ready to read
                busy_wait!(rx_ne);
                *byte = (*I2C1::ptr()).datar.read().bits() as u8;
            }

            // make STOP condition
            (*I2C1::ptr()).ctlr1.modify(|_, w| w.stop().set_bit());
        }

        Ok(())
    }
}
