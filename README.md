This is studying Rust embedded-hal for WCH CH32V103 project.

## Based Hal sample

[japaric/stm32f30x-hal](https://github.com/japaric/stm32f30x-hal)

## CH32V103R8T6-EVT-R1

Connect KEY pin to BOOT0 and BOOT1 to GND.
The KEY pin is Pull-Uped and goes to L by pressing the User button.

To download firmware, press RST button and release it.
To run downloaded firmware, press RST and User buttons and release them.

## Examples

```
$ cd samples/blinky && make release && make flash

```

## Debug macro_rules

```
$ cargo install cargo-expand
$ cargo expand --package ch32v103_hal
```

## Get assembly

[How to get assembly output from building with Cargo?](https://stackoverflow.com/questions/39219961/how-to-get-assembly-output-from-building-with-cargo)

```
$ cargo rustc --release -- --emit asm
$ ls target/<ARCH>/release/deps/*.s
```

or

```
$ cargo install cargo-show-asm
$ cargo asm --bin <PACKAGE.NAME>
```

## Object dump

[Embedded Rust Techniques 3-1. Cargo](https://tomoyuki-nakabayashi.github.io/embedded-rust-techniques/04-tools/cargo.html)

```
$ cargo install cargo-binutils
$ rustup component add llvm-tools-preview

$ cargo objdump --release -- -d --no-show-raw-insn --print-imm-hex
```
