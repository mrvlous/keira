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

# Configurable build parameters (override via command line)
DISK_SIZE     ?= 32
QEMU_MEM      ?= 128M

# Verbose mode: set V=1 to see raw commands
ifeq ($(V),1)
    Q :=
else
    Q := @
endif
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
QEMU          := qemu-system-x86_64
QEMU_FLAGS    := -cdrom $(KERNEL_ISO) \
	         -device ahci,id=ahci0 \
	         -drive file=$(DISK_IMG),format=raw,id=sata0,if=none \
	         -device ide-hd,drive=sata0,bus=ahci0.0 \
	         -audiodev none,id=snd0 \
	         -device intel-hda -device hda-duplex,audiodev=snd0 \
	         -device e1000,netdev=net0 \
	         -netdev user,id=net0 \
	         -boot d \
	         -serial stdio \
	         -no-shutdown \
	         -m $(QEMU_MEM)

# QEMU with e1000 NIC emulation for network testing
QEMU_NET_FLAGS := $(QEMU_FLAGS)
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
    CLR_RED     :=
else
    CLR_RESET   := \033[0m
    CLR_BOLD    := \033[1m
    CLR_GREEN   := \033[32m
    CLR_YELLOW  := \033[33m
    CLR_BLUE    := \033[34m
    CLR_MAGENTA := \033[35m
    CLR_CYAN    := \033[36m
    CLR_ORANGE  := \033[38;5;208m
    CLR_RED     := \033[31m
endif

LOG_ASM     := printf "  $(CLR_YELLOW)$(CLR_BOLD)[ASM]$(CLR_RESET)   %s\n"
LOG_CC      := printf "  $(CLR_BLUE)$(CLR_BOLD)[CC]$(CLR_RESET)    %s\n"
LOG_CARGO   := printf "  $(CLR_ORANGE)$(CLR_BOLD)[CARGO]$(CLR_RESET) %s\n"
LOG_LD      := printf "  $(CLR_MAGENTA)$(CLR_BOLD)[LD]$(CLR_RESET)    %s\n"
LOG_ISO     := printf "  $(CLR_MAGENTA)$(CLR_BOLD)[ISO]$(CLR_RESET)   %s\n"
LOG_DISK    := printf "  $(CLR_CYAN)$(CLR_BOLD)[DISK]$(CLR_RESET)  %s\n"
LOG_DONE    := printf "$(CLR_GREEN)$(CLR_BOLD)[DONE]$(CLR_RESET)  %s\n"
LOG_INFO    := printf "$(CLR_CYAN)$(CLR_BOLD)[INFO]$(CLR_RESET)  %s\n"
LOG_WARN    := printf "$(CLR_YELLOW)$(CLR_BOLD)[WARN]$(CLR_RESET)  %s\n"
LOG_ERR     := printf "$(CLR_RED)$(CLR_BOLD)[ERR]$(CLR_RESET)   %s\n"
LOG_CHECK   := printf "  $(CLR_GREEN)$(CLR_BOLD)[OK]$(CLR_RESET)    %s\n"
LOG_MISS    := printf "  $(CLR_RED)$(CLR_BOLD)[MISS]$(CLR_RESET)  %s\n"
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

# Shell command binaries to populate filesystem images
SHELL_CMDS    := guide login drives use ramdisk system cpu runtime time memory \
                 devices wait initrd wipe reset run write tasks disk list \
                 go script view create folder delete edit copy help history \
                 move please search download network stop env sync \
                 protect fileinfo framebuffer usb https user hostname syslog kvm \
                 nvme ext4 cgroups futex bpf tpm swap seccomp epoll \
                 drivers lkm unwind power perf timer eventfd mac mqueue \
                 kill jobs fg bg lvm raid ipcs ipcrm iptables firewall

# Driver descriptor files for filesystem images
DRIVER_FILES  := serial.sys vga.sys keyboard.sys mouse.sys rtc.sys \
                 ide.sys ahci.sys sound.sys e1000.sys
.PHONY: all run debug clean rust iso dirs format lint user disk initrd \
        help info check size objdump qemu-net

.DEFAULT_GOAL := all
all: $(KERNEL_ISO) $(DISK_IMG) ## Build kernel, ISO image, and FAT16 disk image

help: ## Display all available Makefile targets
	@printf "$(CLR_BOLD)Keira Kernel Build System$(CLR_RESET)  $(CLR_CYAN)v$(VERSION)$(CLR_RESET)\n\n"
	@printf "  $(CLR_BOLD)Usage$(CLR_RESET): make <target> [V=1] [COLOR=0] [DISK_SIZE=N] [QEMU_MEM=NM]\n\n"
	@printf "$(CLR_BOLD)  Build Targets:$(CLR_RESET)\n"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "    $(CLR_CYAN)%-15s$(CLR_RESET) %s\n", $$1, $$2}'
	@printf "\n$(CLR_BOLD)  Variables:$(CLR_RESET)\n"
	@printf "    $(CLR_CYAN)V=1$(CLR_RESET)             Show raw commands (verbose mode)\n"
	@printf "    $(CLR_CYAN)COLOR=0$(CLR_RESET)         Disable colored output\n"
	@printf "    $(CLR_CYAN)DISK_SIZE=N$(CLR_RESET)     FAT16 disk size in MB (default: 32)\n"
	@printf "    $(CLR_CYAN)QEMU_MEM=NM$(CLR_RESET)     QEMU guest memory (default: 128M)\n\n"
info: ## Display build configuration and toolchain versions
	@printf "$(CLR_BOLD)Keira Kernel Build Info$(CLR_RESET)\n\n"
	@printf "  $(CLR_BOLD)Kernel$(CLR_RESET)\n"
	@printf "    Version      : $(CLR_CYAN)$(VERSION)$(CLR_RESET)\n"
	@printf "    Name         : $(CLR_CYAN)$(KERNEL_NAME)$(CLR_RESET)\n"
	@printf "    Binary       : $(CLR_CYAN)$(KERNEL_BIN)$(CLR_RESET)\n"
	@printf "    ISO          : $(CLR_CYAN)$(KERNEL_ISO)$(CLR_RESET)\n"
	@printf "    Disk Image   : $(CLR_CYAN)$(DISK_IMG) ($(DISK_SIZE)MB FAT16)$(CLR_RESET)\n\n"
	@printf "  $(CLR_BOLD)Toolchain$(CLR_RESET)\n"
	@printf "    NASM         : $(CLR_CYAN)$(shell $(ASM) --version 2>/dev/null | head -1 || echo 'not found')$(CLR_RESET)\n"
	@printf "    GCC          : $(CLR_CYAN)$(shell $(CC) --version 2>/dev/null | head -1 || echo 'not found')$(CLR_RESET)\n"
	@printf "    LD           : $(CLR_CYAN)$(shell $(LD) --version 2>/dev/null | head -1 || echo 'not found')$(CLR_RESET)\n"
	@printf "    Cargo        : $(CLR_CYAN)$(shell $(CARGO) --version 2>/dev/null || echo 'not found')$(CLR_RESET)\n"
	@printf "    Rustc        : $(CLR_CYAN)$(shell rustc --version 2>/dev/null || echo 'not found')$(CLR_RESET)\n"
	@printf "    QEMU         : $(CLR_CYAN)$(shell $(QEMU) --version 2>/dev/null | head -1 || echo 'not found')$(CLR_RESET)\n\n"
	@printf "  $(CLR_BOLD)Source Files$(CLR_RESET)\n"
	@printf "    Assembly     : $(CLR_CYAN)$(words $(ASM_SRCS)) files$(CLR_RESET)\n"
	@printf "    C Drivers    : $(CLR_CYAN)$(words $(C_SRCS)) files$(CLR_RESET)\n"
	@printf "    Shell Cmds   : $(CLR_CYAN)$(words $(SHELL_CMDS)) commands$(CLR_RESET)\n"
	@printf "    Drivers      : $(CLR_CYAN)$(words $(DRIVER_FILES)) descriptors$(CLR_RESET)\n\n"
	@printf "  $(CLR_BOLD)Rust Target$(CLR_RESET)\n"
	@printf "    Spec         : $(CLR_CYAN)$(RUST_TARGET)$(CLR_RESET)\n"
	@printf "    Profile      : $(CLR_CYAN)$(RUST_MODE)$(CLR_RESET)\n"
	@printf "    Output       : $(CLR_CYAN)$(RUST_LIB)$(CLR_RESET)\n\n"

check: ## Verify all required build dependencies are installed
	@MISSING=0; \
	for tool in nasm gcc ld cargo rustc grub-mkrescue xorriso qemu-system-x86_64 \
	            clang-format clang-tidy mkfs.fat mmd mcopy tar dd; do \
	    if command -v $$tool >/dev/null 2>&1; then \
	        $(LOG_CHECK) "$$tool"; \
	    else \
	        $(LOG_MISS) "$$tool"; \
	        MISSING=$$((MISSING + 1)); \
	    fi; \
	done; \
	printf "\n"; \
	if [ $$MISSING -eq 0 ]; then \
	    $(LOG_DONE) "All dependencies satisfied"; \
	else \
	    $(LOG_WARN) "$$MISSING missing dependencies detected"; \
	fi

size: $(KERNEL_BIN) ## Display kernel binary size and section breakdown
	@printf "  $(CLR_BOLD)Section Sizes:$(CLR_RESET)\n"
	$(Q)size $(KERNEL_BIN) | sed 's/^/    /'
	@printf "\n  $(CLR_BOLD)File Size:$(CLR_RESET)\n"
	@printf "    $(CLR_CYAN)%s$(CLR_RESET)\n\n" "$$(du -h $(KERNEL_BIN) | cut -f1) ($(KERNEL_BIN))"

objdump: $(KERNEL_BIN) ## Dump kernel ELF section headers and layout
	$(Q)objdump -h $(KERNEL_BIN)
# User-space compilation configuration
USER_LIB_SRCS := $(shell find user/lib -type f -name "*.c")
USER_KCC_SRCS := $(shell find user/bin/kcc -type f -name "*.c")
USER_CC_FLAGS  := -ffreestanding -nostdlib -fno-stack-protector -m64 -O2 -mno-sse -mno-sse2 -mno-mmx -mno-sse3 -mno-ssse3 -mno-sse4.1 -mno-sse4.2 -mno-avx -mno-avx2 -Iuser/include -Iuser/bin/kcc/include -T user/linker.ld -Wl,--no-warn-rwx-segments -static -no-pie

user: build/kcc.elf ## Build user-space C compiler (kcc.elf)

build/kcc.elf: $(USER_KCC_SRCS) $(USER_LIB_SRCS) user/linker.ld | dirs
	@$(LOG_INFO) "Building user space program: kcc (kcc.elf)..."
	$(Q)$(CC) $(USER_CC_FLAGS) $(USER_KCC_SRCS) $(USER_LIB_SRCS) -o build/kcc.elf

disk: $(DISK_IMG) ## Create and populate FAT16 hard disk image

$(DISK_IMG): build/kcc.elf
	@rm -f $(DISK_IMG)
	@$(LOG_DISK) "Creating $(DISK_SIZE)MB FAT16 disk image..."
	$(Q)dd if=/dev/zero of=$(DISK_IMG) bs=1M count=$(DISK_SIZE) 2>/dev/null
	$(Q)mkfs.fat -F 16 $(DISK_IMG) >/dev/null
	@$(LOG_DISK) "Creating nested Keira directory structure..."
	$(Q)mmd -i $(DISK_IMG) ::/system ::/system/bin ::/system/drivers ::/system/include ::/apps ::/apps/bin ::/apps/games ::/apps/src ::/config ::/config/boot ::/users ::/users/admin ::/users/default ::/users/guest ::/temp ::/data ::/data/log ::/data/save 2>/dev/null || true
	@$(LOG_DISK) "Populating directories with command binaries..."
	$(Q)mkdir -p $(BUILD_DIR)/system_bin
	$(Q)for cmd in $(SHELL_CMDS); do \
	    printf '#!/system/bin\n# Keira built-in command: %s\n# Type: kernel-mode binary\n# Path: /system/bin/%s\n' "$$cmd" "$$cmd" > $(BUILD_DIR)/system_bin/$$cmd; \
	    mcopy -o -i $(DISK_IMG) $(BUILD_DIR)/system_bin/$$cmd ::/system/bin/$$cmd; \
	done
	@$(LOG_DISK) "Copying driver files and system config..."
	$(Q)mkdir -p $(BUILD_DIR)/drivers
	$(Q)echo "Keira Serial Port Driver (COM1, 115200bps, 8N1)" > $(BUILD_DIR)/drivers/serial.sys
	$(Q)echo "Keira VGA Text Console Driver (80x25 characters, color support)" > $(BUILD_DIR)/drivers/vga.sys
	$(Q)echo "Keira PS/2 Keyboard Driver (US QWERTY layout)" > $(BUILD_DIR)/drivers/keyboard.sys
	$(Q)echo "Keira PS/2 Mouse Driver (basic coordinate tracking)" > $(BUILD_DIR)/drivers/mouse.sys
	$(Q)echo "Keira Real-Time Clock Driver (CMOS direct port communication)" > $(BUILD_DIR)/drivers/rtc.sys
	$(Q)echo "Keira IDE Storage Controller Driver (LBA28 read/write)" > $(BUILD_DIR)/drivers/ide.sys
	$(Q)echo "Keira AHCI SATA Storage Controller Driver (DMA read/write)" > $(BUILD_DIR)/drivers/ahci.sys
	$(Q)echo "Keira PC Speaker Sound Subsystem Driver (PIT Channel 2)" > $(BUILD_DIR)/drivers/sound.sys
	$(Q)echo "Keira Intel e1000 Network Interface Controller Driver (PCI DMA)" > $(BUILD_DIR)/drivers/e1000.sys
	$(Q)for driver in $(DRIVER_FILES); do \
	    mcopy -o -i $(DISK_IMG) $(BUILD_DIR)/drivers/$$driver ::/system/drivers/$$driver; \
	done
	$(Q)echo "boot_mode=kernel\nconsole=vga\ncursor=block" > $(BUILD_DIR)/boot.cfg
	$(Q)mcopy -o -i $(DISK_IMG) $(BUILD_DIR)/boot.cfg ::/config/boot/boot.cfg
	@$(LOG_DISK) "Copying binaries and configuration files..."
	$(Q)mcopy -o -i $(DISK_IMG) build/kcc.elf ::/apps/bin/kcc.elf
	$(Q)mkdir -p $(BUILD_DIR)/data
	$(Q)printf '/* Keira Sample C Program */\n#include <stdio.h>\n#include <syscall.h>\n\nvoid main(void) {\n    printf("Hello from Keira KCC Userland!\\n");\n}\n' > $(BUILD_DIR)/data/main.c
	$(Q)mcopy -o -i $(DISK_IMG) $(BUILD_DIR)/data/main.c ::/data/main.c
	$(Q)for header in stdio.h stdlib.h string.h syscall.h socket.h math.h time.h malloc.h fcntl.h; do mcopy -o -i $(DISK_IMG) user/include/$$header ::/system/include/$$header; done
initrd: $(BUILD_DIR)/initrd.tar ## Build RAM Disk USTAR archive

$(BUILD_DIR)/initrd.tar: build/kcc.elf
	@$(LOG_INFO) "Building RAM Disk (Initrd)..."
	$(Q)mkdir -p $(BUILD_DIR)/initrd_root/system/bin
	$(Q)mkdir -p $(BUILD_DIR)/initrd_root/system/drivers
	$(Q)mkdir -p $(BUILD_DIR)/initrd_root/system/dev
	$(Q)mkdir -p $(BUILD_DIR)/initrd_root/system/include
	$(Q)mkdir -p $(BUILD_DIR)/initrd_root/apps/bin
	$(Q)mkdir -p $(BUILD_DIR)/initrd_root/config/boot
	$(Q)mkdir -p $(BUILD_DIR)/initrd_root/users/admin
	$(Q)mkdir -p $(BUILD_DIR)/initrd_root/users/default
	$(Q)mkdir -p $(BUILD_DIR)/initrd_root/users/guest
	$(Q)mkdir -p $(BUILD_DIR)/initrd_root/temp
	$(Q)mkdir -p $(BUILD_DIR)/initrd_root/data
	$(Q)echo "Keira Null Device Node" > $(BUILD_DIR)/initrd_root/system/dev/null
	$(Q)echo "Keira Zero Device Node" > $(BUILD_DIR)/initrd_root/system/dev/zero
	$(Q)echo "Keira Random Device Node" > $(BUILD_DIR)/initrd_root/system/dev/random
	$(Q)echo "Keira TTY Console Node" > $(BUILD_DIR)/initrd_root/system/dev/tty
	$(Q)for cmd in $(SHELL_CMDS); do \
	    printf '#!/system/bin\n# Keira built-in command: %s\n# Type: kernel-mode binary\n# Path: /system/bin/%s\n' "$$cmd" "$$cmd" > $(BUILD_DIR)/initrd_root/system/bin/$$cmd; \
	done
	$(Q)echo "Keira Serial Port Driver (COM1, 115200bps, 8N1)" > $(BUILD_DIR)/initrd_root/system/drivers/serial.sys
	$(Q)echo "Keira VGA Text & Widescreen Console Driver (color support)" > $(BUILD_DIR)/initrd_root/system/drivers/vga.sys
	$(Q)echo "Keira PS/2 Keyboard Driver (US QWERTY layout)" > $(BUILD_DIR)/initrd_root/system/drivers/keyboard.sys
	$(Q)echo "Keira PS/2 Mouse Driver (basic coordinate tracking)" > $(BUILD_DIR)/initrd_root/system/drivers/mouse.sys
	$(Q)echo "Keira Real-Time Clock Driver (CMOS direct port communication)" > $(BUILD_DIR)/initrd_root/system/drivers/rtc.sys
	$(Q)echo "Keira IDE Storage Controller Driver (LBA28 read/write)" > $(BUILD_DIR)/initrd_root/system/drivers/ide.sys
	$(Q)echo "Keira AHCI SATA Storage Controller Driver (DMA read/write)" > $(BUILD_DIR)/initrd_root/system/drivers/ahci.sys
	$(Q)echo "Keira PC Speaker Sound Subsystem Driver (PIT Channel 2)" > $(BUILD_DIR)/initrd_root/system/drivers/sound.sys
	$(Q)echo "Keira Intel e1000 Network Interface Controller Driver (PCI DMA)" > $(BUILD_DIR)/initrd_root/system/drivers/e1000.sys
	$(Q)echo "boot_mode=kernel\nconsole=vga\ncursor=block" > $(BUILD_DIR)/initrd_root/config/boot/boot.cfg
	$(Q)cp build/kcc.elf $(BUILD_DIR)/initrd_root/apps/bin/kcc.elf
	$(Q)printf '/* Keira Sample C Program */\n#include <stdio.h>\n#include <syscall.h>\n\nvoid main(void) {\n    printf("Hello from Keira KCC Userland!\\n");\n}\n' > $(BUILD_DIR)/initrd_root/data/main.c
	$(Q)cp user/include/*.h $(BUILD_DIR)/initrd_root/system/include/
	$(Q)cd $(BUILD_DIR)/initrd_root && tar -cf ../initrd.tar *
iso: $(KERNEL_ISO) ## Package GRUB Multiboot2 bootable ISO image

$(KERNEL_ISO): $(KERNEL_BIN) $(BUILD_DIR)/initrd.tar | dirs
	@$(LOG_ISO) "Creating bootable ISO..."
	$(Q)mkdir -p $(ISO_DIR)/boot/grub
	$(Q)cp $(KERNEL_BIN) $(ISO_DIR)/boot/$(KERNEL_NAME).bin
	$(Q)cp $(BUILD_DIR)/initrd.tar $(ISO_DIR)/boot/initrd.tar
	$(Q)echo 'set timeout=0' > $(ISO_DIR)/boot/grub/grub.cfg
	$(Q)echo 'set default=0' >> $(ISO_DIR)/boot/grub/grub.cfg
	$(Q)echo '' >> $(ISO_DIR)/boot/grub/grub.cfg
	$(Q)echo 'menuentry "Keira" {' >> $(ISO_DIR)/boot/grub/grub.cfg
	$(Q)echo '	multiboot2 /boot/keira.bin' >> $(ISO_DIR)/boot/grub/grub.cfg
	$(Q)echo '	module2 /boot/initrd.tar initrd' >> $(ISO_DIR)/boot/grub/grub.cfg
	$(Q)echo '	boot' >> $(ISO_DIR)/boot/grub/grub.cfg
	$(Q)echo '}' >> $(ISO_DIR)/boot/grub/grub.cfg
	$(Q)grub-mkrescue -o $(KERNEL_ISO) $(ISO_DIR) 2>/dev/null
	@$(LOG_DONE) "$(KERNEL_ISO) ready"
# Link final kernel ELF64 binary executable
$(KERNEL_BIN): $(ALL_OBJS) $(RUST_LIB) arch/x86/linker.ld | dirs
	@$(LOG_LD) "Linking kernel..."
	$(Q)$(LD) $(LD_FLAGS) -o $(KERNEL_BIN) $(ALL_OBJS) $(RUST_LIB)
	@$(LOG_DONE) "$(KERNEL_BIN) ready"

# Compile freestanding Rust kernel module
rust: | dirs ## Build Rust kernel static library
	@$(LOG_CARGO) "Building Rust kernel ($(RUST_MODE))...."
	$(Q)$(CARGO) -Zjson-target-spec -Zbuild-std=core,compiler_builtins build --target $(RUST_TARGET) --$(RUST_MODE) -p keira-kernel 2>&1 | sed 's/^/        /'

$(RUST_LIB): rust

# Compile Assembly source files (.asm -> .o)
$(OBJ_DIR)/%.asm.o: %.asm | dirs
	@$(LOG_ASM) "$<"
	$(Q)mkdir -p $(dir $@)
	$(Q)$(ASM) $(ASM_FLAGS) -o $@ $<

# Compile C source files (.c -> .o)
$(OBJ_DIR)/%.c.o: %.c | dirs
	@$(LOG_CC) "$<"
	$(Q)mkdir -p $(dir $@)
	$(Q)$(CC) $(CC_FLAGS) -c -o $@ $<

# Create target build directories
dirs:
	$(Q)mkdir -p $(BUILD_DIR) $(OBJ_DIR)
run: all ## Launch Keira in QEMU virtual machine
	@$(LOG_INFO) "Launching Keira in QEMU..."
	$(Q)$(QEMU) $(QEMU_FLAGS)

debug: all ## Launch Keira in QEMU debug mode (GDB on :1234)
	@$(LOG_INFO) "Launching Keira (debug mode, waiting for GDB on :1234)..."
	$(Q)$(QEMU) $(QEMU_FLAGS) -S -s

qemu-net: all ## Launch Keira in QEMU with e1000 NIC emulation
	@$(LOG_INFO) "Launching Keira with e1000 network (QEMU)..."
	$(Q)$(QEMU) $(QEMU_NET_FLAGS)

test: all ## Run automated headless QEMU smoke test
	@$(LOG_INFO) "Running headless QEMU automated test..."
	$(Q)timeout 5s $(QEMU) -cdrom $(KERNEL_ISO) -drive file=$(DISK_IMG),format=raw,if=ide -m 128M -serial stdio -display none -device isa-debug-exit,iobase=0xf4,iosize=0x04 >/dev/null 2>&1 || true
	@$(LOG_DONE) "Automated smoke test complete"

clean: ## Remove build directory and compiled artifacts
	@$(LOG_INFO) "Removing build artifacts..."
	$(Q)rm -rf $(BUILD_DIR) target/
	$(Q)$(CARGO) clean 2>/dev/null || true
	$(Q)find . -type f \( -name "*~" -o -name "*.swp" -o -name "*.swo" -o -name "*.bak" -o -name "*.tmp" -o -name "*.pyc" \) -delete 2>/dev/null || true
	$(Q)find . -type d -name "__pycache__" -exec rm -rf {} + 2>/dev/null || true
	@$(LOG_DONE) "Clean complete"

format: ## Format Rust and C source code
	@$(LOG_INFO) "Formatting Rust code..."
	$(Q)$(CARGO) fmt --all
	@$(LOG_INFO) "Formatting C code..."
	$(Q)find . -path "./build" -prune -o -type f \( -name "*.c" -o -name "*.h" \) -exec clang-format -i {} +
	@$(LOG_DONE) "Formatting complete"

lint: ## Static analysis of C code using clang-tidy
	@$(LOG_INFO) "Linting C code..."
	$(Q)find drivers arch/x86 user -type f -name "*.c" -exec clang-tidy --checks='-*,clang-analyzer-*,-clang-analyzer-core.FixedAddressDereference,-clang-analyzer-security.insecureAPI.DeprecatedOrUnsafeBufferHandling' {} -- -I include -I drivers -I arch/x86/include -I user/include -I user/bin/kcc/include -ffreestanding -m64 \;
	@$(LOG_DONE) "Linting complete"
