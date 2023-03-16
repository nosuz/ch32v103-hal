#![no_std]
#![no_main]

// use core::cell::RefCell;
// use riscv::interrupt::Mutex;

// provide implementation for critical-section
use riscv_rt::{ entry };
use panic_halt as _;
use core::arch::asm;

// use ch32v1::ch32v103; // PAC for CH32V103
use ch32v1::ch32v103::Peripherals;
use ch32v1::ch32v103::{ RCC, TIM1, GPIOA, PFIC };
use ch32v1::ch32v103::interrupt::Interrupt;

use ch32v103_hal::prelude::*;
use ch32v103_hal::rcc::*;
use ch32v103_hal::gpio::*;
use ch32v103_hal::delay::*;
// use ch32v103_hal::gpio::gpioa::PA5;

// STM32F4 Embedded Rust at the HAL: GPIO Interrupts
// https://dev.to/apollolabsbin/stm32f4-embedded-rust-at-the-hal-gpio-interrupts-e5
// STM32F4 Embedded Rust at the HAL: Timer Interrupts
// https://apollolabsblog.hashnode.dev/stm32f4-embedded-rust-at-the-hal-timer-interrupts

// type LedPin = PA5<Output<PushPull>>;
// static G_LED: Mutex<RefCell<Option<LedPin>>> = Mutex::new(RefCell::new(None));

// patch is require for ch32v crate
// https://github.com/ch32-rs/ch32-rs/issues/3
// interrupt!(TIM1_UP, tim1_up, locals: {tick: bool = false;});

// fn tim1_up(locals: &mut TIM1_UP::Locals) {
//     locals.tick = !locals.tick;

//     // unsafe {
//     //     (*TIM1::ptr()).intfr.modify(|_, w| w.uif().clear_bit());
//     //     // if locals.tick {
//     //     //     (*GPIOA::ptr()).bshr.write(|w| w.bs5().set_bit());
//     //     // } else {
//     //     //     (*GPIOA::ptr()).bshr.write(|w| w.br5().set_bit());
//     //     // }
//     //     riscv::interrupt::enable();
//     // }
// }

// interrupt!(TIM1_UP, tim1_up);
// fn tim1_up() {
//     unsafe {
//         asm!("mret");
//         // (*TIM1::ptr()).intfr.modify(|_, w| w.uif().clear_bit());
//         // (*GPIOA::ptr()).bshr.write(|w| w.bs5().set_bit());
//         // for _ in 0..11 {
//         //     riscv::asm::nop();
//         // }
//     }
// }

// set interrupts vector
#[no_mangle]
pub extern "C" fn _setup_interrupts() {
    let handler: usize = _interrupt_handler as *const () as usize;
    unsafe {
        riscv::register::mtvec::write(handler, riscv::register::mtvec::TrapMode::Direct);
    }
}

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take().unwrap();
    let rcc = peripherals.RCC.constrain();

    let clocks = rcc.cfgr.freeze();
    // 72MHz not worked for me
    // let clocks = rcc.cfgr.use_pll((64).mhz(), PllClkSrc::Hsi).freeze();
    // let clocks = rcc.cfgr.use_pll((48).mhz(), PllClkSrc::HsiDiv2).hclk((24).mhz()).freeze();

    let gpioa = peripherals.GPIOA.split();
    let mut io1 = gpioa.pa4.into_push_pull_output();
    let mut io2 = gpioa.pa5.into_push_pull_output();

    let mut delay = Delay::new(&clocks);

    io1.set_low().unwrap();

    io2.set_low().unwrap();
    delay.delay_ms(5);
    io2.set_high().unwrap();
    delay.delay_ms(5);
    io2.set_low().unwrap();
    delay.delay_ms(5);
    io2.set_high().unwrap();
    delay.delay_ms(5);

    setup_timer1(&clocks);

    loop {
        delay.delay_ms(50);
        io1.toggle().unwrap();
    }
}

fn setup_timer1(clocks: &Clocks) {
    unsafe {
        (*RCC::ptr()).apb2pcenr.modify(|_, w| w.tim1en().set_bit());

        let prescale = (clocks.hclk().0 / 1_000_000) * 100 - 1; // count for 0.1ms
        (*TIM1::ptr()).psc.write(|w| w.bits(prescale as u16));
        let down_count: u16 = 70 * 10 - 1; // 0.1ms * 10 * 70 = 70ms
        (*TIM1::ptr()).cnt.write(|w| w.bits(down_count));
        (*TIM1::ptr()).atrlr.write(|w| w.bits(down_count));
        (*TIM1::ptr()).ctlr1.modify(|_, w| w.arpe().set_bit().cen().set_bit());

        // clear interupt requist by the above counter update
        // (*TIM1::ptr()).intfr.modify(|_, w| w.uif().clear_bit());
        // (*PFIC::ptr()).iprr2.write(|w| w.bits(0b1 << ((Interrupt::TIM1_UP as u32) - 32)));

        // enable interrupt on Update. All 3 lines are require to enable correct interrupt.
        (*PFIC::ptr()).ienr2.write(|w| w.bits(0b1 << ((Interrupt::TIM1_UP as u32) - 32)));
        (*TIM1::ptr()).dmaintenr.modify(|_, w| w.uie().set_bit());
        riscv::interrupt::enable();
    }
}

#[no_mangle]
fn _interrupt_dispatcher() {
    unsafe {
        riscv::interrupt::free(|| {
            (*TIM1::ptr()).intfr.modify(|_, w| w.uif().clear_bit());
            (*GPIOA::ptr()).bshr.write(|w| w.br5().set_bit());
            for _ in 0..4_000 {
                riscv::asm::nop();
            }
            (*GPIOA::ptr()).bshr.write(|w| w.bs5().set_bit());
        });
    }
}

#[no_mangle]
pub extern "C" fn _interrupt_handler() {
    unsafe {
        asm!(
            "addi	sp, sp, -64",
            "sw	ra, 0(sp)",
            "sw	t0, 4(sp)",
            "sw	t1, 8(sp)",
            "sw	t2, 12(sp)",
            "sw	t3, 16(sp)",
            "sw	t4, 20(sp)",
            "sw	t5, 24(sp)",
            "sw	t6, 28(sp)",
            "sw	a0, 32(sp)",
            "sw	a1, 36(sp)",
            "sw	a2, 40(sp)",
            "sw	a3, 44(sp)",
            "sw	a4, 48(sp)",
            "sw	a5, 52(sp)",
            "sw	a6, 56(sp)",
            "sw	a7, 60(sp)",
            "add	a0, sp, zero",
            "jal	_interrupt_dispatcher",
            "lw	ra, 0(sp)",
            "lw	t0, 4(sp)",
            "lw	t1, 8(sp)",
            "lw	t2, 12(sp)",
            "lw	t3, 16(sp)",
            "lw	t4, 20(sp)",
            "lw	t5, 24(sp)",
            "lw	t6, 28(sp)",
            "lw	a0, 32(sp)",
            "lw	a1, 36(sp)",
            "lw	a2, 40(sp)",
            "lw	a3, 44(sp)",
            "lw	a4, 48(sp)",
            "lw	a5, 52(sp)",
            "lw	a6, 56(sp)",
            "lw	a7, 60(sp)",
            "addi	sp, sp, 64",
            "mret"
        )
    }
}