#![no_std]
#![no_main]

// provide implementation for critical-section
use ch32v_rt::entry;
use panic_halt as _;

use ch32v1::ch32v103; // PAC for CH32V103
use ch32v103_hal::prelude::*;
use ch32v103_hal::rcc::*;
use ch32v103_hal::gpio::*;
use ch32v103_hal::serial::*;
use nb;
use ch32v103_hal::delay::*;

#[entry]
fn main() -> ! {
    let peripherals = ch32v103::Peripherals::take().unwrap();
    let rcc = peripherals.RCC.constrain();

    let clocks = rcc.cfgr.freeze();

    let gpioa = peripherals.GPIOA.split();
    let pa9 = gpioa.pa9.into_multiplex_push_pull_output();
    let pa10 = gpioa.pa10.into_floating_input();

    let gpiob = peripherals.GPIOB.split();
    let mut led1 = gpiob.pb2.into_push_pull_output();
    let mut led2 = gpiob.pb15.into_push_pull_output();

    let usart = Serial::usart1(peripherals.USART1, (pa9, pa10), (115200).bps(), &clocks);
    let (mut tx, mut rx) = usart.split();

    led1.set_high().unwrap();
    led2.set_low().unwrap();

    let mut delay = Delay::new(&clocks);

    #[allow(unused_assignments)]
    loop {
        // wait key press
        let result = nb::block!(rx.read());
        match result {
            Ok(chr) => {
                match nb::block!(tx.write(chr)) {
                    Ok(_) => {}
                    Err(_) => {}
                }

                if chr == 0x0d {
                    match nb::block!(tx.write(0x0a)) {
                        Ok(_) => {}
                        Err(_) => {}
                    }
                }
            }
            Err(_) => {
                // Ignore erro
            }
        }
        // let chr = result.unwrap();
        // if chr == 0x0d {
        //     newline = true;
        // } else {
        //     newline = false;
        // }
        // nb::block!(tx.write(chr)); //.unwrap();
        // if newline {
        //     nb::block!(tx.write(0x0a)); //.unwrap();
        // }

        led1.set_low().unwrap();
        led2.set_high().unwrap();
        delay.delay_ms(50);
        led1.set_high().unwrap();
        led2.set_low().unwrap();
    }
}