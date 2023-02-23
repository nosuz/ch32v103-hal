use ch32v1::ch32v103::{ AFIO, RCC };

pub trait AfioExt {
    fn remap_usart1(&self);
}

impl AfioExt for AFIO {
    fn remap_usart1(&self) {
        unsafe {
            // clock is required before remap.
            (*RCC::ptr()).apb2pcenr.modify(|_, w| w.afioen().set_bit());
            (*AFIO::ptr()).pcfr.modify(|_, w| w.usart1rm().set_bit());
        }
    }
}