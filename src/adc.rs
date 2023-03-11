use embedded_hal::adc::{ Channel, OneShot };
use ch32v1::ch32v103::{ RCC, ADC };
// use crate::time::*;
use crate::prelude::*;
use crate::rcc::*;
use crate::gpio::*;
use crate::delay::*;
use crate::gpio::gpioa::{ PA0 };

// pub trait Channel<ADC> {
//     type ID;
//     fn channel() -> Self::ID;
// }

impl<ADC> Channel<ADC> for PA0<Input<Analog>> {
    type ID = u8;

    fn channel() -> u8 {
        0_u8
    }
}

pub struct Adc<ADC> {
    adc: ADC,
    delay: Delay,
}

impl<ADCX> Adc<ADCX> {
    pub fn adc(adc: ADCX, clocks: &Clocks) -> Self {
        unsafe {
            (*RCC::ptr()).apb2pcenr.modify(|_, w| w.adcen().set_bit());
        }

        Self {
            adc: adc,
            delay: Delay::new(clocks),
        }
    }

    pub fn power_up(&mut self) {
        unsafe {
            (*ADC::ptr()).ctlr2.modify(|_, w| w.adon().set_bit());
            self.delay.delay_us(1);
            (*ADC::ptr()).ctlr2.modify(|_, w| w.adon().set_bit());
        }
    }

    pub fn power_down(&self) {
        unsafe {
            (*ADC::ptr()).ctlr2.modify(|_, w| w.adon().clear_bit());
        }
    }

    pub fn do_conversion(&self, channel: u8) -> u16 {
        unsafe {
            (*ADC::ptr()).rsqr3.modify(|_, w| w.sq1().bits(channel));
            (*ADC::ptr()).rsqr1.modify(|_, w| w.l().bits(0x1));

            (*ADC::ptr()).ctlr2.modify(|_, w| w.adon().set_bit());

            (*ADC::ptr()).rdatar.read().bits() as u16
        }
    }

    pub fn read_temp(&self) -> u16 {
        unsafe { (*ADC::ptr()).rdatar.read().bits() as u16 }
    }
}

impl<WORD, PIN> OneShot<Adc<ADC>, WORD, PIN>
    for Adc<ADC>
    where WORD: From<u16>, PIN: Channel<Adc<ADC>, ID = u8>
{
    type Error = ();

    fn read(&mut self, _pin: &mut PIN) -> nb::Result<WORD, Self::Error> {
        // delay.delay_us(1);
        self.power_up();

        let chan = PIN::channel();
        let result = self.do_conversion(chan);
        self.power_down();
        Ok(result.into())
    }
}