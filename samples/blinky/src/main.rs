#![no_std]
#![no_main]

// provide implementation for critical-section
use riscv_rt::entry;
use panic_halt as _;

use ch32v1::ch32v103; // PAC for CH32V103
use ch32v103_hal::prelude::*;
use ch32v103_hal::rcc::*;
use ch32v103_hal::gpio::*;
use ch32v103_hal::delay::*;

#[entry]
fn main() -> ! {
    let peripherals = ch32v103::Peripherals::take().unwrap();
    let rcc = peripherals.RCC.constrain();

    // let clocks = rcc.cfgr.freeze();
    // 72MHz not worked for me
    // let clocks = rcc.cfgr.use_pll((64).mhz(), PllClkSrc::Hsi).freeze();
    let clocks = rcc.cfgr.use_pll((48).mhz(), PllClkSrc::HsiDiv2).hclk((24).mhz()).freeze();

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

    let mut delay = Delay::new(&clocks);
    led2.set_high().unwrap();
    loop {
        led_r1.set_high().unwrap();
        delay.delay_ms(100);
        led_r1.set_low().unwrap();
        delay.delay_ms(100);

        // gpiob.outdr.modify(|_, w| w.odr0().set_bit());
        led1.set_high().unwrap();
        led3.set_state(PinState::Low).unwrap();

        led_r1.set_high().unwrap();
        led_r2.set_low().unwrap();

        // toggle LED2
        // if led2.is_set_high().unwrap() {
        //     led2.set_low().unwrap();
        // } else {
        //     led2.set_high().unwrap();
        // }
        led2.toggle().unwrap();

        delay.delay_ms(500);

        // gpiob.outdr.modify(|_, w| w.odr0().clear_bit());
        led1.set_low().unwrap();
        led3.set_state(PinState::High).unwrap();

        led_r1.set_low().unwrap();
        led_r2.set_high().unwrap();

        // toggle LED2
        led2.toggle().unwrap();
        delay.delay_ms(200);
        led2.toggle().unwrap();

        delay.delay_ms(500);
    }
}