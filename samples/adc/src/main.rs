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
use ch32v103_hal::adc::*;
use ch32v103_hal::delay::*;

#[entry]
fn main() -> ! {
    let peripherals = ch32v103::Peripherals::take().unwrap();
    let rcc = peripherals.RCC.constrain();

    // let clocks = rcc.cfgr.freeze();
    let clocks = rcc.cfgr.use_pll((48).mhz(), PllClkSrc::Hsi).hclk((24).mhz()).freeze();

    let gpioa = peripherals.GPIOA.split();
    let pa9 = gpioa.pa9.into_multiplex_push_pull_output();
    let pa10 = gpioa.pa10.into_floating_input();

    let usart = Serial::usart1(peripherals.USART1, (pa9, pa10), (115200).bps(), &clocks);
    let (tx, _) = usart.split();
    let mut log = SerialWriter::new(tx);

    let gpiob = peripherals.GPIOB.split();
    let mut led1 = gpiob.pb2.into_push_pull_output();
    let mut led2 = gpiob.pb15.into_push_pull_output();

    led1.set_high().unwrap();
    led2.set_low().unwrap();

    let mut delay = Delay::new(&clocks);

    let mut adc_in = gpioa.pa0.into_analog_input();
    let mut adc = Adc::adc(peripherals.ADC, &clocks);

    let cal = adc.calibration();

    loop {
        writeln!(&mut log, "Calb: {:04x}", cal).unwrap();

        led1.set_low().unwrap();
        led2.set_high().unwrap();

        let raw: u16 = adc.read(&mut adc_in).unwrap();
        writeln!(&mut log, "ADC: {}", raw).unwrap();

        let v = ((raw as f32) * 3.3) / 4095.0;
        writeln!(&mut log, "{:.2}V", v).unwrap();

        let tm = adc.read_temp();
        let temp = (((tm as f32) * 3.3) / 4095.0 - 1.34) / 4.3 + 25.0;
        writeln!(&mut log, "Temp:{:.2}C", temp).unwrap();

        led1.set_high().unwrap();
        led2.set_low().unwrap();

        delay.delay_ms(200);
    }
}