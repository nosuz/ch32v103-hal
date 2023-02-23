// use core::convert::Infallible;
use ch32v1::ch32v103::RCC;
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
    UseHsi,
    UseHse,
    UsePll,
}

pub enum PllClkSrc {
    UseHsi,
    UseHse,
    UseHseDiv2,
}

pub enum HclkPreScale {
    Div2,
    Div4,
    Div8,
    Div16,
    Div64,
    Div128,
    Div256,
    Div512,
}

pub enum PclkPreScale {
    Div2,
    Div4,
    Div8,
    Div16,
}

// Clock configuration
pub struct CFGR {
    hse_freq: Option<u32>,
    hse_bypass: bool,
    pll_source: Option<PllClkSrc>,
    pll_freq: Option<u32>,
    sysclk_source: Option<Sysclk>,
    sysclk: Option<u32>,
    hclk_div: Option<HclkPreScale>,
    pclk1_div: Option<PclkPreScale>,
    pclk2_div: Option<PclkPreScale>,
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
                hclk_div: None,
                pclk1_div: None,
                pclk2_div: None,
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
        self.sysclk_source = Some(Sysclk::UseHsi);
        self
    }

    pub fn use_hse(mut self) -> Self {
        self.sysclk_source = Some(Sysclk::UseHse);
        self
    }

    pub fn use_pll(mut self, freq: Hertz, src: PllClkSrc) -> Self {
        assert!(freq.0 <= 72_000_000);
        self.sysclk_source = Some(Sysclk::UsePll);
        self.pll_source = Some(src);
        self.pll_freq = Some(freq.0);
        self
    }

    pub fn hclk_prescale(mut self, div: HclkPreScale) -> Self {
        self.hclk_div = Some(div);
        self
    }

    pub fn pclk1_prescale(mut self, div: PclkPreScale) -> Self {
        self.pclk1_div = Some(div);
        self
    }

    pub fn pclk12_prescale(mut self, div: PclkPreScale) -> Self {
        self.pclk2_div = Some(div);
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
            Some(Sysclk::UseHsi) => {
                unsafe {
                    (*RCC::ptr()).cfgr0.modify(|_, w| w.sw().bits(0));
                }
                self.sysclk = Some(HSI);
            }
            Some(Sysclk::UseHse) => {
                unsafe {
                    (*RCC::ptr()).cfgr0.modify(|_, w| w.sw().bits(0b1));
                }
                self.sysclk = self.hse_freq;
            }
            Some(Sysclk::UsePll) => {
                let mut pll_base_freq = HSI;
                match self.pll_source {
                    Some(PllClkSrc::UseHsi) => {
                        unsafe {
                            (*RCC::ptr()).cfgr0.modify(|_, w| w.pllsrc().clear_bit());
                        }
                    }
                    Some(PllClkSrc::UseHse) => {
                        assert!(self.hse_freq.is_some());
                        pll_base_freq = self.hse_freq.unwrap();
                        unsafe {
                            (*RCC::ptr()).cfgr0.modify(|_, w|
                                w.pllsrc().set_bit().pllxtpre().clear_bit()
                            );
                        }
                    }
                    Some(PllClkSrc::UseHseDiv2) => {
                        assert!(self.hse_freq.is_some());
                        pll_base_freq = self.hse_freq.unwrap() / 2;
                        unsafe {
                            (*RCC::ptr()).cfgr0.modify(|_, w|
                                w.pllsrc().set_bit().pllxtpre().set_bit()
                            );
                        }
                    }
                    None => {
                        // pll_source must be set if UsePll.
                        panic!("Why comes here");
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

        let (hpre_bits, hclk) = match self.hclk_div {
            Some(HclkPreScale::Div2) => (0b1000, sysclk / 2),
            Some(HclkPreScale::Div4) => (0b1001, sysclk / 4),
            Some(HclkPreScale::Div8) => (0b1010, sysclk / 8),
            Some(HclkPreScale::Div16) => (0b1011, sysclk / 16),
            Some(HclkPreScale::Div64) => (0b1100, sysclk / 64),
            Some(HclkPreScale::Div128) => (0b1101, sysclk / 128),
            Some(HclkPreScale::Div256) => (0b1110, sysclk / 256),
            Some(HclkPreScale::Div512) => (0b1111, sysclk / 512),
            None => (0b0111, sysclk),
        };
        unsafe {
            (*RCC::ptr()).cfgr0.modify(|_, w| w.hpre().bits(hpre_bits));
        }

        // APB1
        let (ppre1_bits, pclk1) = match self.pclk1_div {
            Some(PclkPreScale::Div2) => (0b100, hclk / 2),
            Some(PclkPreScale::Div4) => (0b101, hclk / 4),
            Some(PclkPreScale::Div8) => (0b110, hclk / 8),
            Some(PclkPreScale::Div16) => (0b111, hclk / 16),
            None => (0b011, hclk),
        };
        unsafe {
            (*RCC::ptr()).cfgr0.modify(|_, w| w.ppre1().bits(ppre1_bits));
        }

        // APB2
        let (ppre2_bits, pclk2) = match self.pclk2_div {
            Some(PclkPreScale::Div2) => (0b100, hclk / 2),
            Some(PclkPreScale::Div4) => (0b101, hclk / 4),
            Some(PclkPreScale::Div8) => (0b110, hclk / 8),
            Some(PclkPreScale::Div16) => (0b111, hclk / 16),
            None => (0b011, hclk),
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