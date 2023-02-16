// use core::convert::Infallible;
use ch32v1::ch32v103::RCC;

pub struct AHB;
pub struct APB1;
pub struct APB2;

pub struct Rcc {
    pub ahb: AHB,
    pub apb1: APB1,
    pub apb2: APB2,
}

pub trait RccExt {
    type Rcc;
    fn constrain(self) -> Self::Rcc;
}

impl RccExt for RCC {
    type Rcc = Rcc;

    fn constrain(self) -> Rcc {
        // unsafe {
        //     (*RCC::ptr()).cfgr0.modify(|r, w| w.bits((r.bits() & !(0b1111 << 4)) | (0b1010 << 4))); // HPRE DIV8  8MHz / 8 = 1us /cycle
        //     (*RCC::ptr()).apb2pcenr.modify(|_, w| w.tim1en().set_bit());
        // }

        Rcc {
            ahb: AHB {},
            apb1: APB1 {},
            apb2: APB2 {},
        }
    }
}

impl APB2 {
    // type Error = Infallible;

    pub fn enable_gpioa(&self) {
        unsafe {
            (*RCC::ptr()).apb2pcenr.modify(|_, w| w.iopaen().set_bit());
        }
    }

    pub fn enable_gpiob(&self) {
        unsafe {
            (*RCC::ptr()).apb2pcenr.modify(|_, w| w.iopben().set_bit());
        }
    }
}