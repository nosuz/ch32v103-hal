#![no_std]
#![no_main]

use core::cell::RefCell;
use critical_section::Mutex;

// provide implementation for critical-section
use ch32v103_rt::{ entry };
use panic_halt as _;

// use ch32v1::ch32v103; // PAC for CH32V103
use ch32v1::ch32v103::Peripherals;
use ch32v1::ch32v103::{ RCC, TIM1, PFIC };
use ch32v1::ch32v103::interrupt::Interrupt;
// use ch32v1::interrupt;

use ch32v103_hal::prelude::*;
use ch32v103_hal::rcc::*;
use ch32v103_hal::gpio::*;
use ch32v103_hal::delay::*;
use ch32v103_hal::interrupt;
use ch32v103_hal::gpio::gpiob::PB15;

// use core::fmt::Write; // required for writeln!
// use ch32v103_hal::serial::*;

// STM32F4 Embedded Rust at the HAL: GPIO Interrupts
// https://dev.to/apollolabsbin/stm32f4-embedded-rust-at-the-hal-gpio-interrupts-e5
// STM32F4 Embedded Rust at the HAL: Timer Interrupts
// https://apollolabsblog.hashnode.dev/stm32f4-embedded-rust-at-the-hal-timer-interrupts

type LedPin = PB15<Output<PushPull>>;
static LED: Mutex<RefCell<Option<LedPin>>> = Mutex::new(RefCell::new(None));

// patch is require for ch32v crate
// https://github.com/ch32-rs/ch32-rs/issues/3

// interrupt!(TIM1_UP, tim1_up, locals: {tick: bool = false;});
// fn tim1_up(locals: &mut TIM1_UP::Locals) {
//     unsafe {
//         (*TIM1::ptr()).intfr.modify(|_, w| w.uif().clear_bit());
//     }

//     locals.tick = !locals.tick;
//     critical_section::with(|cs| {
//         let mut led = LED.borrow(cs).borrow_mut();
//         if locals.tick {
//             led.as_mut().unwrap().set_high().unwrap();
//         } else {
//             led.as_mut().unwrap().set_low().unwrap();
//         }
//     });
// }

interrupt!(TIM1_UP, tim1_up);
fn tim1_up() {
    unsafe {
        (*TIM1::ptr()).intfr.modify(|_, w| w.uif().clear_bit());
    }

    critical_section::with(|cs| {
        let mut led = LED.borrow(cs).borrow_mut();
        led.as_mut().unwrap().toggle().unwrap();
    });
}

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take().unwrap();
    let rcc = peripherals.RCC.constrain();

    let clocks = rcc.cfgr.use_lsi().freeze();
    // 72MHz not worked for me
    // let clocks = rcc.cfgr.use_pll((64).mhz(), PllClkSrc::Hsi).use_lsi().freeze();
    // let clocks = rcc.cfgr
    //     .use_pll((48).mhz(), PllClkSrc::HsiDiv2)
    //     .hclk((24).mhz())
    //     .use_lsi()
    //     .freeze();

    // let gpioa = peripherals.GPIOA.split();

    let gpiob = peripherals.GPIOB.split();
    let mut led1 = gpiob.pb2.into_push_pull_output();
    let led2 = gpiob.pb15.into_push_pull_output();

    let mut delay = Delay::new(&clocks);

    // https://docs.rs/critical-section/latest/critical_section/#
    critical_section::with(|cs| {
        LED.borrow(cs).replace(Some(led2));
    });

    setup_timer1(&clocks);

    // Serial
    // let pa9 = gpioa.pa9.into_multiplex_push_pull_output();
    // let pa10 = gpioa.pa10.into_floating_input();
    //  remapped ports
    // let pb6 = gpiob.pb6.into_multiplex_push_pull_output();
    // let pb7 = gpiob.pb7.into_floating_input();

    // let usart = Serial::usart1(peripherals.USART1, (pa9, pa10), (115200).bps(), &clocks);
    // let (tx, _) = usart.split();
    // let mut log = SerialWriter::new(tx);

    led1.set_low().unwrap();
    for _ in 0..3 {
        // writeln!(&mut log, "START").unwrap();
        delay.sleep_ms(1000);
        led1.toggle().unwrap();
        delay.stop_ms(1000);
        led1.toggle().unwrap();
    }
    //  standby nad restart
    delay.standby_ms(3000);
    unreachable!();
}

fn setup_timer1(clocks: &Clocks) {
    unsafe {
        (*RCC::ptr()).apb2pcenr.modify(|_, w| w.tim1en().set_bit());

        let prescale = (clocks.hclk().0 / 1_000_000) * 100 - 1; // count for 0.1ms
        (*TIM1::ptr()).psc.write(|w| w.bits(prescale as u16));
        let down_count: u16 = 100 * 10 - 1; // 0.1ms * 10 * 100 = 100ms
        (*TIM1::ptr()).cnt.write(|w| w.bits(down_count));
        (*TIM1::ptr()).atrlr.write(|w| w.bits(down_count));
        (*TIM1::ptr()).ctlr1.modify(|_, w| w.arpe().set_bit().cen().set_bit());

        // clear interupt requist by the above counter update
        // (*TIM1::ptr()).intfr.modify(|_, w| w.uif().clear_bit());
        // (*PFIC::ptr()).iprr2.write(|w| w.bits(0b1 << ((Interrupt::TIM1_UP as u32) - 32)));

        // enable interrupt on Update. All 3 lines are require to enable correct interrupt.
        (*PFIC::ptr()).ienr2.modify(|_, w| w.bits(0b1 << ((Interrupt::TIM1_UP as u32) - 32)));
        (*TIM1::ptr()).dmaintenr.modify(|_, w| w.uie().set_bit());
        riscv::interrupt::enable();
    }
}