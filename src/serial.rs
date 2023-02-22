use ch32v1::ch32v103::{ RCC, USART1 };

pub struct USART {}

impl USART {
    pub fn init() {
        unsafe {
            (*RCC::ptr()).apb2pcenr.modify(|_, w| w.usart1en().set_bit());

            // USARTDIV = Fclk / bps / 16
            // 8 MHz / 9600 / 16 = 52.08
            // 8 MHz / 115200 / 16 = 4.34
            // (*USART1::ptr()).statr.modify(|r, w| w.bits(r.bits() | w.txe.set_bit()));
            // let div = 52.08; // 9600bps
            let div = 4.34; // 115200bps
            let brr_div = (div * 16.0) as u32;
            (*USART1::ptr()).brr.write(|w| w.bits(brr_div));

            (*USART1::ptr()).ctlr1.modify(|_, w| w.ue().set_bit().te().set_bit().re().set_bit());
        }
    }

    pub fn write(chr: char) {
        unsafe {
            // check TX register is empty
            while ((*USART1::ptr()).statr.read().bits() & (0b1 << 7)) == 0 {}

            (*USART1::ptr()).datar.write(|w| w.bits(chr as u32));
            // block until complete transmittion
            // TC flag
            // while ((*USART1::ptr()).statr.read().bits() & (0b1 << 6)) == 0 {}
        }
    }

    pub fn read() -> u8 {
        unsafe {
            // check data received
            while ((*USART1::ptr()).statr.read().bits() & (0b1 << 5)) == 0 {}

            (*USART1::ptr()).datar.read().bits() as u8
        }
    }
}