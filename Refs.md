## CH32V103

- [32-bit General Enhanced RISC-V MCU](https://github.com/openwch/ch32v103)

## Depending creates

- [ch32v1](https://crates.io/crates/ch32v1)
  - [riscv](https://crates.io/crates/riscv)

```
# svd command
# https://pypi.org/project/svdtools/
$ pip3 install --upgrade --user svdtools
# svd2rust command
$ cargo install svd2rust

$ git clone git@github.com:nosuz/ch32-rs.git
$ cd ch32-rs
$ ./scripts/generate.sh

$ cd ch32v1
$ cargo build
$ cargo build --all-features --release
```

## HAL

- [Embedded devices Working Group](https://github.com/rust-embedded/wg)
- [Demystifying Rust Embedded HAL Split and Constrain Methods](https://dev.to/apollolabsbin/demystifying-rust-embedded-hal-split-and-constrain-methods-591e)
- [HAL Design Patterns](https://doc.rust-lang.org/beta/embedded-book/design-patterns/hal/index.html)
- [rust-embedded/embedded-hal](https://github.com/rust-embedded/embedded-hal)
- [ch32-rs/ch32v20x-hal](https://github.com/ch32-rs/ch32v20x-hal)

* [4-Step Primer on Navigating Embedded Rust HAL Documentation](https://dev.to/apollolabsbin/4-step-primer-on-navigating-embedded-rust-hal-documentation-2d2m)
* [What the HAL? The Quest for Finding a Suitable Embedded Rust HAL](https://dev.to/apollolabsbin/what-the-hal-the-quest-for-finding-a-suitable-embedded-rust-hal-2i02)

### Implimentation

- [Crate embedded_hal](https://docs.rs/embedded-hal/latest/embedded_hal/)
  - [japaric/stm32f30x-hal](https://github.com/japaric/stm32f30x-hal)
- [Using Rust crates for an STM32 microcontroller board](https://stackoverflow.com/questions/67654351/using-rust-crates-for-an-stm32-microcontroller-board)
- [Module stm32f1xx_hal::gpio src](https://docs.rs/stm32f1xx-hal/0.6.1/src/stm32f1xx_hal/gpio.rs.html)
- [Use of Generics for Embedded HAL Structs](https://stackoverflow.com/questions/71653128/use-of-generics-for-embedded-hal-structs)
- [Embedded Rust Techniques](https://tomoyuki-nakabayashi.github.io/embedded-rust-techniques/01-introduction/introduction.html)

* [Running Rust on Microcontrollers](https://blog.mbedded.ninja/programming/languages/rust/running-rust-on-microcontrollers/)

#### SPI

- [bl602-hal/src/spi.rs](https://github.com/sipeed/bl602-hal/blob/main/src/spi.rs)
- [STM32 Rust hal で SPI を使ってみる](https://moons.link/post-1901/)

## Making Rust Library

- [逆引き Rust ライブラリシステム](https://qiita.com/nirasan/items/8804046c43ba07ee8fde)
- [Specifying path dependencies](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html)
