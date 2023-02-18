Tested on [WCH CH32V103R8T6-EVT-R1](https://www.lcsc.com/product-detail/_WCH-Jiangsu-Qin-Heng-_C2943982.html) evaluation board purchased from [LCSC](https://www.lcsc.com/).

## Setup

```
$ rustup target add riscv32imac-unknown-none-elf
$ sudo apt install binutils-riscv64-unknown-elf
$ cargo install wchisp --git https://github.com/ch32-rs/wchisp
```

## Build and flash

```
$ cargo build --release
$ riscv64-unknown-elf-objcopy -O binary target/riscv32imac-unknown-none-elf/release/ch32v1_study firmware.bin

# Set Boot0 to Vcc and Boot1 to GND. Then, connect a device to USB port.

$ lsusb | grep WinChip
Bus 001 Device 014: ID 4348:55e0 WinChipHead

$ wchisp flash firmware.bin
10:37:44 [WARN] Find chip via alternative id: 0x33
10:37:44 [INFO] Chip: CH32V103(C8T6|C8U6|R8T6)[0x3f15] (Code Flash: 64KiB)
10:37:44 [INFO] Chip UID: cd-ab-f4-6c-49-bc-0a-d5
10:37:44 [INFO] BTVER(bootloader ver): 02.70
10:37:44 [INFO] Code Flash protected: false
10:37:44 [INFO] Current config registers: a55aff00ffffffffffffffff
RDPR_USER: 0x00FF5AA5
  [7:0] RDPR 0b10100101 (0xA5)
    `- Unprotected
  [16:16] IWDG_SW 0b1 (0x1)
    `- IWDG enabled by the software, and disabled by hardware
  [17:17] STOP_RST 0b1 (0x1)
    `- Disable
  [18:18] STANDBY_RST 0b1 (0x1)
    `- Disable, entering standby-mode without RST
DATA: 0xFFFFFFFF
  [7:0] DATA0 0b11111111 (0xFF)
  [23:16] DATA1 0b11111111 (0xFF)
WRP: 0xFFFFFFFF
  `- Unprotected
10:37:44 [INFO] Read firmware.bin as Binary format
10:37:44 [INFO] Firmware size: 1024
10:37:44 [INFO] Erasing...
10:37:44 [WARN] erase_code: set min number of erased sectors to 8
10:37:44 [INFO] Erased 8 code flash sectors
10:37:45 [INFO] Erase done
10:37:45 [INFO] Writing to code flash...
████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████ 1024/102410:37:45 [INFO] Code flash 1024 bytes written
10:37:46 [INFO] Verifying...
████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████ 1024/102410:37:47 [INFO] Verify OK
10:37:47 [INFO] Now reset device and skip any communication errors
10:37:47 [INFO] Device reset

# Remove a device from USB port and set both Boot0 and Boot1 to GND
```

## Workaround for wchisp permission error

### permision

```bash
$ wchisp flash firmware.bin
Error: Access denied (insufficient permissions)
```

`sudo wchisp flash firmware.bin` not work since the PATH is changed.

```bash
$ pushd .
$ cd $HOME/.cargo/bin

$ sudo chown root:root wchisp
$ sudo chmod u+s wchisp

$ ls -l wchisp
-rwsrwxr-x 1 root root 5752696  1月  4 09:05 wchisp

$ popd
```

## Ref

- [ch32-rs/ch32v203-demo](https://github.com/ch32-rs/ch32v203-demo).
- [WCH の RISC-V MCU CH32V203 を Rust で L チカする](https://74th.hateblo.jp/entry/2022/12/22/223956)
