use ch32v1::ch32v103::{ RCC, ADC };
// use crate::time::*;
// use crate::rcc::*;
// use crate::gpio::*;
// use crate::delay::*;

// pub struct Adc<CHANNEL> {
//     channel: CHANNEL,
// }
pub struct Adc;

impl Adc {
    pub fn adc(adc: ADC) -> Self {
        unsafe {
            (*RCC::ptr()).apb2pcenr.modify(|_, w| w.adcen().set_bit());

            (*ADC::ptr()).ctlr2.modify(|_, w| w.tsvrefe().set_bit().adon().set_bit());
        }

        Self
    }

    pub fn power_on(&self) {
        unsafe {
            (*ADC::ptr()).ctlr2.modify(|_, w| w.adon().set_bit());
        }
    }

    pub fn power_off(&self) {
        unsafe {
            (*ADC::ptr()).ctlr2.modify(|_, w| w.adon().clear_bit());
        }
    }

    pub fn start_conv(&self) {
        unsafe {
            (*ADC::ptr()).rsqr1.modify(|_, w| w.l().bits(0x1));
            (*ADC::ptr()).rsqr3.modify(|_, w| w.sq1().bits(16));

            (*ADC::ptr()).ctlr2.modify(|_, w| w.adon().set_bit());
        }
    }

    pub fn read_temp(&self) -> u16 {
        unsafe { (*ADC::ptr()).rdatar.read().bits() as u16 }
    }
}