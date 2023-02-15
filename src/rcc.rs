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