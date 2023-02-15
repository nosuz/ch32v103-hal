PKG_NAME=ch32v103_hal_blinky
BIN_NAME=firmware.bin

all: build

build:
	cargo build && \
	riscv64-unknown-elf-objcopy -O binary target/riscv32imac-unknown-none-elf/debug/${PKG_NAME} ${BIN_NAME}

release:
	cargo build --release && \
	riscv64-unknown-elf-objcopy -O binary target/riscv32imac-unknown-none-elf/release/${PKG_NAME} ${BIN_NAME}

flash:
	wchisp flash ${BIN_NAME}
