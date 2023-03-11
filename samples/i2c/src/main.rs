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
const RESET_ADT7410: [u8; 1] = [0x2f];
const ONE_SHOT: [u8; 2] = [0x03, 0x20];
const READ_TEMP: [u8; 1] = [0x00];
const READ_ID: [u8; 1] = [0x08];
const READ_STATUS: [u8; 1] = [0x02];

#[entry]
fn main() -> ! {
    let peripherals = ch32v103::Peripherals::take().unwrap();
    let rcc = peripherals.RCC.constrain();

    // let clocks = rcc.cfgr.freeze();
    let clocks = rcc.cfgr.use_pll((48).mhz(), PllClkSrc::UseHsi).hclk((8).mhz()).freeze();

    let gpioa = peripherals.GPIOA.split();
    let pa9 = gpioa.pa9.into_multiplex_push_pull_output();
    let pa10 = gpioa.pa10.into_floating_input();

    let usart = Serial::usart1(peripherals.USART1, (pa9, pa10), (115200).bps(), &clocks);
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
    let mut i2c = I2c::i2c1(peripherals.I2C1, (pb6, pb7), I2cMode::Fast, &clocks);

    let mut delay = Delay::new(&clocks);

    led1.set_high().unwrap();
    led2.set_high().unwrap();

    // Reset ADT7310
    i2c.write(ADT7410_ADDR, &RESET_ADT7410).unwrap();
    // wait loading default config
    delay.delay_us(300);

    // finishing the first temp measurement
    loop {
        delay.delay_ms(10);

        let mut stat: [u8; 1] = [0x00];
        i2c.write_read(ADT7410_ADDR, &READ_STATUS, &mut stat).unwrap();
        if (stat[0] & 0x80) == 0 {
            break;
        }
    }

    loop {
        led1.set_low().unwrap();
        led2.set_low().unwrap();

        let mut id: [u8; 1] = [0x00];
        i2c.write_read(ADT7410_ADDR, &READ_ID, &mut id).unwrap();
        writeln!(&mut log, "ID:{:02X}", id[0]).unwrap();

        delay.delay_ms(1);

        i2c.write(ADT7410_ADDR, &ONE_SHOT).unwrap();
        delay.delay_ms(240);

        let mut buffer: [u8; 2] = [0x00; 2]; // init by 0x00
        i2c.write_read(ADT7410_ADDR, &READ_TEMP, &mut buffer).unwrap();
        let _temp = ((buffer[0] as u16) << 8) | (buffer[1] as u16);

        let temp = if (_temp & 0x8000) == 0 {
            ((_temp >> 3) as f32) / 16.0
        } else {
            (((_temp >> 3) as f32) - 8191.0) / 16.0
        };
        writeln!(&mut log, "Temp:{:+.1}", temp).unwrap();

        led1.set_high().unwrap();
        led2.set_high().unwrap();
        delay.delay_ms(200);
    }
}