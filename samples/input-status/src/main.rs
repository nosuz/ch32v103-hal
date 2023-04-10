#![no_std]
#![no_main]

// provide implementation for critical-section
use ch32v_rt::entry;
use panic_halt as _;

use ch32v1::ch32v103; // PAC for CH32V103
use ch32v103_hal::prelude::*;
use ch32v103_hal::gpio::*;

#[entry]
fn main() -> ! {
    let peripherals = ch32v103::Peripherals::take().unwrap();

    let gpioa = peripherals.GPIOA.split();
    let mut led_r1 = gpioa.pa4.into_push_pull_output();
    let mut led_r2 = gpioa.pa5.into_push_pull_output();

    let gpiob = peripherals.GPIOB.split();
    let mut led1 = gpiob.pb2.into_push_pull_output();
    let mut led2 = gpiob.pb15.into_push_pull_output();

    let button = gpiob.pb0.into_pull_up_input();
    // let button = gpiob.pb0.into_pull_down_input();

    loop {
        let status = button.is_high().unwrap();
        if status {
            led1.set_high().unwrap();
            led2.set_low().unwrap();

            led_r1.set_high().unwrap();
            led_r2.set_low().unwrap();
        } else {
            led1.set_low().unwrap();
            led2.set_high().unwrap();

            led_r1.set_low().unwrap();
            led_r2.set_high().unwrap();
        }
    }
}