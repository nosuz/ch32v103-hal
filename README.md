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
