#![no_std]
#![no_main]

// provide implementation for critical-section
use ch32v_rt::entry;
use panic_halt as _;

use core::fmt::Write; // required for writeln!
use ch32v1::ch32v103; // PAC for CH32V103
use ch32v103_hal::prelude::*;
use ch32v103_hal::rcc::*;
use ch32v103_hal::gpio::*;
use ch32v103_hal::serial::*;
use ch32v103_hal::delay::*;
use ch32v103_hal::spi::*;

#[entry]
fn main() -> ! {
    let peripherals = ch32v103::Peripherals::take().unwrap();
    let rcc = peripherals.RCC.constrain();

    // let clocks = rcc.cfgr.freeze();
    // let clocks = rcc.cfgr.use_pll((48).mhz(), PllClkSrc::Hsi).hclk((24).mhz()).freeze();
    let clocks = rcc.cfgr
        .use_pll((48).mhz(), PllClkSrc::Hsi)
        .hclk((24).mhz())
        .pclk2((6).mhz())
        .freeze();

    let gpioa = peripherals.GPIOA.split();
    let gpiob = peripherals.GPIOB.split();

    //SPI
    let mut pa4 = gpioa.pa4.into_push_pull_output(); // CS

    let pa5 = gpioa.pa5.into_multiplex_push_pull_output(); // SCK
    let pa6 = gpioa.pa6.into_floating_input(); // MISO
    let pa7 = gpioa.pa7.into_multiplex_push_pull_output(); // MOSI

    let mut spi = Spi::spi1(peripherals.SPI1, (pa5, pa6, pa7), MODE_3, (100).khz(), &clocks);

    // Serial
    let pa9 = gpioa.pa9.into_multiplex_push_pull_output();
    let pa10 = gpioa.pa10.into_floating_input();
    //  remapped ports
    // let pb6 = gpiob.pb6.into_multiplex_push_pull_output();
    // let pb7 = gpiob.pb7.into_floating_input();

    let usart = Serial::usart1(peripherals.USART1, (pa9, pa10), (115200).bps(), &clocks);
    let (tx, _) = usart.split();
    let mut log = SerialWriter::new(tx);

    let mut led1 = gpiob.pb2.into_push_pull_output();
    let mut led2 = gpiob.pb15.into_push_pull_output();

    led1.set_high().unwrap();
    led2.set_low().unwrap();

    let mut delay = Delay::new(&clocks);

    // Reset ADT7310
    pa4.set_low().unwrap();
    spi.write(&[0xff, 0xff, 0xff, 0xff]).unwrap();
    pa4.set_high().unwrap();
    delay.delay_us(500);

    loop {
        pa4.set_low().unwrap();
        spi.write(&[0x48]).unwrap();
        let mut stat: [u8; 1] = [0xff];
        let result = spi.transfer(&mut stat);
        match result {
            Ok(_) => {
                writeln!(&mut log, "STAT:{:02X}", stat[0]).unwrap();
            }
            Err(_) => {
                writeln!(&mut log, "Read status error").unwrap();
            }
        }
        pa4.set_high().unwrap();
        delay.delay_ms(1);

        // Read ADT7310 ID
        pa4.set_low().unwrap();
        match nb::block!(spi.send(0x58)) {
            Ok(_) => {}
            Err(_) => {}
        }
        match nb::block!(spi.read()) {
            Ok(_) => {}
            Err(_) => {}
        } //dummy
        match nb::block!(spi.send(0xff)) {
            Ok(_) => {}
            Err(_) => {}
        } //dummy
        let result = nb::block!(spi.read());
        pa4.set_high().unwrap();
        match result {
            Ok(id) => {
                writeln!(&mut log, "ID:{:02X}", id).unwrap();
            }
            Err(_) => {
                writeln!(&mut log, "Read ID error").unwrap();
            }
        }

        delay.delay_us(500);

        pa4.set_low().unwrap();
        spi.write(&[0x58]).unwrap();
        let mut id: [u8; 1] = [0x00];
        let result = spi.transfer(&mut id);
        pa4.set_high().unwrap();
        match result {
            Ok(_) => {
                writeln!(&mut log, "ID:{:02X}", id[0]).unwrap();
            }
            Err(_) => {
                writeln!(&mut log, "Read ID error").unwrap();
            }
        }

        delay.delay_ms(1);

        // Read temperature from ADT7310
        pa4.set_low().unwrap();
        // use blocking trait
        spi.write(&[0x08, 0x20]).unwrap();

        delay.delay_ms(250);

        // match nb::block!(spi.send(0x40)) {
        //     Ok(_) => {}
        //     Err(_) => {}
        // }
        // match nb::block!(spi.read()) {
        //     Ok(_) => {}
        //     Err(_) => {}
        // } //dummy
        // match nb::block!(spi.send(0xff)) {
        //     Ok(_) => {}
        //     Err(_) => {}
        // } //dummy
        // match nb::block!(spi.read()) {
        //     Ok(stat) => {
        //         writeln!(&mut log, "Stat:{:04X}", stat).unwrap();
        //     }
        //     Err(_) => {}
        // } //dummy

        // match nb::block!(spi.send(0x54)) {
        //     Ok(_) => {}
        //     Err(_) => {}
        // }
        // match nb::block!(spi.read()) {
        //     Ok(_) => {}
        //     Err(_) => {}
        // } //dummy
        // match nb::block!(spi.send(0xff)) {
        //     Ok(_) => {}
        //     Err(_) => {}
        // } //dummy
        // match nb::block!(spi.read()) {
        //     Ok(temp_h) => {
        //         match nb::block!(spi.send(0xff)) {
        //             Ok(_) => {}
        //             Err(_) => {}
        //         } //dummy
        //         match nb::block!(spi.read()) {
        //             Ok(temp_l) => {
        //                 let temp: u16 = ((temp_h as u16) << 8) | (temp_l as u16);
        //                 writeln!(&mut log, "Raw:{:04X}", temp).unwrap();

        //                 let _temp = if (temp & 0x1000) == 0 {
        //                     // When positive
        //                     ((temp >> 3) as f32) / 16.0
        //                 } else {
        //                     // When negative
        //                     (((temp >> 3) - 8192) as f32) / 16.0
        //                 };
        //                 writeln!(&mut log, "Temp:{:+.1}C", _temp).unwrap();
        //             }
        //             Err(_) => {
        //                 writeln!(&mut log, "Read TempL error").unwrap();
        //             }
        //         }
        //     }
        //     Err(_) => {
        //         writeln!(&mut log, "Read TempH error").unwrap();
        //     }
        // }
        // pa4.set_high().unwrap();

        // delay.delay_ms(1);

        // spi.write(&[0x54]).unwrap(); 0x54 for continuous mode only
        spi.write(&[0x50]).unwrap();
        let mut bytes: [u8; 2] = [0x00, 0x00];
        let result = spi.transfer(&mut bytes);
        pa4.set_high().unwrap();
        match result {
            Ok(_) => {
                writeln!(&mut log, "Raw:{:02X}, {:02X}", bytes[0], bytes[1]).unwrap();
                let temp: u16 = ((bytes[0] as u16) << 8) | (bytes[1] as u16);

                let _temp = if (temp & 0x1000) == 0 {
                    // When positive
                    ((temp >> 3) as f32) / 16.0
                } else {
                    // When negative
                    (((temp >> 3) - 8192) as f32) / 16.0
                };
                writeln!(&mut log, "Temp:{:+.1}C", _temp).unwrap();
            }
            Err(_) => {
                writeln!(&mut log, "Read Temp error").unwrap();
            }
        }

        delay.delay_us(500);

        pa4.set_low().unwrap();
        spi.write(&[0x48]).unwrap();
        let mut stat: [u8; 1] = [0xff];
        let result = spi.transfer(&mut stat);
        match result {
            Ok(_) => {
                writeln!(&mut log, "STAT:{:02X}", stat[0]).unwrap();
            }
            Err(_) => {
                writeln!(&mut log, "Read status error").unwrap();
            }
        }
        pa4.set_high().unwrap();

        led1.toggle().unwrap();
        led2.toggle().unwrap();
        delay.delay_ms(10);
    }
}