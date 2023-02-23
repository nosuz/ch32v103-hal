use core::marker::PhantomData;
use embedded_hal::serial;
use nb;
use core::convert::Infallible;
use core::fmt;

use ch32v1::ch32v103::{ RCC, USART1 };
use crate::time::*;
// use crate::gpio::{ AltOutput, Input, PushPull, PullUp, Floating };
use crate::gpio::*;
use crate::gpio::gpioa::{ PA9, PA10 };

// define serial error
#[derive(Debug)]
pub enum Error {
    // Framing error
    Framing,
    // Noise error
    Noise,
    // RX buffer overrun
    Overrun,
    // Parity check error
    Parity,
}

// define Tx and Rx Pin trait
// Serial receiver
pub struct Rx<USART> {
    _usart: PhantomData<USART>,
}
pub unsafe trait TxPin<USART> {}

// Serial transmitter
pub struct Tx<USART> {
    _usart: PhantomData<USART>,
}
pub unsafe trait RxPin<USART> {}

// why unsafe is required?
unsafe impl TxPin<USART1> for PA9<AltOutput<PushPull>> {}
unsafe impl RxPin<USART1> for PA10<Input<Floating>> {}
unsafe impl RxPin<USART1> for PA10<Input<PullUp>> {}

// Serial abstraction
pub struct Serial<PINS> {
    pins: PINS,
}

impl<TX, RX> Serial<(TX, RX)> {
    // init USART
    pub fn usart1(pins: (TX, RX), baud_rate: Bps) -> Self where TX: TxPin<USART1>, RX: RxPin<USART1> {
        // enable USART
        unsafe {
            // provide clock to USART1
            (*RCC::ptr()).apb2pcenr.modify(|_, w| w.usart1en().set_bit());

            // USARTDIV = Fclk / bps / 16
            // USARTDIV * 16 = Fclk / bps
            // BRR = USARTDIV_M << 4 + USARTDIV_F = USARTDIV
            // 8 MHz / 9600 / 16 = 52.08
            // 8 MHz / 115200 / 16 = 4.34
            let brr_div: u32 = (8_u32).mhz().0 / baud_rate.0;
            (*USART1::ptr()).brr.write(|w| w.bits(brr_div));

            // disable harware flow control
            (*USART1::ptr()).ctlr3.modify(|_, w| w.ctse().clear_bit().rtse().clear_bit());
            // enable USART, enable transmitter and receiver
            (*USART1::ptr()).ctlr1.modify(|_, w| w.ue().set_bit().te().set_bit().re().set_bit());
        }

        Serial { pins }
    }

    /// Splits the `Serial` abstraction into a transmitter and a receiver half
    pub fn split(self) -> (Tx<USART1>, Rx<USART1>) {
        (
            Tx {
                _usart: PhantomData,
            },
            Rx {
                _usart: PhantomData,
            },
        )
    }
}

impl serial::Write<u8> for Tx<USART1> {
    // type Error = Void;
    type Error = Infallible;

    fn write(&mut self, byte: u8) -> nb::Result<(), Self::Error> {
        unsafe {
            // check TX register is empty
            if (*USART1::ptr()).statr.read().bits() & (0b1 << 7) > 0 {
                (*USART1::ptr()).datar.write(|w| w.bits(byte as u32));
                Ok(())
            } else {
                Err(nb::Error::WouldBlock)
            }
        }
    }

    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        unsafe {
            if (*USART1::ptr()).statr.read().bits() & (0b1 << 6) > 0 {
                Ok(())
            } else {
                Err(nb::Error::WouldBlock)
            }
        }
    }
}

impl serial::Read<u8> for Rx<USART1> {
    type Error = Error;

    fn read(&mut self) -> nb::Result<u8, Error> {
        unsafe {
            // read STATR
            let statr = (*USART1::ptr()).statr.read().bits();
            if statr & (0b1 << 5) > 0 {
                Ok((*USART1::ptr()).datar.read().bits() as u8)
            } else {
                Err(
                    if statr & (0b1 << 3) > 0 {
                        nb::Error::Other(Error::Overrun)
                    } else if statr & (0b1 << 2) > 0 {
                        nb::Error::Other(Error::Noise)
                    } else if statr & (0b1 << 1) > 0 {
                        nb::Error::Other(Error::Framing)
                    } else if statr & (0b1 << 0) > 0 {
                        nb::Error::Other(Error::Parity)
                    } else {
                        nb::Error::WouldBlock
                    }
                )
            }
        }
    }
}

pub struct SerialWriter<T> where T: embedded_hal::serial::Write<u8> {
    serial: T,
}

impl<T> SerialWriter<T> where T: embedded_hal::serial::Write<u8> {
    pub fn new(serial: T) -> Self {
        SerialWriter { serial }
    }
}

impl<T> fmt::Write for SerialWriter<T> where T: embedded_hal::serial::Write<u8> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for b in s.bytes() {
            match nb::block!(self.serial.write(b)) {
                Ok(_) => {}
                Err(_) => {}
            }
        }
        Ok(())
    }
}