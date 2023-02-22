#![no_std]
#![no_main]

// provide implementation for critical-section
use riscv_rt::entry;
use panic_halt as _;

use ch32v1::ch32v103; // PAC for CH32V103
use ch32v103_hal::prelude::*;
use ch32v103_hal::gpio::*;
use ch32v103_hal::serial::*;
use ch32v103_hal::systick::SysTick;

#[entry]
fn main() -> ! {
    let peripherals = ch32v103::Peripherals::take().unwrap();
    let gpioa = peripherals.GPIOA.split();
    let tx = gpioa.pa9.into_multiplex_push_pull_output();
    let rx = gpioa.pa10.into_floating_input();

    let gpiob = peripherals.GPIOB.split();
    let mut led1 = gpiob.pb2.into_push_pull_output();
    let mut led2 = gpiob.pb15.into_push_pull_output();

    // let usart = usart::init(gpioa, (115200).Hz(), stop_bit, parity_bit);
    USART::init();

    led1.set_high().unwrap();
    led2.set_low().unwrap();

    let mut systick = SysTick::new();
    loop {
        let chr = USART::read();
        USART::write(chr as char);
        if chr == 0x0d {
            USART::write(0x0a as char);
        }

        led1.set_low().unwrap();
        led2.set_high().unwrap();
        systick.delay_ms(50);
        led1.set_high().unwrap();
        led2.set_low().unwrap();
    }
}