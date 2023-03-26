#[cfg(feature = "interrupt")]
use ch32v1::ch32v103::{ PWR, PFIC, RTC, EXTI };
#[cfg(feature = "interrupt")]
use ch32v1::ch32v103::interrupt::Interrupt;
#[cfg(feature = "interrupt")]
use core::arch::asm;
#[cfg(feature = "interrupt")]
use crate::interrupt;

use embedded_hal::prelude::*;
use embedded_hal::blocking::delay;

// SysTick (STK) register strucure is not same as CH32V20x and CH32V30x
use crate::rcc::*;

#[cfg(feature = "interrupt")]
static mut TIME_UP: bool = false;

pub struct Delay {
    hclk: u32,
    ctlr: usize,
    cntl: usize,
    cnth: usize,
    // cmplr: usize,
    // cmphr: usize,
}

impl Delay {
    pub fn new(clocks: &Clocks) -> Self {
        const STK_BASE: usize = 0xe000_f000;

        Delay {
            hclk: clocks.hclk().0,
            ctlr: STK_BASE,
            cntl: STK_BASE + 0x4,
            cnth: STK_BASE + 0x8,
            // cmplr: STK_BASE + 0xc,
            // cmphr: STK_BASE + 0x10,
        }
    }

    fn has_wrapped(&self) -> bool {
        unsafe {
            let valh = (self.cnth as *mut usize).read_volatile();
            valh > 0
        }
    }

    fn set_counter(&mut self, value: u32) {
        unsafe {
            // CH32V103 (RISC V) is Little Endian.
            // 0: LSB, 3: MSB
            for i in 0..4_usize {
                // set cntl
                ((self.cntl + i) as *mut u8).write_volatile(((value >> (8 * i)) & 0xff) as u8);
                // reset cnth
                ((self.cnth + i) as *mut u8).write_volatile(0);
            }
        }
    }

    fn start_count(&mut self) {
        unsafe {
            (self.ctlr as *mut u32).write_volatile(1);
        }
    }

    fn stop_count(&mut self) {
        unsafe {
            (self.ctlr as *mut u32).write_volatile(0);
        }
    }

    #[cfg(feature = "interrupt")]
    pub fn sleep_ms(&mut self, duration: u32) {
        // sleep specified ms. Max. 2^32ms = about 49 days.

        // don't sleep less than 10 ms.
        if duration < 10 {
            self.delay_ms(duration);
            unsafe { core::ptr::write_volatile(&mut TIME_UP, true) }
        } else {
            unsafe {
                // what will happen if reading RTC_L just before overflowed to RTC_H
                // wait next update
                (*RTC::ptr()).ctlrl.modify(|_, w| w.secf().clear_bit());
                while (*RTC::ptr()).ctlrl.read().secf().bit_is_clear() {}

                // get current RTC count
                let rtc_l = (*RTC::ptr()).cntl.read().bits() as u32;
                let rtc_h = (*RTC::ptr()).cnth.read().bits() as u32;
                #[allow(arithmetic_overflow)]
                let rtc = (rtc_h << 16) | rtc_l;

                let wake_up = rtc + duration - 3; // 3 is magic number to adjust sleep duration.
                let wake_up_h = (wake_up >> 16) as u16;
                let wake_up_l = (wake_up & 0xffff) as u16;

                (*PWR::ptr()).ctlr.modify(|_, w| w.dbp().set_bit());

                while !(*RTC::ptr()).ctlrl.read().rtoff().bit_is_set() {}
                (*RTC::ptr()).ctlrl.modify(|_, w| w.cnf().set_bit());
                // whitout seting PSCRH, PSC[19:16] will be set to 1.
                (*RTC::ptr()).alrmh.write(|w| w.bits(wake_up_h));
                (*RTC::ptr()).alrml.write(|w| w.bits(wake_up_l));
                (*RTC::ptr()).ctlrl.modify(|_, w| w.cnf().clear_bit().alrf().clear_bit());
                // Wait write completed
                while !(*RTC::ptr()).ctlrl.read().rtoff().bit_is_set() {}

                (*PWR::ptr()).ctlr.modify(|_, w| w.dbp().clear_bit());

                // enable interrupt and get into sleep mode
                // EXTI17 routed to RTCALARM
                (*EXTI::ptr()).intenr.modify(|_, w| w.mr17().set_bit());
                // (*EXTI::ptr()).evenr.modify(|_, w| w.mr17().set_bit());
                (*EXTI::ptr()).rtenr.modify(|_, w| w.tr17().set_bit());

                (*PFIC::ptr()).ienr2.modify(|_, w|
                    w.bits(0b1 << ((Interrupt::RTCALARM as u32) - 32))
                );
                // enable ALARM interrupt
                (*RTC::ptr()).ctlrh.modify(|_, w| w.alrie().set_bit());
                riscv::interrupt::enable();

                (*PWR::ptr()).ctlr.modify(|_, w| w.pdds().clear_bit());
                (*PFIC::ptr()).sctlr.modify(|_, w| w.sleepdeep().clear_bit());
                // (*PFIC::ptr()).sctlr.modify(|_, w| w.sleepdeep().clear_bit().wfitowfe().set_bit());
                core::ptr::write_volatile(&mut TIME_UP, false);
                while !core::ptr::read_volatile(&TIME_UP) {
                    asm!("wfi");
                }
            }
        }
    }

    #[cfg(feature = "interrupt")]
    pub fn sleep_sec(&mut self, duration: u32) {
        assert!(duration < 4_294_968); //(2 ^ (32 - 1)) / 1000
        self.sleep_ms(duration * 1000);
    }

    #[cfg(feature = "interrupt")]
    interrupt!(RTCALARM, Self::rtc_handler);
    #[cfg(feature = "interrupt")]
    fn rtc_handler() {
        unsafe {
            (*RTC::ptr()).ctlrl.modify(|_, w| w.alrf().clear_bit());
            (*EXTI::ptr()).intfr.modify(|_, w| w.if17().clear_bit_by_one());

            core::ptr::write_volatile(&mut TIME_UP, true);
        }
    }
}

impl delay::DelayUs<u32> for Delay {
    fn delay_us(&mut self, us: u32) {
        self.stop_count();
        // self.reset_counter();
        // HSI is 8MHz and counting Div 8. Thus 1 count is 1us.
        // self.hclk / 1_000_000 / 8 // cycles /us
        // control calc order to avoid overflowing max value of u32.
        let count = (us * (self.hclk / 1_000_000)) / 8; // cycles
        self.set_counter(0_u32 - count);

        self.start_count();
        // busy wait until over 2^32
        while !self.has_wrapped() {}
        self.stop_count();
    }
}

impl delay::DelayMs<u32> for Delay {
    fn delay_ms(&mut self, ms: u32) {
        self.delay_us((ms * 1_000_u32).into()); // 10^6 us = 1 s
    }
}