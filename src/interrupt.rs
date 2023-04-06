use core::arch::asm;
use riscv::register::{ mtvec, mcause };

use ch32v1::ch32v103::__EXTERNAL_INTERRUPTS;

// set interrupts vector
#[no_mangle]
pub extern "C" fn _setup_interrupts() {
    let handler: usize = _interrupt_handler as *const () as usize;
    unsafe {
        mtvec::write(handler, mtvec::TrapMode::Direct);
    }
}

#[no_mangle]
fn _interrupt_dispatcher() {
    // Refered https://github.com/rust-embedded/riscv-rt/blob/master/src/lib.rs
    unsafe {
        let cause = mcause::read();

        if cause.is_exception() {
            loop {
                continue;
            }
        } else {
            if cause.code() < __EXTERNAL_INTERRUPTS.len() {
                let h = &__EXTERNAL_INTERRUPTS[cause.code()];
                if h._reserved == 0 {
                    loop {
                        continue;
                    }
                } else {
                    (h._handler)();
                }
            } else {
                loop {
                    continue;
                }
            }
        }
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

// https://github.com/ch32-rs/ch32-rs-nightlies/blob/main/ch32v1/src/ch32v103/mod.rs
// https://github.com/ch32-rs/ch32-rs/issues/3

#[macro_export]
macro_rules! interrupt {
    (
        $ NAME: ident,
        $ path: path,
        locals: { $ ($ lvar: ident: $ lty: ty = $ lval: expr;) * }
    ) => { # [allow (non_snake_case)]
mod $ NAME { pub struct Locals { $ (pub $ lvar : $ lty ,) * } } # [allow (non_snake_case)]
# [no_mangle]
pub extern "C" fn $ NAME () { let _ = ch32v1::ch32v103 :: interrupt :: Interrupt :: $ NAME ; static mut LOCALS : self :: $ NAME :: Locals = self :: $ NAME :: Locals { $ ($ lvar : $ lval ,) * } ; let f : fn (& mut self :: $ NAME :: Locals) = $ path ; f (unsafe { & mut LOCALS }) ; } };
    (
        $ NAME: ident,
        $ path: path
    ) => { # [allow (non_snake_case)]
# [no_mangle]
pub extern "C" fn $ NAME () { let _ = ch32v1::ch32v103:: interrupt :: Interrupt :: $ NAME ; let f : fn () = $ path ; f () ; } };
}