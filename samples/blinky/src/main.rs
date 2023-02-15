#![no_std]
#![no_main]

// provide implementation for critical-section
use riscv_rt::entry;
use panic_halt as _;

use ch32v1::ch32v103; // PAC for CH32V103
use ch32v103_hal::prelude::*;
use ch32v103_hal::gpio::*;
use ch32v103_hal::rcc::*;
use ch32v1::ch32v103::TIM1;

#[entry]
fn main() -> ! {
    let peripherals = ch32v103::Peripherals::take().unwrap();
    let rcc = peripherals.RCC.constrain();

    // Waht is the good manner in Rust?
    rcc.apb2.enable_gpioa(); // Enable GPIOA
    rcc.apb2.enable_gpiob(); // Enable GPIOB
    // rcc.apb2pcenr.modify(|_, w| w.iopaen().set_bit());

    let gpioa = peripherals.GPIOA.split();
    let mut led_r1 = gpioa.pa4.into_push_pull_output();
    let mut led_r2 = gpioa.pa5.into_push_pull_output();

    let gpiob = peripherals.GPIOB.split();
    let mut led1 = gpiob.pb2.into_push_pull_output();
    let mut led2 = gpiob.pb15.into_push_pull_output();
    let mut led3 = gpiob.pb0.into_push_pull_output();

    // Push-pull at max 50MHz
    // unsafe {
    //     gpiob.cfglr.modify(|_, w| w.cnf0().bits(0b00).mode0().bits(0b11))
    // };

    // HSI 8MHz
    // 4 opcodes to do a nop sleep here
    // let wait_count = 8_000_000 / 4;
    unsafe {
        (*TIM1::ptr()).psc.write(|w| w.bits(15)); // 1us * 2^16 * 15 = 1s
        (*TIM1::ptr()).ctlr1.modify(|_, w| w.cen().set_bit());
    }
    loop {
        // gpiob.outdr.modify(|_, w| w.odr0().set_bit());
        led1.set_high().unwrap();
        led2.set_low().unwrap();
        led3.set_state(PinState::Low).unwrap();

        led_r1.set_high().unwrap();
        led_r2.set_low().unwrap();

        unsafe {
            while (*TIM1::ptr()).cnt.read().cnt() != 0 {}
        }

        // gpiob.outdr.modify(|_, w| w.odr0().clear_bit());
        led1.set_low().unwrap();
        led2.set_high().unwrap();
        led3.set_state(PinState::High).unwrap();

        led_r1.set_low().unwrap();
        led_r2.set_high().unwrap();
        unsafe {
            while (*TIM1::ptr()).cnt.read().cnt() != 0 {}
        }
    }
}