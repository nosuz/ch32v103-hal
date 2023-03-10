use core::marker::PhantomData;

// Extend PAC::GPIOX to get individual pins
pub trait GpioExt {
    type Parts;

    fn split(self) -> Self::Parts;
}

// Input Mode
pub struct Input<MODE> {
    _mode: PhantomData<MODE>,
}
// Modes for Input
pub struct Analog;
pub struct Floating;
pub struct PullDown;
pub struct PullUp;

// Output Mode
pub struct Output<MODE> {
    _mode: PhantomData<MODE>,
}
// Multiplex Output Mode
pub struct AltOutput<MODE> {
    _mode: PhantomData<MODE>,
}
// Modes for Output
pub struct PushPull;
pub struct OpenDrain;

// define Ports
macro_rules! gpio {
    (
        $GPIOX:ident,
        $gpiox:ident,
        $x:expr,
        [$($PXi:ident: ($pxi:ident, $i:expr, $MODE:ty, $CFGR:ident),)+]
    ) => {
        // define GPIOX
        pub mod $gpiox {
            use core::marker::PhantomData;
            use core::convert::Infallible;
            use embedded_hal::digital::v2::{ OutputPin, InputPin, StatefulOutputPin, ToggleableOutputPin };
            use ch32v1::ch32v103::RCC;
            use ch32v1::ch32v103::$GPIOX;

            // Use struct defined in outer scope
            use super::{
                GpioExt,
                Input, Analog, Floating, PullDown, PullUp,
                Output, AltOutput, PushPull, OpenDrain,
            };

            pub struct Parts {
                $(
                    // define pins with default state
                    pub $pxi: $PXi<$MODE>,
                )+
            }

            // Extend PAC::GPIOX
            impl GpioExt for $GPIOX {
                type Parts = Parts;

                fn split(self) -> Parts {
                    unsafe {
                        // (*RCC::ptr()).apb2pcenr.modify(|_, w| w.iopaen().set_bit());
                        (*RCC::ptr()).apb2pcenr.modify(|r, w| w.bits((r.bits() | (0b1 << ($x + 2)))));
                    }

                    Parts {
                        $(
                            $pxi: $PXi { _mode: PhantomData },
                        )+
                    }
                }
            }

            $(
                pub struct $PXi<MODE> {
                    _mode: PhantomData<MODE>,
                }

                // Impliment fn to set pins mode
                impl<MODE> $PXi<MODE> {
                    pub fn into_analog_input(self) -> $PXi<Input<Analog>> {
                        unsafe {
                            let offset = 4 * ($i & 0b111);
                            // Input mode, maximum speed: 50MHz;
                            let mode = 0b00;
                            // Analog input mode
                            let cnf = 0b00;
                            // Reset target bits, and set the target mode and cnf bits.
                            (*$GPIOX::ptr()).$CFGR.modify(|r, w| w.bits((r.bits() & !(0b1111 << offset) | (mode << offset) | (cnf << (offset + 2)))));
                            // Using PAC
                            // (*$GPIOX::ptr()).cfglr.modify(|_, w| w.cnf0().bits(0b01).mode0().bits(0b00));
                        }

                        $PXi { _mode: PhantomData }
                    }

                    pub fn into_floating_input(self) -> $PXi<Input<Floating>> {
                        unsafe {
                            let offset = 4 * ($i & 0b111);
                            // Input mode, maximum speed: 50MHz;
                            let mode = 0b00;
                            // Floating input mode
                            let cnf = 0b01;
                            // Reset target bits, and set the target mode and cnf bits.
                            (*$GPIOX::ptr()).$CFGR.modify(|r, w| w.bits((r.bits() & !(0b1111 << offset) | (mode << offset) | (cnf << (offset + 2)))));
                            // Using PAC
                            // (*$GPIOX::ptr()).cfglr.modify(|_, w| w.cnf0().bits(0b01).mode0().bits(0b00));
                        }

                        $PXi { _mode: PhantomData }
                    }

                    pub fn into_pull_up_input(self) -> $PXi<Input<PullUp>> {
                        unsafe {
                            let offset = 4 * ($i & 0b111);
                            // Input mode, maximum speed: 50MHz;
                            let mode = 0b00;
                            // Pull-up and pull-down input mode
                            let cnf = 0b10;
                            // Reset target bits, and set the target mode and cnf bits.
                            (*$GPIOX::ptr()).$CFGR.modify(|r, w| w.bits((r.bits() & !(0b1111 << offset) | (mode << offset) | (cnf << (offset + 2)))));
                            // Using PAC
                            // (*$GPIOX::ptr()).cfglr.modify(|_, w| w.cnf0().bits(0b10).mode0().bits(0b00));

                            // Set OUTDR for pull-up.
                            (*$GPIOX::ptr()).bshr.write(|w| w.bits(0b1 << $i));
                        }

                        $PXi { _mode: PhantomData }
                    }

                    pub fn into_pull_down_input(self) -> $PXi<Input<PullDown>> {
                        unsafe {
                            let offset = 4 * ($i & 0b111);
                            // Input mode, maximum speed: 50MHz;
                            let mode = 0b00;
                            // Pull-up and pull-down input mode
                            let cnf = 0b10;
                            // Reset target bits, and set the target mode and cnf bits.
                            (*$GPIOX::ptr()).$CFGR.modify(|r, w| w.bits((r.bits() & !(0b1111 << offset) | (mode << offset) | (cnf << (offset + 2)))));
                            // Using PAC
                            // (*$GPIOX::ptr()).cfglr.modify(|_, w| w.cnf0().bits(0b10).mode0().bits(0b00));

                            // Clear OUTDR for pull-down.
                            (*$GPIOX::ptr()).bcr.write(|w| w.bits(0b1 << $i));
                        }

                        $PXi { _mode: PhantomData }
                    }

                    pub fn into_open_drain_output(self) -> $PXi<Output<OpenDrain>> {
                        unsafe {
                            let offset = 4 * ($i & 0b111);
                            // Output mode, maximum speed: 50MHz;
                            let mode = 0b11;
                            // General open-drain output mode
                            let cnf = 0b01;
                            // Reset target bits, and set the target mode and cnf bits.
                            (*$GPIOX::ptr()).$CFGR.modify(|r, w| w.bits((r.bits() & !(0b1111 << offset) | (mode << offset) | (cnf << (offset + 2)))));
                            // Using PAC
                            // (*$GPIOX::ptr()).cfglr.modify(|_, w| w.cnf0().bits(0b01).mode0().bits(0b11));
                        }

                        $PXi { _mode: PhantomData }
                    }

                    pub fn into_push_pull_output(self) -> $PXi<Output<PushPull>> {
                        unsafe {
                            let offset = 4 * ($i & 0b111);
                            // Output mode, maximum speed: 50MHz;
                            let mode = 0b11;
                            // General push-pull output mode
                            let cnf = 0b00;
                            // Reset target bits, and set the target mode and cnf bits.
                            (*$GPIOX::ptr()).$CFGR.modify(|r, w| w.bits((r.bits() & !(0b1111 << offset) | (mode << offset) | (cnf << (offset + 2)))));
                            // Using PAC
                            // (*$GPIOX::ptr()).cfglr.modify(|_, w| w.cnf0().bits(0b00).mode0().bits(0b11));
                        }

                        $PXi { _mode: PhantomData }
                    }

                    pub fn into_multiplex_push_pull_output(self) -> $PXi<AltOutput<PushPull>> {
                        unsafe {
                            let offset = 4 * ($i & 0b111);
                            // Output mode, maximum speed: 50MHz;
                            let mode = 0b11;
                            // Multiplex push-pull output mode
                            let cnf = 0b10;
                            // Reset target bits, and set the target mode and cnf bits.
                            (*$GPIOX::ptr()).$CFGR.modify(|r, w| w.bits((r.bits() & !(0b1111 << offset) | (mode << offset) | (cnf << (offset + 2)))));
                            // Using PAC
                            // (*$GPIOX::ptr()).cfglr.modify(|_, w| w.cnf0().bits(0b00).mode0().bits(0b11));
                        }

                        $PXi { _mode: PhantomData }
                    }

                    pub fn into_multiplex_open_drain_output(self) -> $PXi<AltOutput<OpenDrain>> {
                        unsafe {
                            let offset = 4 * ($i & 0b111);
                            // Output mode, maximum speed: 50MHz;
                            let mode = 0b11;
                            // Multiplex open-drain output mode
                            let cnf = 0b11;
                            // Reset target bits, and set the target mode and cnf bits.
                            (*$GPIOX::ptr()).$CFGR.modify(|r, w| w.bits((r.bits() & !(0b1111 << offset) | (mode << offset) | (cnf << (offset + 2)))));
                            // Using PAC
                            // (*$GPIOX::ptr()).cfglr.modify(|_, w| w.cnf0().bits(0b01).mode0().bits(0b11));
                        }

                        $PXi { _mode: PhantomData }
                    }
                }

                // impl<MODE> OutputPin for $PXi<Output<MODE>> {
                //     fn set_high(&mut self) {
                //         unsafe { (*$GPIOX::ptr()).bsrr.write(|w| w.bits(1 << $i)) }
                //     }

                //     fn set_low(&mut self) {
                //         unsafe { (*$GPIOX::ptr()).bsrr.write(|w| w.bits(1 << (16 + $i))) }
                //     }
                // }

                // Impliment embedded-hal gpio
                impl<MODE> InputPin for $PXi<Input<MODE>> {
                    type Error = Infallible;

                    fn is_high(&self) -> Result<bool, Self::Error> {
                        unsafe {
                            Ok((*$GPIOX::ptr()).indr.read().bits() & (0b1 << $i) > 0)
                        }
                    }

                    fn is_low(&self) -> Result<bool, Self::Error> {
                        unsafe {
                            Ok((*$GPIOX::ptr()).indr.read().bits() & (0b1 << $i) == 0)
                        }
                    }
                }

                impl<MODE> OutputPin for $PXi<Output<MODE>> {
                    type Error = Infallible;

                    fn set_high(&mut self) -> Result<(), Self::Error> {
                        unsafe {
                            // Port set/reset register
                            (*$GPIOX::ptr()).bshr.write(|w| w.bits(0b1 << $i));
                            // Using PAC
                            // (*$GPIOX::ptr()).bshr.write(|w| w.bs7().set_bit());
                        }
                        Ok(())
                    }

                    fn set_low(&mut self) -> Result<(), Self::Error> {
                        unsafe {
                            // Port set/reset register
                            // (*$GPIOX::ptr()).bshr.write(|w| w.bits(0b1 << ($i + 16)));
                            // Port reset register
                            (*$GPIOX::ptr()).bcr.write(|w| w.bits(0b1 << $i));
                            // Using PAC
                            // (*$GPIOX::ptr()).bshr.write(|w| w.br7().set_bit());
                        }
                        Ok(())
                    }
                }

                impl<MODE> StatefulOutputPin for $PXi<Output<MODE>> {
                    // type Error = Infallible;

                    // Return last set value, not acutual state.
                    fn is_set_high(&self) -> Result<bool, Self::Error> {
                        unsafe {
                            // Ok((*$GPIOX::ptr()).indr.read().bits() & (0b1 << $i) > 0)
                            Ok((*$GPIOX::ptr()).outdr.read().bits() & (0b1 << $i) > 0)
                        }
                    }
                    fn is_set_low(&self) -> Result<bool, Self::Error> {
                        unsafe {
                            // Ok((*$GPIOX::ptr()).indr.read().bits() & (0b1 << $i) == 0)
                            Ok((*$GPIOX::ptr()).outdr.read().bits() & (0b1 << $i) == 0)
                        }
                    }
                }

                impl<MODE> ToggleableOutputPin for $PXi<Output<MODE>> {
                    type Error = Infallible;

                    fn toggle(&mut self) -> Result<(), Self::Error> {
                        if self.is_set_high().unwrap() {
                            self.set_low().unwrap();
                        } else {
                            self.set_high().unwrap();
                        }
                        Ok(())
                    }
                }
            )+
        }
    };
}

gpio!(GPIOA, gpioa, 0, [
    PA0: (pa0, 0, Input<Floating>, cfglr),
    PA1: (pa1, 1, Input<Floating>, cfglr),
    PA2: (pa2, 2, Input<Floating>, cfglr),
    PA3: (pa3, 3, Input<Floating>, cfglr),
    PA4: (pa4, 4, Input<Floating>, cfglr),
    PA5: (pa5, 5, Input<Floating>, cfglr),
    PA6: (pa6, 6, Input<Floating>, cfglr),
    PA7: (pa7, 7, Input<Floating>, cfglr),
    PA8: (pa8, 8, Input<Floating>, cfghr),
    PA9: (pa9, 9, Input<Floating>, cfghr),
    PA10: (pa10, 10, Input<Floating>, cfghr),
    PA11: (pa11, 11, Input<Floating>, cfghr), // Connected to USB on CH32V103R8T6-EVT-R1
    PA12: (pa12, 12, Input<Floating>, cfghr), // Connected to USB on CH32V103R8T6-EVT-R1
    PA13: (pa13, 13, Input<PullUp>, cfghr), // Connected DIO(SWDIO) on CH32V103R8T6-EVT-R1. Can release from WCH_Link
    PA14: (pa14, 14, Input<PullDown>, cfghr),  // Connected CLK(SWCLK) on CH32V103R8T6-EVT-R1. Can release from WCH_Link
    PA15: (pa15, 15, Input<PullUp>, cfghr),
]);

gpio!(GPIOB, gpiob, 1, [
    PB0: (pb0, 0, Input<Floating>, cfglr),
    PB1: (pb1, 1, Input<Floating>, cfglr),
    PB2: (pb2, 2, Input<Floating>, cfglr),
    PB3: (pb3, 3, Input<Floating>, cfglr),
    PB4: (pb4, 4, Input<Floating>, cfglr),
    PB5: (pb5, 5, Input<Floating>, cfglr),
    PB6: (pb6, 6, Input<Floating>, cfglr),
    PB7: (pb7, 7, Input<Floating>, cfglr),
    PB8: (pb8, 8, Input<Floating>, cfghr),
    PB9: (pb9, 9, Input<Floating>, cfghr),
    PB10: (pb10, 10, Input<Floating>, cfghr),
    PB11: (pb11, 11, Input<Floating>, cfghr),
    PB12: (pb12, 12, Input<Floating>, cfghr),
    PB13: (pb13, 13, Input<Floating>, cfghr),
    PB14: (pb14, 14, Input<Floating>, cfghr),
    PB15: (pb15, 15, Input<Floating>, cfghr),
]);

// GPIOC, GPIOD ...