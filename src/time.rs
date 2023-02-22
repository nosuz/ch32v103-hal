/// Bits per second
#[derive(Clone, Copy)]
pub struct Bps(pub u32);

/// Hertz
#[derive(Clone, Copy)]
pub struct Hertz(pub u32);

/// Extension trait that adds convenience methods to the `u32` type
pub trait U32Ext {
    fn bps(self) -> Bps;

    fn hz(self) -> Hertz;

    fn khz(self) -> Hertz;

    fn mhz(self) -> Hertz;
}

impl U32Ext for u32 {
    fn bps(self) -> Bps {
        Bps(self)
    }

    fn hz(self) -> Hertz {
        Hertz(self)
    }

    fn khz(self) -> Hertz {
        Hertz(self * 1_000)
    }

    fn mhz(self) -> Hertz {
        Hertz(self * 1_000_000)
    }
}