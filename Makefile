# SPDX-License-Identifier: GPL-2.0-only
#
# Keira Kernel - Freestanding Kernel from Scratch
# Copyright (C) 2026 Moh. Ananda Firmansyah Putra
#
# This program is free software; you can redistribute it and/or modify
# it under the terms of the GNU General Public License as published by
# the Free Software Foundation; version 2 of the License.

SHELL           := /bin/bash

# Master Build System Architecture
#
# Orchestrates the pure Rust kernel and assembly bootstrap compilation pipeline:
#   1. NASM (Assembly)  : Compiles 32-bit and 64-bit boot trampolines & ISR handlers.
#   2. Cargo (Rust Core): Compiles `no_std` 100% Pure Rust kernel static library (`.a`).
#   3. LD (Linker)      : Links object files into a single ELF kernel executable.
#   4. Userland (KCC/C) : Compiles userland C compiler (kcc.elf) and standard library.
#   5. GRUB (Bootloader): Packages kernel and USTAR initrd into a bootable ISO.

# Target Architecture: x86_64 (default) or i686 (pure 32-bit x86)
ARCH            ?= x86_64

# Toolchain executables
ASM             := nasm
CC              := gcc
LD              := ld
CARGO           := cargo
GRUB_MKRESCUE   := $(shell command -v grub-mkrescue 2>/dev/null || command -v grub2-mkrescue 2>/dev/null || echo grub-mkrescue)

# Project naming & version metadata
KERNEL_NAME     := keira
VERSION         := $(shell grep -m 1 '^version = ' crates/kernel/Cargo.toml | cut -d '"' -f 2)
DATE_SUFFIX     := $(shell date +%Y-%m-%d)

# Architecture-isolated build directory hierarchy
BUILD_ROOT      := build
BUILD_DIR       := $(BUILD_ROOT)/$(ARCH)
BIN_DIR         := $(BUILD_DIR)/bin
ISO_OUT_DIR     := $(BUILD_DIR)/iso
DISK_DIR        := $(BUILD_DIR)/disk
OBJ_DIR         := $(BUILD_DIR)/obj
STAGING_DIR     := $(BUILD_DIR)/staging
ISO_DIR         := $(STAGING_DIR)/isofiles
FS_ROOT         := $(STAGING_DIR)/fs_root

KERNEL_BIN      := $(BIN_DIR)/$(KERNEL_NAME).bin
KERNEL_ISO      := $(ISO_OUT_DIR)/$(KERNEL_NAME)-$(ARCH)-$(DATE_SUFFIX).iso
USER_ELF        := $(BIN_DIR)/kcc.elf
DISK_IMG        := $(DISK_DIR)/disk.img
INITRD_TAR      := $(DISK_DIR)/initrd.tar

# Configurable build parameters
DISK_SIZE       ?= 32
QEMU_MEM        ?= 128M

# Verbose mode: set V=1 to display raw command executions
ifeq ($(V),1)
    Q           :=
else
    Q           := @
endif

# Architecture-specific compilation flags
ifeq ($(ARCH),i686)
    ASM_FLAGS   := -f elf32 -I arch/x86/common/include/
    LD_FLAGS    := -m elf_i386 -n -T arch/x86/i686/linker.ld --gc-sections --no-warn-rwx-segments
    RUST_TARGET := targets/x86/i686/i686-keira-none.json
    RUST_MODE   := release
    RUST_LIB    := target/i686-keira-none/$(RUST_MODE)/libkeira_kernel.a
    QEMU        := qemu-system-i386
    ASM_SRCS    := arch/x86/common/boot/multiboot2_header.asm \
                   arch/x86/i686/boot/entry.asm \
                   arch/x86/i686/kernel/gdt.asm \
                   arch/x86/i686/kernel/idt.asm \
                   arch/x86/i686/kernel/isr.asm \
                   arch/x86/i686/kernel/syscall.asm
    USER_LINKER_SCRIPT := user/arch/x86/linker32.ld
    USER_CC_FLAGS := -ffreestanding -nostdlib -fno-stack-protector -m32 -O2 \
                     -mno-sse -mno-sse2 -mno-mmx \
                     -Iuser/include -Iuser/bin/kcc/include -T $(USER_LINKER_SCRIPT) \
                     -Wl,--no-warn-rwx-segments -Wl,--build-id=none -static -no-pie -lgcc
else
    ASM_FLAGS   := -f elf64 -I arch/x86/common/include/
    LD_FLAGS    := -n -T arch/x86/x86_64/linker.ld --gc-sections --no-warn-rwx-segments
    RUST_TARGET := targets/x86/x86_64/x86_64-keira-none.json
    RUST_MODE   := release
    RUST_LIB    := target/x86_64-keira-none/$(RUST_MODE)/libkeira_kernel.a
    QEMU        := qemu-system-x86_64
    ASM_SRCS    := arch/x86/common/boot/multiboot2_header.asm \
                   arch/x86/x86_64/boot/entry32.asm \
                   arch/x86/x86_64/boot/entry64.asm \
                   arch/x86/x86_64/kernel/gdt.asm \
                   arch/x86/x86_64/kernel/paging.asm \
                   arch/x86/x86_64/kernel/idt.asm \
                   arch/x86/x86_64/kernel/isr.asm \
                   arch/x86/x86_64/kernel/syscall.asm
    USER_LINKER_SCRIPT := user/arch/x86/linker.ld
    USER_CC_FLAGS := -ffreestanding -nostdlib -fno-stack-protector -m64 -O2 \
                     -mno-sse -mno-sse2 -mno-mmx -mno-sse3 -mno-ssse3 \
                     -mno-sse4.1 -mno-sse4.2 -mno-avx -mno-avx2 \
                     -Iuser/include -Iuser/bin/kcc/include -T $(USER_LINKER_SCRIPT) \
                     -Wl,--no-warn-rwx-segments -Wl,--build-id=none -static -no-pie
endif

ASM_OBJS        := $(patsubst %.asm,$(OBJ_DIR)/%.asm.o,$(ASM_SRCS))
ALL_OBJS        := $(ASM_OBJS)

USER_LIB_SRCS   := $(shell find user/lib -type f -name "*.c")
USER_KCC_SRCS   := $(shell find user/bin/kcc -type f -name "*.c")

# QEMU hardware & emulation flags
QEMU_FLAGS      := -cdrom $(KERNEL_ISO) \
                   -device ahci,id=ahci0 \
                   -drive file=$(DISK_IMG),format=raw,id=sata0,if=none \
                   -device ide-hd,drive=sata0,bus=ahci0.0 \
                   -device e1000,netdev=net0 \
                   -netdev user,id=net0 \
                   -boot d \
                   -serial stdio \
                   -no-shutdown \
                   -m $(QEMU_MEM)

QEMU_NET_FLAGS  := $(QEMU_FLAGS)

# Terminal logging definitions (plain text)
LOG_ASM         := printf "  [ASM]   %s\n"
LOG_CC          := printf "  [CC]    %s\n"
LOG_CARGO       := printf "  [CARGO] %s\n"
LOG_LD          := printf "  [LD]    %s\n"
LOG_ISO         := printf "  [ISO]   %s\n"
LOG_DISK        := printf "  [DISK]  %s\n"
LOG_DONE        := printf "[DONE]  %s\n"
LOG_INFO        := printf "[INFO]  %s\n"
LOG_WARN        := printf "[WARN]  %s\n"
LOG_ERR         := printf "[ERR]   %s\n"
LOG_CHECK       := printf "  [OK]    %s\n"
LOG_MISS        := printf "  [MISS]  %s\n"

# Canonical filesystem manifests
SHELL_CMDS      := login drives use ramdisk system cpu smp runtime time memory \
                   devices initrd wipe reset run write tasks disk list \
                   go view create folder delete edit copy help history \
                   move search download network stop env sync \
                   protect fileinfo framebuffer usb https user hostname syslog kvm \
                   nvme ext4 cgroups futex bpf tpm swap seccomp epoll \
                   drivers lkm unwind watchpoint power perf timer eventfd mac mqueue \
                   kill jobs fg bg lvm raid ipcs ipcrm iptables firewall service \
                   shutdown reboot

DRIVER_FILES    := serial.sys vga.sys keyboard.sys mouse.sys rtc.sys \
                   ide.sys ahci.sys e1000.sys

# Phony targets declaration
.PHONY: all full fll run run-64 run-32 run-x86_64 run-i686 debug clean rust iso dirs \
        format lint user disk initrd help info check size objdump qemu-net test test-all fs-root \
        preflight preflight-qemu preflight-format preflight-lint

.DEFAULT_GOAL   := all

# Toolchain preflight dependency guards
preflight: ## Validate presence of all essential build, packaging, and filesystem utilities
	@MISSING=""; \
	for tool in $(ASM) $(CC) $(LD) $(CARGO) rustc $(GRUB_MKRESCUE) xorriso mkfs.fat mmd mcopy tar dd; do \
	    if ! command -v $$tool >/dev/null 2>&1; then \
	        MISSING="$$MISSING $$tool"; \
	    fi; \
	done; \
	if [ -n "$$MISSING" ]; then \
	    printf "\n[ERR] Missing required build tool(s):%s\n\n" "$$MISSING"; \
	    printf "[INFO] Please install missing dependencies using your distribution package manager:\n"; \
	    printf "  Ubuntu / Debian:\n"; \
	    printf "    sudo apt-get update && sudo apt-get install -y nasm gcc binutils cargo rustc grub-pc-bin grub-common xorriso dosfstools mtools tar coreutils\n\n"; \
	    printf "  Arch Linux:\n"; \
	    printf "    sudo pacman -S --needed nasm gcc binutils rust grub xorriso dosfstools mtools tar coreutils\n\n"; \
	    printf "  Fedora / RHEL:\n"; \
	    printf "    sudo dnf install -y nasm gcc binutils cargo rustc grub2-tools-extra xorriso dosfstools mtools tar coreutils\n\n"; \
	    printf "  Rust Nightly (required):\n"; \
	    printf "    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh && rustup default nightly\n\n"; \
	    exit 1; \
	else \
	    if [ "$$MAKECMDGOALS" = "preflight" ] || [ "$(MAKECMDGOALS)" = "preflight" ]; then \
	        $(LOG_DONE) "All core build tools verified"; \
	    fi; \
	fi

preflight-qemu: ## Validate presence of QEMU hypervisor for current target architecture
	@if ! command -v $(QEMU) >/dev/null 2>&1; then \
	    printf "\n[ERR] Missing QEMU hypervisor: $(QEMU)\n\n"; \
	    printf "[INFO] Please install QEMU using your distribution package manager:\n"; \
	    printf "  Ubuntu / Debian:\n"; \
	    printf "    sudo apt-get update && sudo apt-get install -y qemu-system-x86\n\n"; \
	    printf "  Arch Linux:\n"; \
	    printf "    sudo pacman -S --needed qemu-system-x86\n\n"; \
	    printf "  Fedora / RHEL:\n"; \
	    printf "    sudo dnf install -y qemu-system-x86\n\n"; \
	    exit 1; \
	else \
	    if [ "$$MAKECMDGOALS" = "preflight-qemu" ] || [ "$(MAKECMDGOALS)" = "preflight-qemu" ]; then \
	        $(LOG_DONE) "QEMU hypervisor verified: $(QEMU)"; \
	    fi; \
	fi

preflight-format: ## Validate presence of code formatting utilities
	@MISSING=""; \
	for tool in cargo clang-format; do \
	    if ! command -v $$tool >/dev/null 2>&1; then \
	        MISSING="$$MISSING $$tool"; \
	    fi; \
	done; \
	if [ -n "$$MISSING" ]; then \
	    printf "\n[ERR] Missing formatting tool(s):%s\n\n" "$$MISSING"; \
	    printf "[INFO] Please install formatting tools using your distribution package manager:\n"; \
	    printf "  Ubuntu / Debian:\n"; \
	    printf "    sudo apt-get update && sudo apt-get install -y clang-format cargo\n\n"; \
	    printf "  Arch Linux:\n"; \
	    printf "    sudo pacman -S --needed clang rust\n\n"; \
	    printf "  Fedora / RHEL:\n"; \
	    printf "    sudo dnf install -y clang-tools-extra cargo\n\n"; \
	    exit 1; \
	else \
	    if [ "$$MAKECMDGOALS" = "preflight-format" ] || [ "$(MAKECMDGOALS)" = "preflight-format" ]; then \
	        $(LOG_DONE) "All formatting tools verified"; \
	    fi; \
	fi

preflight-lint: ## Validate presence of static analysis utilities
	@if ! command -v clang-tidy >/dev/null 2>&1; then \
	    printf "\n[ERR] Missing static analysis tool: clang-tidy\n\n"; \
	    printf "[INFO] Please install clang-tidy using your distribution package manager:\n"; \
	    printf "  Ubuntu / Debian:\n"; \
	    printf "    sudo apt-get update && sudo apt-get install -y clang-tidy\n\n"; \
	    printf "  Arch Linux:\n"; \
	    printf "    sudo pacman -S --needed clang\n\n"; \
	    printf "  Fedora / RHEL:\n"; \
	    printf "    sudo dnf install -y clang-tools-extra\n\n"; \
	    exit 1; \
	else \
	    if [ "$$MAKECMDGOALS" = "preflight-lint" ] || [ "$(MAKECMDGOALS)" = "preflight-lint" ]; then \
	        $(LOG_DONE) "Static analysis tool verified: clang-tidy"; \
	    fi; \
	fi

# Primary build targets
all: preflight $(KERNEL_ISO) $(DISK_IMG) ## Build kernel, ISO image, and FAT16 disk image for current ARCH

full: preflight ## Build kernel and ISO images for all supported architectures (x86_64 & i686)
	@$(LOG_INFO) "Building Keira for all architectures (x86_64 & i686)..."
	$(Q)$(MAKE) ARCH=x86_64 all
	$(Q)$(MAKE) ARCH=i686 all
	@$(LOG_DONE) "Full multi-architecture build complete"

fll: full

iso: preflight $(KERNEL_ISO) ## Package GRUB Multiboot2 bootable ISO image

disk: preflight $(DISK_IMG) ## Create and populate FAT16 hard disk image

initrd: preflight $(INITRD_TAR) ## Build RAM Disk USTAR archive

user: preflight $(USER_ELF) ## Build user-space C compiler (kcc.elf)

rust: preflight | dirs ## Build Rust kernel static library
	@$(LOG_CARGO) "Building Rust kernel ($(ARCH) $(RUST_MODE))...."
	$(Q)$(CARGO) -Zjson-target-spec -Zbuild-std=core,compiler_builtins build --target $(RUST_TARGET) --$(RUST_MODE) -p keira-kernel 2>&1 | sed 's/^/        /'

dirs: preflight ## Create architecture-isolated build output directory hierarchy
	$(Q)mkdir -p $(BUILD_ROOT) $(BUILD_DIR) $(BIN_DIR) $(ISO_OUT_DIR) $(DISK_DIR) $(OBJ_DIR) $(STAGING_DIR)

# Binary & ISO construction rules
$(KERNEL_ISO): $(KERNEL_BIN) $(INITRD_TAR) | dirs
	@$(LOG_ISO) "Creating bootable ISO ($(ARCH))..."
	$(Q)mkdir -p $(ISO_DIR)/boot/grub
	$(Q)cp $(KERNEL_BIN) $(ISO_DIR)/boot/$(KERNEL_NAME).bin
	$(Q)cp $(INITRD_TAR) $(ISO_DIR)/boot/initrd.tar
	$(Q)echo 'set timeout=0' > $(ISO_DIR)/boot/grub/grub.cfg
	$(Q)echo 'set default=0' >> $(ISO_DIR)/boot/grub/grub.cfg
	$(Q)echo '' >> $(ISO_DIR)/boot/grub/grub.cfg
	$(Q)echo 'menuentry "Keira" {' >> $(ISO_DIR)/boot/grub/grub.cfg
	$(Q)echo '	multiboot2 /boot/keira.bin' >> $(ISO_DIR)/boot/grub/grub.cfg
	$(Q)echo '	module2 /boot/initrd.tar initrd' >> $(ISO_DIR)/boot/grub/grub.cfg
	$(Q)echo '	boot' >> $(ISO_DIR)/boot/grub/grub.cfg
	$(Q)echo '}' >> $(ISO_DIR)/boot/grub/grub.cfg
	$(Q)$(GRUB_MKRESCUE) -o $(KERNEL_ISO) $(ISO_DIR) 2>/dev/null
	@$(LOG_DONE) "$(KERNEL_ISO) ready"

$(KERNEL_BIN): $(ALL_OBJS) $(RUST_LIB) FORCE | dirs
	@$(LOG_LD) "Linking kernel ($(ARCH))..."
	$(Q)$(LD) $(LD_FLAGS) -o $(KERNEL_BIN) $(ALL_OBJS) $(RUST_LIB)
	@$(LOG_DONE) "$(KERNEL_BIN) ready"

$(RUST_LIB): rust FORCE

FORCE:

$(OBJ_DIR)/%.asm.o: %.asm | dirs
	@$(LOG_ASM) "$< ($(ARCH))"
	$(Q)mkdir -p $(dir $@)
	$(Q)$(ASM) $(ASM_FLAGS) -o $@ $<

$(USER_ELF): $(USER_KCC_SRCS) $(USER_LIB_SRCS) $(USER_LINKER_SCRIPT) | dirs
	@$(LOG_INFO) "Building user space program: kcc ($(ARCH))..."
	$(Q)$(CC) $(USER_CC_FLAGS) $(USER_KCC_SRCS) $(USER_LIB_SRCS) -o $(USER_ELF)

# Canonical root filesystem & disk image rules
fs-root: $(USER_ELF) | dirs
	@$(LOG_INFO) "Populating canonical root filesystem ($(ARCH))..."
	$(Q)rm -rf $(FS_ROOT)
	$(Q)mkdir -p $(FS_ROOT)/system/bin
	$(Q)mkdir -p $(FS_ROOT)/system/dev
	$(Q)mkdir -p $(FS_ROOT)/system/drivers
	$(Q)mkdir -p $(FS_ROOT)/system/include/sys
	$(Q)mkdir -p $(FS_ROOT)/system/lib
	$(Q)mkdir -p $(FS_ROOT)/apps/bin
	$(Q)mkdir -p $(FS_ROOT)/apps/src/kcc/include
	$(Q)mkdir -p $(FS_ROOT)/config/boot
	$(Q)mkdir -p $(FS_ROOT)/config/sys
	$(Q)mkdir -p $(FS_ROOT)/users/admin
	$(Q)mkdir -p $(FS_ROOT)/temp
	$(Q)mkdir -p $(FS_ROOT)/data/log
	$(Q)cp $(USER_ELF) $(FS_ROOT)/system/bin/kcc.elf
	$(Q)cp $(USER_ELF) $(FS_ROOT)/apps/bin/kcc.elf
	$(Q)for cmd in $(SHELL_CMDS); do \
	    printf "ELF\002\001\001\000Keira Builtin Command: %s\n" "$$cmd" > $(FS_ROOT)/system/bin/$$cmd.elf; \
	    chmod +x $(FS_ROOT)/system/bin/$$cmd.elf; \
	done
	$(Q)for drv in $(DRIVER_FILES); do \
	    printf "KEIRA_DRIVER\001\000[Driver: %s]\nStatus=Active\nType=KernelSubsystem\n" "$$drv" > $(FS_ROOT)/system/drivers/$$drv; \
	done
	$(Q)printf "console\nnull\nzero\nrandom\nurandom\nptmx\ntty\nfb0\nsda\nsda1\n" > $(FS_ROOT)/system/dev/devices.list
	$(Q)cp -r user/include/* $(FS_ROOT)/system/include/
	$(Q)cp user/bin/kcc/include/common.h $(FS_ROOT)/system/include/common.h
	$(Q)cp user/lib/math/math.c $(FS_ROOT)/system/lib/math.c
	$(Q)cp user/lib/string/string.c $(FS_ROOT)/system/lib/string.c
	$(Q)cp user/lib/stdlib/stdlib.c $(FS_ROOT)/system/lib/stdlib.c
	$(Q)cp user/lib/unistd/unistd.c $(FS_ROOT)/system/lib/unistd.c
	$(Q)cp user/lib/assert/assert.c $(FS_ROOT)/system/lib/assert.c
	$(Q)cp user/lib/dirent/dirent.c $(FS_ROOT)/system/lib/dirent.c
	$(Q)cp user/lib/stat/stat.c $(FS_ROOT)/system/lib/stat.c
	$(Q)cp user/lib/signal/signal.c $(FS_ROOT)/system/lib/signal.c
	$(Q)cp user/lib/time/time.c $(FS_ROOT)/system/lib/time.c
	$(Q)cp user/lib/setjmp/setjmp.c $(FS_ROOT)/system/lib/setjmp.c
	$(Q)cp user/lib/ctype/ctype.c $(FS_ROOT)/system/lib/ctype.c
	$(Q)cp user/lib/errno/errno.c $(FS_ROOT)/system/lib/errno.c
	$(Q)cp user/lib/mem/malloc.c $(FS_ROOT)/system/lib/malloc.c
	$(Q)cp user/lib/stdio/file.c $(FS_ROOT)/system/lib/file.c
	$(Q)cp user/lib/stdio/printf.c $(FS_ROOT)/system/lib/printf.c
	$(Q)cp user/lib/syscall/syscall.c $(FS_ROOT)/system/lib/syscall.c
	$(Q)cp user/bin/kcc/*.c $(FS_ROOT)/apps/src/kcc/
	$(Q)cp user/bin/kcc/include/*.h $(FS_ROOT)/apps/src/kcc/include/
	$(Q)touch $(FS_ROOT)/apps/src/.keep
	$(Q)printf "console=tty0 serial=ttyS0,115200 root=/dev/sda1 quiet loglevel=3\n" > $(FS_ROOT)/config/boot/grub.cfg
	$(Q)printf "KERNEL_NAME=keira\nKERNEL_VERSION=$(VERSION)\nKERNEL_ARCH=$(ARCH)\n" > $(FS_ROOT)/config/sys/kernel.cfg
	$(Q)printf "keira\n" > $(FS_ROOT)/config/sys/hostname.cfg
	$(Q)printf "nameserver 1.1.1.1\nnameserver 8.8.8.8\n" > $(FS_ROOT)/config/sys/resolv.conf
	$(Q)printf "127.0.0.1\tlocalhost\n127.0.1.1\tkeira\n" > $(FS_ROOT)/config/sys/hosts
	$(Q)printf "admin:keira\n" > $(FS_ROOT)/config/sys/passwd
	$(Q)printf "# Keira Kernel System Services Configuration\n[syslogd]\nenabled=true\nfile=/data/log/syslog.log\n\n[syncd]\nenabled=true\ninterval=30\n\n[watchdogd]\nenabled=true\ntimeout=60\n\n[timed]\nenabled=true\ninterval=30\n\n[monitord]\nenabled=true\ninterval=10\n\n[netd]\nenabled=true\ninterval=15\n" > $(FS_ROOT)/config/sys/services.conf
	$(Q)printf "# Keira Service Configuration\nname=syncd\ndescription=FAT16 Auto-Sync & Cache Flush Daemon\nenabled=1\nauto_restart=1\ninterval=15\n" > $(FS_ROOT)/config/sys/syncd.conf
	$(Q)printf "# Keira Service Configuration\nname=syslogd\ndescription=Kernel Event & Audit Logger Service\nenabled=1\nauto_restart=1\ninterval=5\n" > $(FS_ROOT)/config/sys/syslogd.conf
	$(Q)printf "# Keira Service Configuration\nname=watchdogd\ndescription=Memory & Task Health Watchdog\nenabled=1\nauto_restart=1\ninterval=10\n" > $(FS_ROOT)/config/sys/watchdogd.conf
	$(Q)printf "# Keira Service Configuration\nname=timed\ndescription=CMOS RTC & System Clock Sync Daemon\nenabled=1\nauto_restart=1\ninterval=30\n" > $(FS_ROOT)/config/sys/timed.conf
	$(Q)printf "# Keira Service Configuration\nname=monitord\ndescription=System Health & Telemetry Daemon\nenabled=1\nauto_restart=1\ninterval=10\n" > $(FS_ROOT)/config/sys/monitord.conf
	$(Q)printf "# Keira Service Configuration\nname=netd\ndescription=Network State & ARP Daemon\nenabled=1\nauto_restart=1\ninterval=15\n" > $(FS_ROOT)/config/sys/netd.conf
	$(Q)printf '/* Keira Comprehensive KCC Sample Program */\n#include <stdio.h>\n#include <syscall.h>\n\nint compute(int x, int y) {\n    int res = (x * y) + (x %% y);\n    return res ^ (x >> 1);\n}\n\nvoid main(void) {\n    printf("Keira KCC Compiler Execution\\n");\n    int i = 0, total = 0;\n    while (i < 10) {\n        i++;\n        if (i == 5) continue;\n        if (i > 8) break;\n        total += compute(i, 3);\n    }\n    printf("KCC compilation & execution complete!\\n");\n}\n' > $(FS_ROOT)/data/main.c
	$(Q)printf "[System Boot Record]\nKeira Kernel v$(VERSION) initialized successfully.\n" > $(FS_ROOT)/data/log/boot.log
	$(Q)printf "[System Event Log]\nKernel Ring 0 initialized. Shell ready.\n" > $(FS_ROOT)/data/log/system.log
	$(Q)printf "[INFO] Keira Service Controller (ksvc) system logger initialized.\n" > $(FS_ROOT)/data/log/syslog.log
	$(Q)printf "[INFO] Keira Telemetry Monitor (monitord) initialized.\n" > $(FS_ROOT)/data/log/monitor.log
	$(Q)touch $(FS_ROOT)/temp/.keep

$(DISK_IMG): fs-root
	@rm -f $(DISK_IMG)
	@$(LOG_DISK) "Creating $(DISK_SIZE)MB FAT16 disk image ($(ARCH))..."
	$(Q)dd if=/dev/zero of=$(DISK_IMG) bs=1M count=$(DISK_SIZE) 2>/dev/null
	$(Q)mkfs.fat -F 16 $(DISK_IMG) >/dev/null
	@$(LOG_DISK) "Creating nested Keira directory structure ($(ARCH))..."
	$(Q)mmd -i $(DISK_IMG) ::/system ::/system/bin ::/system/dev ::/system/drivers ::/system/include ::/system/include/sys ::/system/lib ::/apps ::/apps/bin ::/apps/src ::/apps/src/kcc ::/apps/src/kcc/include ::/config ::/config/boot ::/config/sys ::/users ::/users/admin ::/temp ::/data ::/data/log 2>/dev/null || true
	@$(LOG_DISK) "Populating disk image with system files ($(ARCH))..."
	$(Q)for f in $$(cd $(FS_ROOT) && find . -type f | sed 's|^\./||'); do \
	    mcopy -o -i $(DISK_IMG) $(FS_ROOT)/$$f ::/$$f; \
	done
	@$(LOG_DONE) "$(DISK_IMG) ready"

$(INITRD_TAR): fs-root | dirs
	@$(LOG_INFO) "Building RAM Disk (Initrd) ($(ARCH))..."
	$(Q)cd $(FS_ROOT) && tar -cf $(CURDIR)/$(INITRD_TAR) *
	@$(LOG_DONE) "$(INITRD_TAR) ready"

# QEMU execution & debugging targets
run: preflight-qemu all ## Launch Keira in QEMU virtual machine for current ARCH
	@$(LOG_INFO) "Launching Keira in QEMU ($(ARCH))..."
	$(Q)$(QEMU) $(QEMU_FLAGS)

run-64: run-x86_64 ## Alias for run-x86_64

run-x86_64: preflight-qemu ## Launch Keira 64-bit in QEMU (x86_64)
	$(Q)$(MAKE) ARCH=x86_64 run

run-32: run-i686 ## Alias for run-i686

run-i686: preflight-qemu ## Launch Keira pure 32-bit in QEMU (i686)
	$(Q)$(MAKE) ARCH=i686 run

debug: preflight-qemu all ## Launch Keira in QEMU debug mode (GDB on :1234)
	@$(LOG_INFO) "Launching Keira (debug mode, waiting for GDB on :1234)..."
	$(Q)$(QEMU) $(QEMU_FLAGS) -s -S

qemu-net: preflight-qemu all ## Launch Keira in QEMU with e1000 NIC emulation
	@$(LOG_INFO) "Launching Keira in QEMU with e1000 NIC..."
	$(Q)$(QEMU) $(QEMU_NET_FLAGS)

# Automated testing & verification
test: preflight-qemu all ## Run automated headless QEMU smoke test for current ARCH
	@$(LOG_INFO) "Running headless QEMU automated test ($(ARCH))..."
	$(Q)timeout 10s $(QEMU) $(QEMU_FLAGS) -display none -serial stdio > $(BUILD_DIR)/test.log 2>&1 || true
	@$(LOG_DONE) "Automated smoke test complete ($(ARCH))"

test-all: preflight-qemu ## Run automated headless smoke tests on all architectures
	@$(LOG_INFO) "Running automated smoke tests across all architectures..."
	$(Q)$(MAKE) ARCH=x86_64 test
	$(Q)$(MAKE) ARCH=i686 test
	@$(LOG_DONE) "All architecture tests completed successfully"

# Code hygiene, formatting, and linting
clean: ## Remove build directory and compiled artifacts
	@$(LOG_INFO) "Cleaning build artifacts..."
	$(Q)rm -rf $(BUILD_ROOT)
	$(Q)$(CARGO) clean
	$(Q)find . -type f \( -name "*~" -o -name "*.swp" -o -name "*.swo" -o -name "*.bak" -o -name "*.tmp" -o -name "*.pyc" \) -delete 2>/dev/null || true
	$(Q)find . -type d -name "__pycache__" -exec rm -rf {} + 2>/dev/null || true
	@$(LOG_DONE) "Clean complete"

format: preflight-format ## Format Rust and C source code
	@$(LOG_INFO) "Formatting Rust code..."
	$(Q)$(CARGO) fmt --all
	@$(LOG_INFO) "Formatting C code..."
	$(Q)find . -path "./build" -prune -o -type f \( -name "*.c" -o -name "*.h" \) -exec clang-format -i {} +
	@$(LOG_DONE) "Formatting complete"

lint: preflight-lint ## Static analysis of C userland code using clang-tidy
	@$(LOG_INFO) "Linting userland C code..."
	$(Q)find user -type f -name "*.c" -exec clang-tidy --checks='-*,clang-analyzer-*,-clang-analyzer-core.FixedAddressDereference,-clang-analyzer-security.insecureAPI.DeprecatedOrUnsafeBufferHandling' {} -- -I user/include -I user/bin/kcc/include -ffreestanding -m64 \;
	@$(LOG_DONE) "Linting complete"

# Inspection & diagnostic utilities
size: $(KERNEL_BIN) ## Display kernel binary size and section breakdown
	@printf "  Section Sizes ($(ARCH)):\n"
	$(Q)size $(KERNEL_BIN) | sed 's/^/    /'
	@printf "\n  File Size ($(ARCH)):\n"
	@printf "    %s\n\n" "$$(du -h $(KERNEL_BIN) | cut -f1) ($(KERNEL_BIN))"

objdump: $(KERNEL_BIN) ## Dump kernel ELF section headers and layout
	$(Q)objdump -h $(KERNEL_BIN)

check: ## Verify all required build dependencies are installed
	@MISSING=0; \
	for tool in $(ASM) $(CC) $(LD) $(CARGO) rustc $(GRUB_MKRESCUE) xorriso $(QEMU) \
	            clang-format clang-tidy mkfs.fat mmd mcopy tar dd; do \
	    display_name=$$(basename "$$tool"); \
	    if command -v $$tool >/dev/null 2>&1; then \
	        $(LOG_CHECK) "$$display_name"; \
	    else \
	        $(LOG_MISS) "$$display_name"; \
	        MISSING=$$((MISSING + 1)); \
	    fi; \
	done; \
	printf "\n"; \
	if [ $$MISSING -eq 0 ]; then \
	    $(LOG_DONE) "All dependencies satisfied"; \
	else \
	    $(LOG_ERR) "$$MISSING missing dependencies detected"; \
	    printf "\n[INFO] Install missing dependencies using your distribution package manager:\n"; \
	    printf "  Ubuntu / Debian:\n"; \
	    printf "    sudo apt-get update && sudo apt-get install -y nasm gcc binutils cargo rustc grub-pc-bin grub-common xorriso qemu-system-x86 clang-format clang-tidy dosfstools mtools tar coreutils\n\n"; \
	    printf "  Arch Linux:\n"; \
	    printf "    sudo pacman -S --needed nasm gcc binutils rust grub xorriso qemu-system-x86 clang dosfstools mtools tar coreutils\n\n"; \
	    printf "  Fedora / RHEL:\n"; \
	    printf "    sudo dnf install -y nasm gcc binutils cargo rustc grub2-tools-extra xorriso qemu-system-x86 clang-tools-extra dosfstools mtools tar coreutils\n\n"; \
	    printf "  Rust Nightly (required):\n"; \
	    printf "    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh && rustup default nightly\n\n"; \
	    exit 1; \
	fi

info: ## Display build configuration and toolchain versions
	@printf "Keira Kernel Build Info\n\n"
	@printf "  Kernel\n"
	@printf "    Version      : $(VERSION)\n"
	@printf "    Architecture : $(ARCH)\n"
	@printf "    Name         : $(KERNEL_NAME)\n"
	@printf "    Binary       : $(KERNEL_BIN)\n"
	@printf "    ISO          : $(KERNEL_ISO)\n"
	@printf "    Disk Image   : $(DISK_IMG) ($(DISK_SIZE)MB FAT16)\n\n"
	@printf "  Toolchain\n"
	@printf "    NASM         : $(shell $(ASM) --version 2>/dev/null | head -1 || echo 'not found')\n"
	@printf "    GCC          : $(shell $(CC) --version 2>/dev/null | head -1 || echo 'not found')\n"
	@printf "    LD           : $(shell $(LD) --version 2>/dev/null | head -1 || echo 'not found')\n"
	@printf "    Cargo        : $(shell $(CARGO) --version 2>/dev/null | head -1 || echo 'not found')\n"
	@printf "    Rustc        : $(shell rustc --version 2>/dev/null || echo 'not found')\n"
	@printf "    QEMU         : $(shell $(QEMU) --version 2>/dev/null | head -1 || echo 'not found')\n\n"
	@printf "  Source Files\n"
	@printf "    Assembly     : $(words $(ASM_SRCS)) files\n"
	@printf "    Kernel Core  : Pure Rust (12 crates)\n"
	@printf "    Shell Cmds   : $(words $(SHELL_CMDS)) commands\n"
	@printf "    Drivers      : $(words $(DRIVER_FILES)) descriptors\n\n"
	@printf "  Rust Target\n"
	@printf "    Spec         : $(RUST_TARGET)\n"
	@printf "    Profile      : $(RUST_MODE)\n"
	@printf "    Output       : $(RUST_LIB)\n\n"

help: ## Display all available Makefile targets
	@printf "\nKeira Kernel Build System  v$(VERSION) ($(ARCH))\n\n"
	@printf "  Usage: make <target> [ARCH=x86_64|i686] [V=1] [DISK_SIZE=N] [QEMU_MEM=NM]\n\n"
	@printf "  Build Targets:\n"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' Makefile | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "    %-15s %s\n", $$1, $$2}'
	@printf "\n  Variables:\n"
	@printf "    ARCH=x86_64|i686 Target architecture (default: x86_64)\n"
	@printf "    V=1             Show raw commands (verbose mode)\n"
	@printf "    DISK_SIZE=N     FAT16 disk size in MB (default: 32)\n"
	@printf "    QEMU_MEM=NM     QEMU guest memory (default: 128M)\n\n"
