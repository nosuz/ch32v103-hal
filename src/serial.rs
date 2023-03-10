use core::marker::PhantomData;
use embedded_hal::{ blocking, serial };
use nb;
use core::convert::Infallible;
use core::fmt;

use ch32v1::ch32v103::{ AFIO, RCC, USART1 };
use crate::time::*;
use crate::rcc::*;
use crate::gpio::*;
use crate::gpio::gpioa::{ PA9, PA10 };
use crate::gpio::gpiob::{ PB6, PB7 };

// define serial error
#[derive(Debug)]
pub enum UsartError {
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

// Serial transmitter
pub struct Tx<USART> {
    _usart: PhantomData<USART>,
}

pub unsafe trait TxPin<USART> {
    fn remap(&self) -> bool;
}
pub unsafe trait RxPin<USART> {
    fn remap(&self) -> bool;
}

// why unsafe is required?
unsafe impl TxPin<USART1> for PA9<AltOutput<PushPull>> {
    fn remap(&self) -> bool {
        false
    }
}
unsafe impl RxPin<USART1> for PA10<Input<Floating>> {
    fn remap(&self) -> bool {
        false
    }
}
unsafe impl RxPin<USART1> for PA10<Input<PullUp>> {
    fn remap(&self) -> bool {
        false
    }
}

// Remap
unsafe impl TxPin<USART1> for PB6<AltOutput<PushPull>> {
    fn remap(&self) -> bool {
        true
    }
}
unsafe impl RxPin<USART1> for PB7<Input<Floating>> {
    fn remap(&self) -> bool {
        true
    }
}
unsafe impl RxPin<USART1> for PB7<Input<PullUp>> {
    fn remap(&self) -> bool {
        true
    }
}

// Serial abstraction
pub struct Serial<USART, PINS> {
    usart: USART,
    pins: PINS,
}

impl<TX, RX> Serial<USART1, (TX, RX)> {
    // init USART
    pub fn usart1(usart: USART1, pins: (TX, RX), baud_rate: Bps, clocks: &Clocks) -> Self
        where TX: TxPin<USART1>, RX: RxPin<USART1>
    {
        // enable USART
        unsafe {
            // ToDo: Want to check while compiling.
            // remap USART1
            if pins.0.remap() & pins.1.remap() {
                // clock is required before remap.
                (*RCC::ptr()).apb2pcenr.modify(|_, w| w.afioen().set_bit());
                (*AFIO::ptr()).pcfr.modify(|_, w| w.usart1rm().set_bit());
            } else if pins.0.remap() | pins.1.remap() {
                unreachable!();
            }

            // provide clock to USART1
            (*RCC::ptr()).apb2pcenr.modify(|_, w| w.usart1en().set_bit());

            // USARTDIV = Fclk / bps / 16
            // USARTDIV * 16 = Fclk / bps
            // BRR = USARTDIV_M << 4 + USARTDIV_F = USARTDIV
            // 8 MHz / 9600 / 16 = 52.08
            // 8 MHz / 115200 / 16 = 4.34
            let brr_div: u32 = clocks.pclk2().0 / baud_rate.0;
            (*USART1::ptr()).brr.write(|w| w.bits(brr_div));

            // disable harware flow control
            (*USART1::ptr()).ctlr3.modify(|_, w| w.ctse().clear_bit().rtse().clear_bit());
            // enable USART, enable transmitter and receiver
            (*USART1::ptr()).ctlr1.modify(|_, w| w.ue().set_bit().te().set_bit().re().set_bit());
        }

        Serial { usart: usart, pins: pins }
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
    type Error = Infallible;

    fn write(&mut self, byte: u8) -> nb::Result<(), Self::Error> {
        unsafe {
            // check TX register is empty
            if (*USART1::ptr()).statr.read().txe().bit_is_set() {
                (*USART1::ptr()).datar.write(|w| w.bits(byte as u32));
                Ok(())
            } else {
                Err(nb::Error::WouldBlock)
            }
        }
    }

    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        unsafe {
            if (*USART1::ptr()).statr.read().tc().bit_is_set() {
                Ok(())
            } else {
                Err(nb::Error::WouldBlock)
            }
        }
    }
}

impl serial::Read<u8> for Rx<USART1> {
    type Error = UsartError;

    fn read(&mut self) -> nb::Result<u8, UsartError> {
        unsafe {
            // read STATR
            let statr = (*USART1::ptr()).statr.read();
            if statr.rxne().bit_is_set() {
                Ok((*USART1::ptr()).datar.read().bits() as u8)
            } else {
                Err(
                    if statr.ore().bit_is_set() {
                        nb::Error::Other(UsartError::Overrun)
                    } else if statr.ne().bit_is_set() {
                        nb::Error::Other(UsartError::Noise)
                    } else if statr.fe().bit_is_set() {
                        nb::Error::Other(UsartError::Framing)
                    } else if statr.pe().bit_is_set() {
                        nb::Error::Other(UsartError::Parity)
                    } else {
                        nb::Error::WouldBlock
                    }
                )
            }
        }
    }
}

pub struct SerialWriter<T> where T: serial::Write<u8> {
    serial: T,
}

impl<T> SerialWriter<T> where T: serial::Write<u8> {
    pub fn new(serial: T) -> Self {
        SerialWriter { serial }
    }
}

impl<T> fmt::Write for SerialWriter<T> where T: serial::Write<u8> {
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

// Only implimenting this marker trait, methods in blocking::serial::Write are available.
impl blocking::serial::write::Default<u8> for Tx<USART1> {}

// impl blocking::serial::Write<u8> for Tx<USART1> {
//     type Error = Infallible;

//     fn bwrite_all(&mut self, buffer: &[u8]) -> Result<(), Self::Error> {
//         Ok(())
//     }

//     fn bflush(&mut self) -> Result<(), Self::Error> {
//         match nb::block!(self.flush()) {
//             Ok(_) => {}
//             Err(_) => {}
//         }

//         Ok(())
//     }
// }