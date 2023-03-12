use embedded_hal::adc::{ Channel, OneShot };
use ch32v1::ch32v103::{ RCC, ADC };
// use crate::time::*;
use crate::prelude::*;
use crate::rcc::*;
use crate::gpio::*;
use crate::delay::*;
use crate::gpio::gpioa::{ PA0, PA1, PA2, PA3, PA4, PA5, PA6, PA7 };
use crate::gpio::gpiob::{ PB0, PB1 };
// use crate::gpio::gpioc::{ PC0, PC1, PC2, PC3, PC4, PC5 };

macro_rules! adc_channel {
    ($PXi:ident, $i:expr) => {
        impl<ADC> Channel<ADC> for $PXi<Input<Analog>> {
            type ID = u8;

            fn channel() -> u8 {
                $i
            }
        }
    };
}

adc_channel!(PA0, 0);
adc_channel!(PA1, 1);
adc_channel!(PA2, 2);
adc_channel!(PA3, 3);
adc_channel!(PA4, 4);
adc_channel!(PA5, 5);
adc_channel!(PA6, 6);
adc_channel!(PA7, 7);
adc_channel!(PB0, 8);
adc_channel!(PB1, 9);
// adc_channel!(PC0, 10);
// adc_channel!(PC1, 11);
// adc_channel!(PC2, 12);
// adc_channel!(PC3, 13);
// adc_channel!(PC4, 14);
// adc_channel!(PC5, 15);

pub struct Adc<ADC> {
    adc: ADC,
    delay: Delay,
    tconv: u32,
}

impl<ADCX> Adc<ADCX> {
    pub fn adc(adc: ADCX, clocks: &Clocks) -> Self {
        let (adcpre_bits, adc_clock) = match clocks.pclk2().0 / 14_000_000 {
            0..=1 => (0b00, clocks.pclk2().0 / 2),
            2..=3 => (0b01, clocks.pclk2().0 / 4),
            4..=5 => (0b10, clocks.pclk2().0 / 6),
            _ => (0x11, clocks.pclk2().0 / 8),
        };

        unsafe {
            // enable ADC
            (*RCC::ptr()).apb2pcenr.modify(|_, w| w.adcen().set_bit());
            // set ADC clock's prescale
            (*RCC::ptr()).cfgr0.modify(|_, w| w.adcpre().bits(adcpre_bits));
            // set every SMP 0b111 or 239.5 cycles
            // slow is not worse than too fast.
            // TODO: make interface to change SMPx
            (*ADC::ptr()).samptr1.write(|w| w.bits(0xffff_ffff));
            (*ADC::ptr()).samptr2.write(|w| w.bits(0xffff_ffff));
        }

        let tconv = (240.0 + 12.5) / ((adc_clock / 1_000_000) as f32);
        Self {
            adc: adc,
            delay: Delay::new(clocks),
            tconv: (tconv as u32) + 1, // in us
        }
    }

    fn power_up(&mut self) {
        unsafe {
            (*ADC::ptr()).ctlr2.modify(|_, w| w.adon().set_bit());
            //wait Tstab
            self.delay.delay_us(1);
            (*ADC::ptr()).ctlr2.modify(|_, w| w.adon().set_bit());
        }
    }

    fn power_down(&self) {
        unsafe {
            (*ADC::ptr()).ctlr2.modify(|_, w| w.adon().clear_bit());
        }
    }

    fn do_conversion(&self, channel: u8) -> u16 {
        unsafe {
            (*ADC::ptr()).rsqr3.modify(|_, w| w.sq1().bits(channel));
            (*ADC::ptr()).rsqr1.modify(|_, w| w.l().bits(0x1));

            // start conversion
            (*ADC::ptr()).ctlr2.modify(|_, w| w.adon().set_bit());
            // wait conversion
            while (*ADC::ptr()).statr.read().eoc().bit_is_clear() {}

            (*ADC::ptr()).rdatar.read().bits() as u16
        }
    }

    pub fn read_temp(&mut self) -> u16 {
        unsafe {
            (*ADC::ptr()).ctlr2.modify(|_, w| w.tsvrefe().set_bit());
        }

        self.power_up();
        // internal temperature sensor in on channel 16
        let result = self.do_conversion(16);
        unsafe {
            (*ADC::ptr()).ctlr2.modify(|_, w| w.tsvrefe().clear_bit());
        }
        self.power_down();
        result.into()
    }

    pub fn calibration(&mut self) -> u16 {
        self.power_up();
        self.delay.delay_us(10);

        let mut cal = 0_u16;
        unsafe {
            // start calibration
            // NOTE: rstcal does not have set_bit() and clear_bit() but has reset()
            // REF: https://raw.githubusercontent.com/ch32-rs/ch32-rs-nightlies/main/ch32v1/src/ch32v103/mod.rs
            // (*ADC::ptr()).ctlr2.modify(|_, w| w.rstcal().set_bit());
            (*ADC::ptr()).ctlr2.modify(|_, w| w.rstcal().reset());
            // while (*ADC::ptr()).ctlr2.read().rstcal().bit_is_set() {}
            while (*ADC::ptr()).ctlr2.read().rstcal().is_resetting() {}

            // (*ADC::ptr()).ctlr2.modify(|_, w| w.cal().set_bit());
            // while (*ADC::ptr()).ctlr2.read().cal().bit_is_set() {}
            (*ADC::ptr()).ctlr2.modify(|_, w| w.cal().calibrate());
            while (*ADC::ptr()).ctlr2.read().cal().is_calibrating() {}

            cal = (*ADC::ptr()).rdatar.read().bits() as u16;
        }

        self.power_down();
        cal
    }
}

impl<WORD, PIN> OneShot<Adc<ADC>, WORD, PIN>
    for Adc<ADC>
    where WORD: From<u16>, PIN: Channel<Adc<ADC>, ID = u8>
{
    type Error = ();

    fn read(&mut self, _pin: &mut PIN) -> nb::Result<WORD, Self::Error> {
        self.power_up();

        let chan = PIN::channel();
        let result = self.do_conversion(chan);
        self.power_down();
        Ok(result.into())
    }
}