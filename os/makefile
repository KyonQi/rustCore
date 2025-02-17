TARGET = riscv64gc-unknown-none-elf

BIN = target/$(TARGET)/release/os
BIN_OUT = $(BIN).bin

BOOTLOADER = ../bootloader/rustsbi-qemu.bin

ADDR = 0x80200000

export LOG ?= DEBUG

.PHONY: all build objcopy run clean

all: build objcopy

build:
	cargo build --release

objcopy: build
	rust-objcopy --strip-all $(BIN) -O binary $(BIN_OUT)

run: objcopy
	qemu-system-riscv64 -machine virt -nographic \
		-bios $(BOOTLOADER) \
		-device loader,file=$(BIN_OUT),addr=$(ADDR)
