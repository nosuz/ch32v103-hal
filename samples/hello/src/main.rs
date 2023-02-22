#![no_std]
#![no_main]

// provide implementation for critical-section
use riscv_rt::entry;
use panic_halt as _;

use core::fmt::Write; // required for writeln!
use ch32v1::ch32v103; // PAC for CH32V103
use ch32v103_hal::prelude::*;
use ch32v103_hal::gpio::*;
use ch32v103_hal::serial::*;
use nb;
use ch32v103_hal::systick::SysTick;

#[entry]
fn main() -> ! {
    let peripherals = ch32v103::Peripherals::take().unwrap();
    let gpioa = peripherals.GPIOA.split();
    let pa9 = gpioa.pa9.into_multiplex_push_pull_output();
    let pa10 = gpioa.pa10.into_floating_input();

    let gpiob = peripherals.GPIOB.split();
    let mut led1 = gpiob.pb2.into_push_pull_output();
    let mut led2 = gpiob.pb15.into_push_pull_output();

    let usart = Serial::usart1((pa9, pa10), (115200).bps());
    let (tx, _) = usart.split();
    let mut log = SerialWriter::new(tx);

    led1.set_high().unwrap();
    led2.set_low().unwrap();

    let mut systick = SysTick::new();
    let mut count = 0;
    loop {
        led1.set_low().unwrap();
        led2.set_high().unwrap();

        // for c in b"Hello".iter() {
        //     // with unwrap(), VS code removes ! after nb::block
        //     nb::block!(tx.write(*c)); //.unwrap();
        // }

        writeln!(&mut log, "Hello {}: {}", "world", count).unwrap();

        led1.set_high().unwrap();
        led2.set_low().unwrap();
        systick.delay_ms(100);
        count += 1;
    }
}