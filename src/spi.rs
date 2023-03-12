use embedded_hal::{ blocking, spi };
use ch32v1::ch32v103::{ RCC, SPI1 };

use nb;
use crate::rcc::*;
use crate::time::*;
use crate::gpio::*;
use crate::gpio::gpioa::{ PA5, PA6, PA7 };

pub enum SpiMode {
    Mode0,
    Mode1,
    Mode2,
    Mode3,
}

// define spi error
#[derive(Debug)]
pub enum Error {
    // Mode error
    Mode,
    // CRC error
    Crc,
    // RX buffer overrun
    Overrun,
    // Unkown
    Unkown,
}

pub trait SckPin<T> {
    fn remap(&self) -> bool;
}

pub trait MisoPin<T> {
    fn remap(&self) -> bool;
}
pub trait MosiPin<T> {
    fn remap(&self) -> bool;
}

impl SckPin<SPI1> for PA5<AltOutput<PushPull>> {
    fn remap(&self) -> bool {
        false
    }
}

impl MisoPin<SPI1> for PA6<Input<Floating>> {
    fn remap(&self) -> bool {
        false
    }
}

impl MosiPin<SPI1> for PA7<AltOutput<PushPull>> {
    fn remap(&self) -> bool {
        false
    }
}

pub struct Spi<SPI, PINS> {
    spi: SPI,
    pins: PINS,
}

impl<SCK, MISO, MOSI> Spi<SPI1, (SCK, MISO, MOSI)> {
    // init USART
    pub fn spi1(
        spi: SPI1,
        pins: (SCK, MISO, MOSI),
        mode: spi::Mode,
        speed: Hertz,
        clocks: &Clocks
    )
        -> Self
        where SCK: SckPin<SPI1>, MISO: MisoPin<SPI1>, MOSI: MosiPin<SPI1>
    {
        unsafe {
            // provide clock to USART1
            (*RCC::ptr()).apb2pcenr.modify(|_, w| w.spi1en().set_bit());

            // Set SPI1 to Master mode
            let br_bits = match clocks.pclk2().0 / speed.0 {
                0 => unreachable!(),
                1..=2 => 0b000, // Div2
                3..=5 => 0b001, // Div4
                6..=11 => 0b010, //Div8
                12..=23 => 0b011, // Div16
                24..=47 => 0b100, // Div32
                48..=95 => 0b101, // Div64
                96..=191 => 0b110, // Div128
                _ => 0b111, // Div256
            };
            (*SPI1::ptr()).ctlr1.modify(|_, w| w.br().bits(br_bits));

            match mode.polarity {
                spi::Polarity::IdleLow => {
                    (*SPI1::ptr()).ctlr1.modify(|_, w| w.cpol().clear_bit());
                }
                spi::Polarity::IdleHigh => {
                    (*SPI1::ptr()).ctlr1.modify(|_, w| w.cpol().set_bit());
                }
            }

            match mode.phase {
                spi::Phase::CaptureOnFirstTransition => {
                    (*SPI1::ptr()).ctlr1.modify(|_, w| w.cpha().clear_bit());
                }
                spi::Phase::CaptureOnSecondTransition => {
                    (*SPI1::ptr()).ctlr1.modify(|_, w| w.cpha().set_bit());
                }
            }

            // set DEF and LSBFIRST

            // Setup NSS, SSM, SSI, SSOE

            // Control CS by hardware. One Master and One Slave
            // CS is Low when SPE is set and High when SPE is High.
            // (*SPI1::ptr()).ctlr1.modify(|_, w| w.ssm().clear_bit().ssi().clear_bit()); // ssi may not care on Master
            // (*SPI1::ptr()).ctlr2.modify(|_, w| w.ssoe().set_bit());
            // // Enable SPI as Master
            // (*SPI1::ptr()).ctlr1.modify(|_, w| w.mstr().set_bit().spe().clear_bit());

            // Control CS by software or GPIO
            (*SPI1::ptr()).ctlr1.modify(|_, w| w.ssm().set_bit().ssi().set_bit());
            // ssi must set 1. Why?
            // I found good explanation on stack overflow.
            // Setting nss_soft in Master (SPI)
            //  https://stackoverflow.com/questions/48849942/setting-nss-soft-in-master-spi

            // Enable SPI as Master
            (*SPI1::ptr()).ctlr1.modify(|_, w| w.mstr().set_bit().spe().set_bit());
        }

        Spi { spi, pins }
    }

    pub fn enable(&self) {
        unsafe {
            (*SPI1::ptr()).ctlr1.modify(|_, w| w.spe().set_bit());
        }
    }

    pub fn disable(&self) {
        unsafe {
            (*SPI1::ptr()).ctlr1.modify(|_, w| w.spe().clear_bit());
        }
    }
}

impl<SCK, MISO, MOSI> spi::FullDuplex<u8> for Spi<SPI1, (SCK, MISO, MOSI)> {
    type Error = Error;

    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        unsafe {
            let stat = (*SPI1::ptr()).statr.read();
            if stat.bsy().bit_is_clear() & stat.rxne().bit_is_set() {
                Ok((*SPI1::ptr()).datar.read().bits() as u8)
            } else {
                Err(nb::Error::WouldBlock)
            }
        }
    }

    fn send(&mut self, word: u8) -> nb::Result<(), Self::Error> {
        unsafe {
            if (*SPI1::ptr()).statr.read().txe().bit_is_set() {
                (*SPI1::ptr()).datar.write(|w| w.bits(word as u16));
                Ok(())
            } else {
                Err(nb::Error::WouldBlock)
            }
        }
    }
}

// This trait has default implementation for blocking::spi::Transfer<u8>.
impl<SCK, MISO, MOSI> blocking::spi::transfer::Default<u8>
    for Spi<SPI1, (SCK, MISO, MOSI)>
    where SCK: SckPin<SPI1>, MISO: MisoPin<SPI1>, MOSI: MosiPin<SPI1> {}

// This trait has default implementation for blocking::spi::Write<u8>.
impl<SCK, MISO, MOSI> blocking::spi::write::Default<u8>
    for Spi<SPI1, (SCK, MISO, MOSI)>
    where SCK: SckPin<SPI1>, MISO: MisoPin<SPI1>, MOSI: MosiPin<SPI1> {}

// This trait has default implementation for blocking::spi::WriteIter<u8>.
impl<SCK, MISO, MOSI> blocking::spi::write_iter::Default<u8>
    for Spi<SPI1, (SCK, MISO, MOSI)>
    where SCK: SckPin<SPI1>, MISO: MisoPin<SPI1>, MOSI: MosiPin<SPI1> {}