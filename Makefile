# SPDX-License-Identifier: GPL-2.0-only
#
# Keira Kernel - Operating System Kernel
# Copyright (C) 2026 Moh. Ananda Firmansyah Putra
#
# This program is free software; you can redistribute it and/or modify
# it under the terms of the GNU General Public License as published by
# the Free Software Foundation; version 2 of the License.

# Master Build System Architecture
#
# Orchestrates the tri-language kernel compilation pipeline:
#   1. NASM (Assembly)  : Compiles 32-bit and 64-bit boot trampolines & ISR stubs.
#   2. GCC (C Driver)   : Compiles hardware drivers, PIC/PIT, and C heap allocator.
#   3. Cargo (Rust Core): Compiles `no_std` kernel core static library (`.a`).
#   4. LD (Linker)      : Links object files into a single ELF64 kernel executable.
#   5. GRUB (Bootloader): Packages kernel and USTAR initrd into a bootable ISO.

# Directory layout paths
BUILD_DIR     := build
OBJ_DIR       := $(BUILD_DIR)/obj
ISO_DIR       := $(BUILD_DIR)/isofiles
DISK_IMG      := $(BUILD_DIR)/disk.img

# Project naming & versioning extracted from Cargo.toml
KERNEL_NAME   := keira
VERSION       := $(shell grep -m 1 '^version = ' Cargo.toml | cut -d '"' -f 2)
KERNEL_BIN    := $(BUILD_DIR)/$(KERNEL_NAME).bin
DATE_SUFFIX   := $(shell date +%Y-%m-%d)
KERNEL_ISO    := $(BUILD_DIR)/$(KERNEL_NAME)-$(DATE_SUFFIX).iso

# Toolchain executables
ASM           := nasm
CC            := gcc
LD            := ld
CARGO         := cargo

# Assembler flags: ELF64 output format with assembly include path
ASM_FLAGS     := -f elf64 -I arch/x86/include/asm/

# C Compiler flags: freestanding 64-bit kernel mode without red zone or SSE
CC_FLAGS      := -ffreestanding \
	         -mno-red-zone \
	         -mno-mmx \
	         -mno-sse \
	         -mno-sse2 \
	         -mno-sse3 \
	         -mno-ssse3 \
	         -mno-sse4.1 \
	         -mno-sse4.2 \
	         -mno-avx \
	         -mno-avx2 \
	         -msoft-float \
	         -mcmodel=large \
	         -fno-stack-protector \
	         -fno-pic \
	         -nostdlib \
	         -m64 \
	         -I include \
	         -I arch/x86/include \
	         -I drivers \
	         -Wall -Wextra \
	         -O2

# Linker flags: large code model linking using custom linker script
LD_FLAGS      := -n \
	         -T arch/x86/linker.ld \
	         --gc-sections \
	         --no-warn-rwx-segments

# Rust freestanding compilation target & output library
RUST_TARGET   := targets/x86_64-keira-none.json
RUST_MODE     := release
RUST_LIB      := target/x86_64-keira-none/$(RUST_MODE)/libkeira_kernel.a

# QEMU emulator execution flags
QEMU          := qemu-system-x86_64
QEMU_FLAGS    := -cdrom $(KERNEL_ISO) \
	         -device ahci,id=ahci0 \
	         -drive file=$(DISK_IMG),format=raw,id=sata0,if=none \
	         -device ide-hd,drive=sata0,bus=ahci0.0 \
	         -audiodev none,id=snd0 \
	         -device intel-hda -device hda-duplex,audiodev=snd0 \
	         -boot d \
	         -serial stdio \
	         -no-shutdown \
	         -m 128M

# ANSI color codes for terminal feedback
ifeq ($(COLOR),0)
    CLR_RESET   :=
    CLR_BOLD    :=
    CLR_GREEN   :=
    CLR_YELLOW  :=
    CLR_BLUE    :=
    CLR_MAGENTA :=
    CLR_CYAN    :=
    CLR_ORANGE  :=
else
    CLR_RESET   := \033[0m
    CLR_BOLD    := \033[1m
    CLR_GREEN   := \033[32m
    CLR_YELLOW  := \033[33m
    CLR_BLUE    := \033[34m
    CLR_MAGENTA := \033[35m
    CLR_CYAN    := \033[36m
    CLR_ORANGE  := \033[38;5;208m
endif

LOG_ASM     := printf "  $(CLR_YELLOW)$(CLR_BOLD)[ASM]$(CLR_RESET)   %s\n"
LOG_CC      := printf "  $(CLR_BLUE)$(CLR_BOLD)[CC]$(CLR_RESET)    %s\n"
LOG_CARGO   := printf "  $(CLR_ORANGE)$(CLR_BOLD)[CARGO]$(CLR_RESET) %s\n"
LOG_LD      := printf "  $(CLR_MAGENTA)$(CLR_BOLD)[LD]$(CLR_RESET)    %s\n"
LOG_ISO     := printf "  $(CLR_MAGENTA)$(CLR_BOLD)[ISO]$(CLR_RESET)   %s\n"
LOG_DISK    := printf "  $(CLR_CYAN)$(CLR_BOLD)[DISK]$(CLR_RESET)  %s\n"
LOG_DONE    := printf "$(CLR_GREEN)$(CLR_BOLD)[DONE]$(CLR_RESET)  %s\n"
LOG_INFO    := printf "$(CLR_CYAN)$(CLR_BOLD)[INFO]$(CLR_RESET)  %s\n"

# Source code files
ASM_SRCS      := arch/x86/boot/multiboot2_header.asm \
	         arch/x86/boot/entry32.asm \
	         arch/x86/boot/entry64.asm \
	         arch/x86/kernel/gdt.asm \
	         arch/x86/kernel/paging.asm \
	         arch/x86/kernel/idt.asm \
	         arch/x86/kernel/isr.asm \
	         arch/x86/kernel/syscall.asm

C_SRCS        := drivers/serial/serial.c \
	         drivers/vga/vga.c \
	         drivers/sound/sound.c \
	         drivers/sound/hda.c \
	         arch/x86/kernel/idt.c \
	         arch/x86/kernel/pic.c \
	         arch/x86/kernel/pit.c \
	         drivers/keyboard/keyboard.c \
	         drivers/mouse/mouse.c \
	         drivers/rtc/rtc.c \
	         drivers/net/e1000.c \
	         mm/heap.c \
	         arch/x86/kernel/hw_init.c

ASM_OBJS      := $(patsubst %.asm,$(OBJ_DIR)/%.asm.o,$(ASM_SRCS))
C_OBJS        := $(patsubst %.c,$(OBJ_DIR)/%.c.o,$(C_SRCS))
ALL_OBJS      := $(ASM_OBJS) $(C_OBJS)

.PHONY: all run debug clean rust iso dirs format lint user disk initrd help

.DEFAULT_GOAL := all

# Build master bootable ISO image and FAT16 disk image
all: $(KERNEL_ISO) $(DISK_IMG)

# Display interactive target usage help
help:
	@printf "$(CLR_BOLD)Keira OS Build System (v$(VERSION))$(CLR_RESET)\n"
	@printf "Usage: make <target> [COLOR=0]\n\n"
	@printf "$(CLR_BOLD)Available Targets:$(CLR_RESET)\n"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2}'
	@printf "\n"

# User-space compilation configuration
USER_LIB_SRCS := user/lib/syscall.c user/lib/string.c user/lib/stdio.c user/lib/malloc.c
USER_CC_FLAGS  := -ffreestanding -nostdlib -fno-stack-protector -m64 -O2 -mno-sse -mno-sse2 -mno-mmx -mno-sse3 -mno-ssse3 -mno-sse4.1 -mno-sse4.2 -mno-avx -mno-avx2 -Iuser/include -T user/linker.ld -Wl,--no-warn-rwx-segments -static -no-pie

# Build user-space programs (init, gcc)
user: build/user_test.elf build/gcc.elf

build/user_test.elf: user/bin/init.c $(USER_LIB_SRCS) user/linker.ld | dirs
	@$(LOG_INFO) "Building user space program: init (user_test.elf)..."
	@$(CC) $(USER_CC_FLAGS) user/bin/init.c $(USER_LIB_SRCS) -o build/user_test.elf

build/gcc.elf: user/bin/gcc.c $(USER_LIB_SRCS) user/linker.ld | dirs
	@$(LOG_INFO) "Building user space program: gcc (gcc.elf)..."
	@$(CC) $(USER_CC_FLAGS) user/bin/gcc.c $(USER_LIB_SRCS) -o build/gcc.elf

# Create and populate FAT16 hard disk image
disk: $(DISK_IMG)

$(DISK_IMG): build/user_test.elf build/gcc.elf
	@rm -f $(DISK_IMG)
	@$(LOG_DISK) "Creating 32MB FAT16 disk image..."
	@dd if=/dev/zero of=$(DISK_IMG) bs=1M count=32 2>/dev/null
	@mkfs.fat -F 16 $(DISK_IMG) >/dev/null
	@$(LOG_DISK) "Creating nested Keira directory structure..."
	@mmd -i $(DISK_IMG) ::/system ::/system/bin ::/system/drivers ::/system/include ::/apps ::/apps/bin ::/apps/games ::/apps/src ::/config ::/config/boot ::/config/theme ::/users ::/users/admin ::/users/default ::/users/guest ::/temp ::/data ::/data/log ::/data/save 2>/dev/null || true
	@$(LOG_DISK) "Populating directories with command binaries..."
	@mkdir -p $(BUILD_DIR)/system_bin
	@for cmd in guide login drives use ramdisk system cpu runtime time memory \
	            devices wait initrd wipe reset run write tasks disk list \
	            go script view create folder delete edit say copy help history \
	            move theme please search play hda; do \
	    printf '#!/system/bin\n# Keira built-in command: %s\n# Type: kernel-mode binary\n# Path: /system/bin/%s\n' "$$cmd" "$$cmd" > $(BUILD_DIR)/system_bin/$$cmd; \
	    mcopy -o -i $(DISK_IMG) $(BUILD_DIR)/system_bin/$$cmd ::/system/bin/$$cmd; \
	done
	@$(LOG_DISK) "Copying driver files and system config..."
	@mkdir -p $(BUILD_DIR)/drivers
	@echo "Keira Serial Port Driver (COM1, 115200bps, 8N1)" > $(BUILD_DIR)/drivers/serial.sys
	@echo "Keira VGA Text Console Driver (80x25 characters, color support)" > $(BUILD_DIR)/drivers/vga.sys
	@echo "Keira PS/2 Keyboard Driver (US QWERTY layout)" > $(BUILD_DIR)/drivers/keyboard.sys
	@echo "Keira PS/2 Mouse Driver (basic coordinate tracking)" > $(BUILD_DIR)/drivers/mouse.sys
	@echo "Keira Real-Time Clock Driver (CMOS direct port communication)" > $(BUILD_DIR)/drivers/rtc.sys
	@echo "Keira IDE Storage Controller Driver (LBA28 read/write)" > $(BUILD_DIR)/drivers/ide.sys
	@echo "Keira AHCI SATA Storage Controller Driver (DMA read/write)" > $(BUILD_DIR)/drivers/ahci.sys
	@echo "Keira PC Speaker Sound Subsystem Driver (PIT Channel 2)" > $(BUILD_DIR)/drivers/sound.sys
	@for driver in serial.sys vga.sys keyboard.sys mouse.sys rtc.sys ide.sys ahci.sys sound.sys; do \
	    mcopy -o -i $(DISK_IMG) $(BUILD_DIR)/drivers/$$driver ::/system/drivers/$$driver; \
	done
	@echo "color_scheme=classic\nprompt_symbol=>\ncursor=block" > $(BUILD_DIR)/default.cfg
	@mcopy -o -i $(DISK_IMG) $(BUILD_DIR)/default.cfg ::/config/theme/default.cfg
	@$(LOG_DISK) "Copying binaries and configuration files..."
	@mcopy -o -i $(DISK_IMG) build/user_test.elf ::/apps/bin/user_test.elf
	@mcopy -o -i $(DISK_IMG) build/gcc.elf ::/apps/bin/gcc.elf
	@for header in stdio.h string.h syscall.h malloc.h; do mcopy -o -i $(DISK_IMG) user/include/$$header ::/system/include/$$header; done

# Build RAM Disk USTAR archive
initrd: $(BUILD_DIR)/initrd.tar

$(BUILD_DIR)/initrd.tar: build/user_test.elf build/gcc.elf
	@$(LOG_INFO) "Building RAM Disk (Initrd)..."
	@mkdir -p $(BUILD_DIR)/initrd_root/system/bin
	@mkdir -p $(BUILD_DIR)/initrd_root/system/drivers
	@mkdir -p $(BUILD_DIR)/initrd_root/system/include
	@mkdir -p $(BUILD_DIR)/initrd_root/apps/bin
	@mkdir -p $(BUILD_DIR)/initrd_root/config/boot
	@mkdir -p $(BUILD_DIR)/initrd_root/config/theme
	@mkdir -p $(BUILD_DIR)/initrd_root/users/admin
	@mkdir -p $(BUILD_DIR)/initrd_root/users/default
	@mkdir -p $(BUILD_DIR)/initrd_root/users/guest
	@mkdir -p $(BUILD_DIR)/initrd_root/temp
	@mkdir -p $(BUILD_DIR)/initrd_root/data
	@for cmd in guide login drives use ramdisk system cpu runtime time memory \
	            devices wait initrd wipe reset run write tasks disk list \
	            go script view create folder delete edit say copy help history \
	            move theme please search play hda; do \
	    printf '#!/system/bin\n# Keira built-in command: %s\n# Type: kernel-mode binary\n# Path: /system/bin/%s\n' "$$cmd" "$$cmd" > $(BUILD_DIR)/initrd_root/system/bin/$$cmd; \
	done
	@echo "Keira Serial Port Driver (COM1, 115200bps, 8N1)" > $(BUILD_DIR)/initrd_root/system/drivers/serial.sys
	@echo "Keira VGA Text & Widescreen Console Driver (color support)" > $(BUILD_DIR)/initrd_root/system/drivers/vga.sys
	@echo "Keira PS/2 Keyboard Driver (US QWERTY layout)" > $(BUILD_DIR)/initrd_root/system/drivers/keyboard.sys
	@echo "Keira PS/2 Mouse Driver (basic coordinate tracking)" > $(BUILD_DIR)/initrd_root/system/drivers/mouse.sys
	@echo "Keira Real-Time Clock Driver (CMOS direct port communication)" > $(BUILD_DIR)/initrd_root/system/drivers/rtc.sys
	@echo "Keira IDE Storage Controller Driver (LBA28 read/write)" > $(BUILD_DIR)/initrd_root/system/drivers/ide.sys
	@echo "Keira AHCI SATA Storage Controller Driver (DMA read/write)" > $(BUILD_DIR)/initrd_root/system/drivers/ahci.sys
	@echo "Keira PC Speaker Sound Subsystem Driver (PIT Channel 2)" > $(BUILD_DIR)/initrd_root/system/drivers/sound.sys
	@echo "color_scheme=classic\nprompt_symbol=>\ncursor=block" > $(BUILD_DIR)/initrd_root/config/theme/default.cfg
	@cp build/user_test.elf $(BUILD_DIR)/initrd_root/apps/bin/user_test.elf
	@cp build/gcc.elf $(BUILD_DIR)/initrd_root/apps/bin/gcc.elf
	@cp user/include/*.h $(BUILD_DIR)/initrd_root/system/include/
	@cd $(BUILD_DIR)/initrd_root && tar -cf ../initrd.tar *

# Package GRUB Multiboot2 bootable ISO image
iso: $(KERNEL_ISO)

$(KERNEL_ISO): $(KERNEL_BIN) $(BUILD_DIR)/initrd.tar | dirs
	@$(LOG_ISO) "Creating bootable ISO..."
	@mkdir -p $(ISO_DIR)/boot/grub
	@cp $(KERNEL_BIN) $(ISO_DIR)/boot/$(KERNEL_NAME).bin
	@cp $(BUILD_DIR)/initrd.tar $(ISO_DIR)/boot/initrd.tar
	@echo 'set timeout=0' > $(ISO_DIR)/boot/grub/grub.cfg
	@echo 'set default=0' >> $(ISO_DIR)/boot/grub/grub.cfg
	@echo '' >> $(ISO_DIR)/boot/grub/grub.cfg
	@echo 'menuentry "Keira" {' >> $(ISO_DIR)/boot/grub/grub.cfg
	@echo '	multiboot2 /boot/keira.bin' >> $(ISO_DIR)/boot/grub/grub.cfg
	@echo '	module2 /boot/initrd.tar initrd' >> $(ISO_DIR)/boot/grub/grub.cfg
	@echo '	boot' >> $(ISO_DIR)/boot/grub/grub.cfg
	@echo '}' >> $(ISO_DIR)/boot/grub/grub.cfg
	@grub-mkrescue -o $(KERNEL_ISO) $(ISO_DIR) 2>/dev/null
	@$(LOG_DONE) "$(KERNEL_ISO) ready"

# Link final kernel ELF64 binary executable
$(KERNEL_BIN): $(ALL_OBJS) $(RUST_LIB) arch/x86/linker.ld | dirs
	@$(LOG_LD) "Linking kernel..."
	@$(LD) $(LD_FLAGS) -o $(KERNEL_BIN) $(ALL_OBJS) $(RUST_LIB)
	@$(LOG_DONE) "$(KERNEL_BIN) ready"

# Compile freestanding Rust kernel module
rust:
	@$(LOG_CARGO) "Building Rust kernel ($(RUST_MODE))...."
	@$(CARGO) -Zjson-target-spec -Zbuild-std=core build --target $(RUST_TARGET) --$(RUST_MODE) 2>&1 | sed 's/^/        /'

$(RUST_LIB): rust

# Compile Assembly source files (.asm -> .o)
$(OBJ_DIR)/%.asm.o: %.asm | dirs
	@$(LOG_ASM) "$<"
	@mkdir -p $(dir $@)
	@$(ASM) $(ASM_FLAGS) -o $@ $<

# Compile C source files (.c -> .o)
$(OBJ_DIR)/%.c.o: %.c | dirs
	@$(LOG_CC) "$<"
	@mkdir -p $(dir $@)
	@$(CC) $(CC_FLAGS) -c -o $@ $<

# Create target build directories
dirs:
	@mkdir -p $(BUILD_DIR) $(OBJ_DIR)

# Launch system in QEMU virtual machine
run: all
	@$(LOG_INFO) "Launching Keira in QEMU..."
	@$(QEMU) $(QEMU_FLAGS)

# Launch system in QEMU debug mode with GDB stub
debug: all
	@$(LOG_INFO) "Launching Keira (debug mode, waiting for GDB on :1234)..."
	@$(QEMU) $(QEMU_FLAGS) -S -s

# Remove build directory and compiled artifacts
clean:
	@$(LOG_INFO) "Removing build artifacts..."
	@rm -rf $(BUILD_DIR)
	@$(CARGO) clean 2>/dev/null || true
	@$(LOG_DONE) "Clean complete"

# Format Rust and C source code
format:
	@$(LOG_INFO) "Formatting Rust code..."
	@$(CARGO) fmt --all
	@$(LOG_INFO) "Formatting C code..."
	@find . -path "./build" -prune -o -type f \( -name "*.c" -o -name "*.h" \) -exec clang-format -i {} +
	@$(LOG_DONE) "Formatting complete"

# Static linting of C code using clang-tidy
lint:
	@$(LOG_INFO) "Linting C code..."
	@find . -path "./build" -prune -o -type f \( -name "*.c" -o -name "*.h" \) -exec clang-tidy --quiet {} -- -I include -I drivers -I arch/x86/include -I user/include -ffreestanding -m64 \;
	@$(LOG_DONE) "Linting complete"
	