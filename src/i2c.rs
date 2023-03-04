use ch32v1::ch32v103::{ RCC, I2C1 };
// use crate::time::*;
use crate::rcc::*;
use crate::gpio::*;
use crate::gpio::gpiob::{ PB6, PB7 };

pub unsafe trait SclPin<I2C> {
    fn remap(&self) -> bool;
}
pub unsafe trait SdaPin<I2C> {
    fn remap(&self) -> bool;
}

unsafe impl SclPin<I2C1> for PB6<AltOutput<OpenDrain>> {
    fn remap(&self) -> bool {
        false
    }
}
unsafe impl SdaPin<I2C1> for PB7<AltOutput<OpenDrain>> {
    fn remap(&self) -> bool {
        false
    }
}

// Serial abstraction
pub struct I2c<PINS> {
    i2c: I2C1,
    pins: PINS,
}

impl<SC, SD> I2c<(SC, SD)> {
    // init I2C1
    pub fn i2c1(i2c: I2C1, pins: (SC, SD), clocks: &Clocks) -> Self
        where SC: SclPin<I2C1>, SD: SdaPin<I2C1>
    {
        unsafe {
            (*RCC::ptr()).apb1pcenr.modify(|_, w| w.i2c1en().set_bit());
            // (*I2C1::ptr()).ctlr2.modify(|_, w| w.freq().bits(0b1100));

            // For testing
            (*I2C1::ptr()).ckcfgr.modify(|_, w| w.f_s().set_bit().ccr().bits(40)); // fast mode
            // Enable I2C
            (*I2C1::ptr()).ctlr1.modify(|_, w| w.pe().set_bit());
        }
        Self {
            i2c: i2c,
            pins: pins,
        }
    }

    pub fn start(&self) {
        unsafe {
            // START condition
            (*I2C1::ptr()).ctlr1.modify(|_, w| w.start().set_bit());
            // wait start transmitted
            while (*I2C1::ptr()).star1.read().sb().bit_is_clear() {}
        }
    }

    pub fn stop(&self) {
        unsafe {
            // All transmission completed
            while (*I2C1::ptr()).star1.read().btf().bit_is_clear() {}
            // STOP condition
            (*I2C1::ptr()).ctlr1.modify(|_, w| w.stop().set_bit());
        }
    }

    pub fn addr(&self, addr: u8, rw: bool) {
        unsafe {
            (*I2C1::ptr()).datar.write(|w|
                w.bits(0x400_u16 | ((addr as u16) << 1) | (if rw { 1 } else { 0 }))
            );
            // Wait transmitted ADDR and RW bit.
            // while (*I2C1::ptr()).star1.read().addr().bit_is_clear() {}
            // (*I2C1::ptr()).star2.read();
        }
    }

    pub fn send(&self, data: u8) {
        unsafe {
            while (*I2C1::ptr()).star1.read().tx_e().bit_is_clear() {}
            (*I2C1::ptr()).datar.write(|w| w.bits(data as u16));
        }
    }

    pub fn read(&self, ack: bool) -> u8 {
        unsafe {
            if ack {
                (*I2C1::ptr()).ctlr1.modify(|_, w| w.ack().set_bit());
            } else {
                (*I2C1::ptr()).ctlr1.modify(|_, w| w.ack().clear_bit());
            }
            // Wait ready to read
            while (*I2C1::ptr()).star1.read().rx_ne().bit_is_clear() {}
            (*I2C1::ptr()).datar.read().bits() as u8
        }
    }

    pub fn status(&self) -> u16 {
        unsafe { (*I2C1::ptr()).star1.read().bits() as u16 }
    }
}