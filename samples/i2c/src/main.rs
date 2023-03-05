#![no_std]
#![no_main]

// provide implementation for critical-section
use riscv_rt::entry;
use panic_halt as _;

use core::fmt::Write; // required for writeln!
use ch32v1::ch32v103; // PAC for CH32V103
use ch32v103_hal::prelude::*;
use ch32v103_hal::rcc::*;
use ch32v103_hal::gpio::*;
use ch32v103_hal::serial::*;
use ch32v103_hal::i2c::*;
use ch32v103_hal::delay::*;

const ADT7410_ADDR: u8 = 0x48;

#[entry]
fn main() -> ! {
    let peripherals = ch32v103::Peripherals::take().unwrap();
    let rcc = peripherals.RCC.constrain();

    // let clocks = rcc.cfgr.freeze();
    let clocks = rcc.cfgr
        .use_pll((48).mhz(), PllClkSrc::UseHsi)
        .hclk_prescale(HclkPreScale::Div4)
        .freeze();

    let gpioa = peripherals.GPIOA.split();
    let pa9 = gpioa.pa9.into_multiplex_push_pull_output();
    let pa10 = gpioa.pa10.into_floating_input();

    let usart = Serial::usart1(&clocks, (pa9, pa10), (115200).bps());
    let (tx, _) = usart.split();
    let mut log = SerialWriter::new(tx);

    let gpiob = peripherals.GPIOB.split();
    // LED
    let mut led1 = gpiob.pb2.into_push_pull_output();
    let mut led2 = gpiob.pb15.into_push_pull_output();

    // Use I2C1
    let pb6 = gpiob.pb6.into_multiplex_open_drain_output();
    let pb7 = gpiob.pb7.into_multiplex_open_drain_output();
    let pb8 = gpiob.pb8.into_multiplex_open_drain_output();
    let pb9 = gpiob.pb9.into_multiplex_open_drain_output();

    // required peripherals.I2C1 to occupy
    let i2c = I2c::i2c1(peripherals.I2C1, (pb6, pb7), &clocks);

    let mut delay = Delay::new(&clocks);
    let mut count = 0;

    led1.set_high().unwrap();
    led2.set_high().unwrap();

    // Reset ADT7310
    // i2c.start();
    // i2c.addr(ADT7410_ADDR, false);
    // i2c.send(0x2f);
    // i2c.stop();
    // delay.delay_ms(1);

    loop {
        led1.set_low().unwrap();
        led2.set_low().unwrap();

        // serial write with format sample.
        writeln!(&mut log, "Hello {}: {}", "world", count).unwrap();

        i2c.start();
        led1.set_high().unwrap();
        i2c.addr(ADT7410_ADDR, false);
        loop {
            writeln!(&mut log, "{:04X}", i2c.status()).unwrap();
        }
        led2.set_high().unwrap();
        i2c.send(0x08);
        i2c.start();
        i2c.addr(ADT7410_ADDR, true);
        // i2c.send(0x08);
        writeln!(&mut log, "{:02X}", i2c.read(false)).unwrap();
        i2c.stop();

        delay.delay_ms(5);

        // One-shot measure
        i2c.start();
        i2c.addr(ADT7410_ADDR, false);
        i2c.send(0x03);
        i2c.send(0x20);
        i2c.stop();

        delay.delay_ms(240);

        i2c.start();
        i2c.addr(ADT7410_ADDR, false);
        i2c.send(0x0);
        i2c.start();
        i2c.addr(ADT7410_ADDR, true);
        let temp_h = i2c.read(true);
        let temp_l = i2c.read(false);
        i2c.stop();

        let _temp = ((temp_h as u16) << 8) | (temp_l as u16);
        let temp = if (_temp & 0x8000) == 0 {
            ((_temp >> 3) as f32) / 16.0
        } else {
            (((_temp >> 3) as f32) - 8191.0) / 16.0
        };
        writeln!(&mut log, "{:+.1}", temp).unwrap();

        led1.set_high().unwrap();
        led2.set_high().unwrap();
        delay.delay_ms(200);
        count += 1;
    }
}