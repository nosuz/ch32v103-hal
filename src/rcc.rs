// use core::convert::Infallible;
use ch32v1::ch32v103::{ RCC, EXTEND };
use crate::time::*;

// pub struct AHB;
// pub struct APB1;
// pub struct APB2;

pub struct Rcc {
    // pub ahb: AHB,
    // pub apb1: APB1,
    // pub apb2: APB2,
    pub cfgr: CFGR,
}

const HSI: u32 = 8_000_000; // Hz

pub enum Sysclk {
    Hsi,
    Hse,
    Pll,
}

pub enum PllClkSrc {
    Hsi,
    // supported in extend_ctr
    HsiDiv2,
    Hse,
    HseDiv2,
}

// Clock configuration
pub struct CFGR {
    hse_freq: Option<u32>,
    hse_bypass: bool,
    pll_source: Option<PllClkSrc>,
    pll_freq: Option<u32>,
    sysclk_source: Option<Sysclk>,
    sysclk: Option<u32>,
    hclk_freq: Option<u32>,
    pclk1_freq: Option<u32>,
    pclk2_freq: Option<u32>,
}

pub struct Clocks {
    sysclk: Hertz,
    hclk: Hertz,
    pclk1: Hertz,
    pclk2: Hertz,
}

pub trait RccExt {
    type Rcc;
    fn constrain(self) -> Self::Rcc;
}

impl RccExt for RCC {
    type Rcc = Rcc;

    fn constrain(self) -> Rcc {
        // unsafe {
        //     (*RCC::ptr()).cfgr0.modify(|r, w| w.bits((r.bits() & !(0b1111 << 4)) | (0b1010 << 4))); // HPRE DIV8  8MHz / 8 = 1us /cycle
        //     (*RCC::ptr()).apb2pcenr.modify(|_, w| w.tim1en().set_bit());
        // }

        Rcc {
            // ahb: AHB {},
            // apb1: APB1 {},
            // apb2: APB2 {},
            cfgr: CFGR {
                hse_freq: None,
                hse_bypass: false,
                pll_source: None,
                pll_freq: None,
                sysclk_source: None,
                sysclk: None,
                hclk_freq: None,
                pclk1_freq: None,
                pclk2_freq: None,
            },
        }
    }
}

impl CFGR {
    pub fn hse_freq(mut self, freq: Hertz) -> Self {
        self.hse_freq = Some(freq.0);
        self
    }

    pub fn bypass_hse_oscillator(mut self) -> Self {
        self.hse_bypass = true;
        self
    }

    pub fn use_hsi(mut self) -> Self {
        self.sysclk_source = Some(Sysclk::Hsi);
        self
    }

    pub fn use_hse(mut self) -> Self {
        self.sysclk_source = Some(Sysclk::Hse);
        self
    }

    pub fn use_pll(mut self, freq: Hertz, src: PllClkSrc) -> Self {
        assert!(freq.0 <= 72_000_000);
        self.sysclk_source = Some(Sysclk::Pll);
        self.pll_source = Some(src);
        self.pll_freq = Some(freq.0);
        self
    }

    pub fn hclk(mut self, freq: Hertz) -> Self {
        self.hclk_freq = Some(freq.0);
        self
    }

    pub fn pclk1(mut self, freq: Hertz) -> Self {
        self.pclk1_freq = Some(freq.0);
        self
    }

    pub fn pclk2(mut self, freq: Hertz) -> Self {
        self.pclk2_freq = Some(freq.0);
        self
    }

    pub fn freeze(mut self) -> Clocks {
        if self.hse_freq.is_some() {
            // check HSE range
            if self.hse_bypass {
                assert!(self.hse_freq.unwrap() <= (25).mhz().0);

                unsafe {
                    (*RCC::ptr()).ctlr.modify(|_, w| w.hsebyp().set_bit());
                }
            } else {
                assert!(
                    (self.hse_freq.unwrap() >= (4).mhz().0) &
                        (self.hse_freq.unwrap() <= (16).mhz().0)
                );

                unsafe {
                    (*RCC::ptr()).ctlr.modify(|_, w| w.hsebyp().clear_bit().hseon().set_bit());
                    while !(*RCC::ptr()).ctlr.read().hserdy().bit_is_set() {}
                }
            }

            // setup HSE
        }

        match self.sysclk_source {
            Some(Sysclk::Hsi) => {
                unsafe {
                    (*RCC::ptr()).cfgr0.modify(|_, w| w.sw().bits(0));
                }
                self.sysclk = Some(HSI);
            }
            Some(Sysclk::Hse) => {
                unsafe {
                    (*RCC::ptr()).cfgr0.modify(|_, w| w.sw().bits(0b1));
                }
                self.sysclk = self.hse_freq;
            }
            Some(Sysclk::Pll) => {
                let mut pll_base_freq = HSI;
                match self.pll_source {
                    Some(PllClkSrc::Hsi) => {
                        unsafe {
                            (*RCC::ptr()).cfgr0.modify(|_, w| w.pllsrc().clear_bit());
                        }
                    }
                    Some(PllClkSrc::HsiDiv2) => {
                        pll_base_freq = HSI / 2;
                        unsafe {
                            (*EXTEND::ptr()).extend_ctr.modify(|_, w| w.hsipre().clear_bit());
                        }
                    }
                    Some(PllClkSrc::Hse) => {
                        assert!(self.hse_freq.is_some());
                        pll_base_freq = self.hse_freq.unwrap();
                        unsafe {
                            (*RCC::ptr()).cfgr0.modify(|_, w|
                                w.pllsrc().set_bit().pllxtpre().clear_bit()
                            );
                        }
                    }
                    Some(PllClkSrc::HseDiv2) => {
                        assert!(self.hse_freq.is_some());
                        pll_base_freq = self.hse_freq.unwrap() / 2;
                        unsafe {
                            (*RCC::ptr()).cfgr0.modify(|_, w|
                                w.pllsrc().set_bit().pllxtpre().set_bit()
                            );
                        }
                    }
                    None => {
                        // pll_source must be set if Pll.
                        unreachable!();
                    }
                }

                // setup PLL
                let pll_multi = self.pll_freq.unwrap() / pll_base_freq;
                assert!(pll_multi > 1);
                unsafe {
                    (*RCC::ptr()).cfgr0.modify(|_, w| w.pllmul().bits((pll_multi as u8) - 2));
                    // (*RCC::ptr()).cfgr0.modify(|_, w| w.pllmul().bits(0b110));
                    (*RCC::ptr()).ctlr.modify(|_, w| w.pllon().set_bit());
                    while !(*RCC::ptr()).ctlr.read().pllrdy().bit_is_set() {}
                    (*RCC::ptr()).cfgr0.modify(|_, w| w.sw().bits(0b10));
                }
                self.sysclk = Some(pll_base_freq * pll_multi);
            }
            None => {
                // use default HSI
                unsafe {
                    (*RCC::ptr()).cfgr0.modify(|_, w| w.sw().bits(0));
                }
                self.sysclk = Some(HSI);
            }
        }

        let sysclk = self.sysclk.unwrap();

        let (hpre_bits, hclk) = if self.hclk_freq.is_some() {
            match sysclk / self.hclk_freq.unwrap() {
                0..=1 => (0b0000, sysclk),
                2 => (0b1000, sysclk / 2),
                3..=6 => (0b1001, sysclk / 4),
                7..=11 => (0b1010, sysclk / 8),
                12..=31 => (0b1011, sysclk / 16),
                32..=90 => (0b1100, sysclk / 64),
                91..=181 => (0b1101, sysclk / 128),
                182..=362 => (0b1110, sysclk / 256),
                _ => (0b1111, sysclk / 512),
            }
        } else {
            (0b0000, sysclk)
        };
        unsafe {
            (*RCC::ptr()).cfgr0.modify(|_, w| w.hpre().bits(hpre_bits));
        }

        // APB1
        let (ppre1_bits, pclk1) = if self.pclk1_freq.is_some() {
            match hclk / self.pclk1_freq.unwrap() {
                0..=1 => (0b000, hclk),
                2 => (0b100, hclk / 2),
                3..=6 => (0b101, hclk / 4),
                7..=11 => (0b110, hclk / 8),
                _ => (0b111, hclk / 16),
            }
        } else {
            (0b000, hclk)
        };
        unsafe {
            (*RCC::ptr()).cfgr0.modify(|_, w| w.ppre1().bits(ppre1_bits));
        }

        // APB2
        let (ppre2_bits, pclk2) = if self.pclk2_freq.is_some() {
            match hclk / self.pclk2_freq.unwrap() {
                0..=1 => (0b000, hclk),
                2 => (0b100, hclk / 2),
                3..=6 => (0b101, hclk / 4),
                7..=11 => (0b110, hclk / 8),
                _ => (0b111, hclk / 16),
            }
        } else {
            (0b000, hclk)
        };
        unsafe {
            (*RCC::ptr()).cfgr0.modify(|_, w| w.ppre2().bits(ppre2_bits));
        }

        Clocks {
            sysclk: sysclk.hz(),
            hclk: hclk.hz(),
            pclk1: pclk1.hz(),
            pclk2: pclk2.hz(),
        }
    }
}

impl Clocks {
    pub fn sysclk(&self) -> Hertz {
        self.sysclk
    }

    pub fn hclk(&self) -> Hertz {
        self.hclk
    }

    pub fn pclk1(&self) -> Hertz {
        self.pclk1
    }

    pub fn pclk2(&self) -> Hertz {
        self.pclk2
    }
}