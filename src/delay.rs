use embedded_hal::prelude::*;
use embedded_hal::blocking::delay;

// SysTick (STK) register strucure is not same as CH32V20x and CH32V30x
use crate::rcc::*;

pub struct Delay {
    hclk: u32,
    ctlr: u32,
    cntl: u32,
    cnth: u32,
    // cmplr: u32,
    // cmphr: u32,
}

impl Delay {
    pub fn new(clocks: &Clocks) -> Self {
        const STK_BASE: u32 = 0xe000_f000;

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
            let valh = (self.cnth as *mut u32).read_volatile();
            valh > 0
        }
    }

    // fn reset_counter(&mut self) {
    //     unsafe {
    //         for i in 0..4 {
    //             ((self.cntl + i) as *mut u8).write_volatile(0);
    //             ((self.cnth + i) as *mut u8).write_volatile(0);
    //         }
    //     }
    // }

    fn set_counter(&mut self, value: u32) {
        unsafe {
            // CH32V103 (RISC V) is Little Endian.
            // 0: LSB, 3: MSB
            for i in 0..4 {
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
        self.delay_us(ms * 1_000_u32); // 10^6 us = 1 s
    }
}