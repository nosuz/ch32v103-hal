#![no_std]
#![no_main]

// provide implementation for critical-section
use ch32v_rt::entry;
use panic_halt as _;

// use ch32v1::ch32v103; // PAC for CH32V103
use ch32v1::ch32v103::Peripherals;
use ch32v103_hal::prelude::*;
use ch32v103_hal::rcc::*;
use ch32v103_hal::gpio::*;
use ch32v103_hal::delay::*;

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take().unwrap();
    let rcc = peripherals.RCC.constrain();

    let clocks = rcc.cfgr.freeze();
    // 72MHz not worked for me
    // let clocks = rcc.cfgr.use_pll((64).mhz(), PllClkSrc::Hsi).freeze();
    // let clocks = rcc.cfgr.use_pll((48).mhz(), PllClkSrc::HsiDiv2).hclk((24).mhz()).freeze();

    let gpiob = peripherals.GPIOB.split();
    let mut led1 = gpiob.pb2.into_push_pull_output();

    // Push-pull at max 50MHz
    // unsafe {
    //     gpiob.cfglr.modify(|_, w| w.cnf0().bits(0b00).mode0().bits(0b11))
    // };

    let mut delay = Delay::new(&clocks);

    loop {
        led1.set_high().unwrap();
        delay.delay_ms(500);
        led1.set_low().unwrap();
        delay.delay_ms(500);
    }
}