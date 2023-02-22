PKG_NAME=ch32v103_hal_blinky
BIN_NAME=firmware.bin

all: release

release:
	cargo build --release && \
	riscv64-unknown-elf-objcopy -O binary target/riscv32imac-unknown-none-elf/release/${PKG_NAME} ${BIN_NAME}

flash:
	wchisp flash ${BIN_NAME}

clean:
	rm -r target ${BIN_NAME}
