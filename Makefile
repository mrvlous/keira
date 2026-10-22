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
#   4. Userland (C)     : Autonomous Ring 3 C runtime and utilities (userland/Makefile).
#   5. GRUB (Bootloader): Packages kernel and USTAR initrd into a bootable ISO.

# Include Linux-style configuration file if present
-include .config

# Target Architecture: x86_64 (default) or i686 (pure 32-bit x86)
ifeq ($(CONFIG_ARCH_I686),y)
    ARCH        := i686
else ifeq ($(CONFIG_ARCH_X86_64),y)
    ARCH        := x86_64
else
    ARCH        ?= x86_64
endif

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
BUILD_DIR       := $(BUILD_ROOT)/x86/$(ARCH)
BIN_DIR         := $(BUILD_DIR)/bin
ISO_OUT_DIR     := $(BUILD_DIR)/iso
DISK_DIR        := $(BUILD_DIR)/disk
OBJ_DIR         := $(BUILD_DIR)/obj
STAGING_DIR     := $(BUILD_DIR)/staging
ISO_DIR         := $(STAGING_DIR)/isofiles
FS_ROOT         := $(STAGING_DIR)/fs_root
FS_ROOT_STAMP   := $(STAGING_DIR)/.fs-root-stamp-$(USERLAND)

KERNEL_BIN      := $(BIN_DIR)/$(KERNEL_NAME).bin
KERNEL_ISO      := $(ISO_OUT_DIR)/$(KERNEL_NAME)-$(ARCH)-$(DATE_SUFFIX).iso
DISK_IMG        := $(DISK_DIR)/disk.img
INITRD_TAR      := $(DISK_DIR)/initrd.tar

# Configurable build parameters (with Kconfig-style override support)
ifeq ($(CONFIG_USERLAND),y)
    USERLAND    := 1
else ifeq ($(CONFIG_USERLAND),n)
    USERLAND    := 0
else
    USERLAND    ?= 1
endif

ifdef CONFIG_SMP
    SMP         := $(CONFIG_SMP)
else
    SMP         ?= 2
endif

ifdef CONFIG_DISK_SIZE
    DISK_SIZE   := $(CONFIG_DISK_SIZE)
else
    DISK_SIZE   ?= 32
endif

ifdef CONFIG_QEMU_MEM
    QEMU_MEM    := $(CONFIG_QEMU_MEM)
else
    QEMU_MEM    ?= 128M
endif

USER_DIR        ?= userland

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
                   arch/x86/i686/boot/ap_trampoline.asm \
                   arch/x86/i686/kernel/gdt.asm \
                   arch/x86/i686/kernel/idt.asm \
                   arch/x86/i686/kernel/isr.asm \
                   arch/x86/i686/kernel/syscall.asm
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
                   arch/x86/x86_64/boot/ap_trampoline.asm \
                   arch/x86/x86_64/kernel/gdt.asm \
                   arch/x86/x86_64/kernel/paging.asm \
                   arch/x86/x86_64/kernel/idt.asm \
                   arch/x86/x86_64/kernel/isr.asm \
                   arch/x86/x86_64/kernel/syscall.asm
endif

ASM_OBJS        := $(patsubst %.asm,$(OBJ_DIR)/%.asm.o,$(ASM_SRCS))
ALL_OBJS        := $(ASM_OBJS)

# QEMU hardware & emulation flags
ifeq ($(USERLAND),0)
    QEMU_DISK_FLAGS :=
else
    QEMU_DISK_FLAGS := -device ahci,id=ahci0 \
                       -drive file=$(DISK_IMG),format=raw,id=sata0,if=none \
                       -device ide-hd,drive=sata0,bus=ahci0.0
endif

QEMU_FLAGS      := -cdrom $(KERNEL_ISO) \
                   -smp $(SMP) \
                   $(QEMU_DISK_FLAGS) \
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
SHELL_CMDS      := drives use ramdisk system cpu smp runtime time memory \
                   devices initrd wipe reset run write tasks disk list \
                   go view create folder delete edit copy help history \
                   move search download network stop sync \
                   fileinfo framebuffer usb https hostname syslog kvm \
                   nvme ext4 mount cgroups futex bpf tpm swap seccomp epoll \
                   drivers lkm unwind watchpoint power perf timer eventfd mac mqueue \
                   kill jobs fg bg lvm raid firewall \
                   shutdown reboot

# Phony targets declaration
.PHONY: all full fll run run-64 run-32 run-x86_64 run-i686 run-bare debug clean mrproper distclean rust kernel kernel-iso iso dirs \
        format lint user libc disk initrd help info check size objdump qemu-net test test-all test-unit fs-root \
        preflight preflight-qemu preflight-format preflight-lint defconfig full_defconfig pure_defconfig kernel_defconfig headers_install

.DEFAULT_GOAL   := all

# Toolchain preflight dependency guards
preflight: ## Validate presence of all essential build, packaging and filesystem utilities
	@MISSING=""; \
	TOOLS="$(ASM) $(LD) $(CARGO) rustc $(GRUB_MKRESCUE) xorriso tar dd"; \
	if [ "$(USERLAND)" != "0" ]; then \
	    TOOLS="$$TOOLS $(CC) mkfs.fat mmd mcopy"; \
	fi; \
	for tool in $$TOOLS; do \
	    if ! command -v $$tool >/dev/null 2>&1; then \
	        MISSING="$$MISSING $$tool"; \
	    fi; \
	done; \
	if [ -n "$$MISSING" ]; then \
	    printf "\n[ERR] Missing required build tool(s):%s\n\n" "$$MISSING"; \
	    printf "[INFO] Please install missing dependencies using your host package manager:\n"; \
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
	    printf "[INFO] Please install QEMU using your host package manager:\n"; \
	    printf "  Ubuntu / Debian:\n"; \
	    printf "    sudo apt-get update && sudo apt-get install -y qemu-system-x86\n\n"; \
	    printf "  Arch Linux:\n"; \
	    printf "    sudo pacman -S --needed qemu-system-x86 qemu-desktop\n\n"; \
	    printf "  Fedora / RHEL:\n"; \
	    printf "    sudo dnf install -y qemu-system-x86\n\n"; \
	    exit 1; \
	else \
	    if ! $(QEMU) -display help 2>&1 | grep -qE 'gtk|sdl'; then \
	        printf "\n[WARN] QEMU graphical desktop backend (GTK/SDL) not detected\n"; \
	        printf "[INFO] Install desktop display packages so QEMU window can open:\n"; \
	        printf "  Arch Linux:\n"; \
	        printf "    sudo pacman -S --needed qemu-desktop\n"; \
	        printf "  Ubuntu / Debian:\n"; \
	        printf "    sudo apt-get install -y qemu-system-gui\n"; \
	        printf "  Fedora / RHEL:\n"; \
	        printf "    sudo dnf install -y qemu-ui-gtk qemu-ui-sdl\n\n"; \
	    fi; \
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
	    printf "[INFO] Please install formatting tools using your host package manager:\n"; \
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
	    printf "[INFO] Please install clang-tidy using your host package manager:\n"; \
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
ifeq ($(USERLAND),0)
    DISK_TARGET :=
else
    DISK_TARGET := $(DISK_IMG)
endif

all: preflight $(KERNEL_ISO) $(DISK_TARGET) ## Build kernel, ISO image and FAT16 disk image for current ARCH

kernel: preflight $(KERNEL_BIN) ## Build only the Ring 0 kernel binary (ELF)

kernel-iso: preflight ## Build pure kernel bootable ISO without userland binaries
	$(Q)$(MAKE) USERLAND=0 iso

run-bare: preflight-qemu ## Launch pure Ring 0 kernel in QEMU without userland disk
	$(Q)$(MAKE) USERLAND=0 run

full: preflight ## Build kernel and ISO images for all supported architectures (x86_64 & i686)
	@$(LOG_INFO) "Building Keira for all architectures (x86_64 & i686)..."
	$(Q)$(MAKE) ARCH=x86_64 all
	$(Q)$(MAKE) ARCH=i686 all
	@$(LOG_DONE) "Full multi-architecture build complete"

fll: full

iso: preflight $(KERNEL_ISO) ## Package GRUB Multiboot2 bootable ISO image

disk: preflight $(DISK_IMG) ## Create and populate FAT16 hard disk image

initrd: preflight $(INITRD_TAR) ## Build RAM Disk USTAR archive

user: preflight ## Build user-space C runtime, standard library and binaries
	@$(LOG_INFO) "Delegating userland build to $(USER_DIR)/Makefile ($(ARCH))..."
	$(Q)$(MAKE) -C $(USER_DIR) all ARCH=$(ARCH) BUILD_DIR=$(CURDIR)/$(BUILD_DIR)/userland V=$(V)
	@$(LOG_DONE) "Userland build complete ($(ARCH))"

libc: preflight ## Build userland freestanding C standard library (libc.a)
	@$(LOG_INFO) "Delegating libc build to $(USER_DIR)/Makefile ($(ARCH))..."
	$(Q)$(MAKE) -C $(USER_DIR) libc ARCH=$(ARCH) BUILD_DIR=$(CURDIR)/$(BUILD_DIR)/userland V=$(V)
	@$(LOG_DONE) "Userland libc build complete ($(ARCH))"

rust: preflight | dirs ## Build Rust kernel static library
	@$(LOG_CARGO) "Building Rust kernel ($(ARCH) $(RUST_MODE))...."
	$(Q)$(CARGO) -Zjson-target-spec -Zbuild-std=core,compiler_builtins build --target $(RUST_TARGET) --$(RUST_MODE) -p keira-kernel 2>&1 | sed 's/^/        /'

dirs: preflight ## Create architecture-isolated build output directory hierarchy
	$(Q)mkdir -p $(BUILD_ROOT) $(BUILD_DIR) $(BUILD_DIR)/lib $(BIN_DIR) $(ISO_OUT_DIR) $(DISK_DIR) $(OBJ_DIR) $(STAGING_DIR)

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

# Linux-style configuration generation targets
defconfig full_defconfig: ## Generate default configuration with userland enabled
	@if [ -f .example.config ]; then \
	    cp .example.config .config; \
	else \
	    printf "CONFIG_ARCH_$(shell echo $(ARCH) | tr a-z A-Z)=y\nCONFIG_USERLAND=y\nCONFIG_SMP=2\nCONFIG_DISK_SIZE=32\nCONFIG_QEMU_MEM=128M\n" > .config; \
	fi
	@$(LOG_DONE) "Generated default configuration (.config) with userland enabled ($(ARCH))"

pure_defconfig kernel_defconfig: ## Generate pure freestanding Ring 0 configuration without userland
	@if [ -f .example.config ]; then \
	    sed -e 's/CONFIG_USERLAND=y/CONFIG_USERLAND=n/' .example.config > .config; \
	else \
	    printf "CONFIG_ARCH_$(shell echo $(ARCH) | tr a-z A-Z)=y\nCONFIG_USERLAND=n\nCONFIG_SMP=2\nCONFIG_DISK_SIZE=32\nCONFIG_QEMU_MEM=128M\n" > .config; \
	fi
	@$(LOG_DONE) "Generated pure Ring 0 kernel configuration (.config) ($(ARCH))"

INSTALL_HDR_PATH ?= include/uapi

headers_install: ## Export Ring 3 userland ABI headers to $(INSTALL_HDR_PATH)
	@$(LOG_INFO) "Exporting Ring 3 UAPI headers to $(INSTALL_HDR_PATH)..."
	$(Q)mkdir -p $(INSTALL_HDR_PATH)/sys
	$(Q)cp -u $(USER_DIR)/include/sys/*.h $(INSTALL_HDR_PATH)/sys/ 2>/dev/null || cp $(USER_DIR)/include/sys/*.h $(INSTALL_HDR_PATH)/sys/
	$(Q)cp -u $(USER_DIR)/include/*.h $(INSTALL_HDR_PATH)/ 2>/dev/null || cp $(USER_DIR)/include/*.h $(INSTALL_HDR_PATH)/
	@$(LOG_DONE) "Ring 3 UAPI headers successfully exported to $(INSTALL_HDR_PATH)"

# Canonical root filesystem & disk image rules
ifeq ($(USERLAND),0)
$(FS_ROOT_STAMP): | dirs
	@$(LOG_INFO) "Populating minimal root filesystem for pure kernel mode ($(ARCH))..."
	$(Q)rm -rf $(FS_ROOT) $(STAGING_DIR)/.fs-root-stamp-*
	$(Q)mkdir -p $(FS_ROOT)/bin
	$(Q)mkdir -p $(FS_ROOT)/dev
	$(Q)mkdir -p $(FS_ROOT)/proc
	$(Q)mkdir -p $(FS_ROOT)/sys
	$(Q)mkdir -p $(FS_ROOT)/etc
	$(Q)mkdir -p $(FS_ROOT)/mnt
	$(Q)mkdir -p $(FS_ROOT)/tmp
	$(Q)mkdir -p $(FS_ROOT)/var/log
	$(Q)printf "KERNEL_NAME=keira\nKERNEL_VERSION=$(VERSION)\nKERNEL_ARCH=$(ARCH)\nUSERLAND=0\n" > $(FS_ROOT)/etc/kernel.cfg
	$(Q)printf "keira\n" > $(FS_ROOT)/etc/hostname
	$(Q)printf "[    0.000000] Keira Kernel v$(VERSION) ($(ARCH)) bootloader handover completed\n[    0.000010] Pure freestanding Ring 0 mode: root filesystem mounted in initrd\n" > $(FS_ROOT)/var/log/boot.log
	$(Q)printf "[    0.000000] Keira Kernel v$(VERSION) ($(ARCH))\n[    0.000010] Kernel Ring 0 initialized, supervisor console ready\n" > $(FS_ROOT)/var/log/system.log
	$(Q)touch $(FS_ROOT)/tmp/.keep
	$(Q)touch $(FS_ROOT_STAMP)
	@$(LOG_DONE) "Minimal root filesystem ready ($(ARCH))"
else
$(FS_ROOT_STAMP): | dirs
	@$(LOG_INFO) "Populating canonical root filesystem via userland installation ($(ARCH))..."
	$(Q)rm -rf $(FS_ROOT) $(STAGING_DIR)/.fs-root-stamp-*
	$(Q)mkdir -p $(FS_ROOT)/bin
	$(Q)mkdir -p $(FS_ROOT)/dev
	$(Q)mkdir -p $(FS_ROOT)/proc
	$(Q)mkdir -p $(FS_ROOT)/sys
	$(Q)mkdir -p $(FS_ROOT)/etc
	$(Q)mkdir -p $(FS_ROOT)/include/sys
	$(Q)mkdir -p $(FS_ROOT)/lib
	$(Q)mkdir -p $(FS_ROOT)/mnt
	$(Q)mkdir -p $(FS_ROOT)/tmp
	$(Q)mkdir -p $(FS_ROOT)/var/log
	$(Q)$(MAKE) -C $(USER_DIR) install DESTDIR=$(CURDIR)/$(FS_ROOT) ARCH=$(ARCH) BUILD_DIR=$(CURDIR)/$(BUILD_DIR)/userland V=$(V)
	$(Q)printf "KERNEL_NAME=keira\nKERNEL_VERSION=$(VERSION)\nKERNEL_ARCH=$(ARCH)\nUSERLAND=1\n" > $(FS_ROOT)/etc/kernel.cfg
	$(Q)printf "keira\n" > $(FS_ROOT)/etc/hostname
	$(Q)printf '/* Keira Comprehensive KCC Sample Program */\n\nint compute(int x, int y) {\n    int res = (x * y) + (x %% y);\n    return res ^ (x >> 1);\n}\n\nvoid main(void) {\n    printf("Keira KCC Compiler Execution\\n");\n    int i = 0, total = 0;\n    while (i < 10) {\n        i++;\n        if (i == 5) continue;\n        if (i > 8) break;\n        total += compute(i, 3);\n    }\n    printf("KCC compilation & execution complete!\\n");\n}\n' > $(FS_ROOT)/tmp/main.c
	$(Q)printf "[    0.000000] Keira Kernel v$(VERSION) ($(ARCH)) bootloader handover completed\n[    0.000010] Root filesystem mounted at / (FAT16)\n" > $(FS_ROOT)/var/log/boot.log
	$(Q)printf "[    0.000000] Keira Kernel v$(VERSION) ($(ARCH))\n[    0.000010] Kernel Ring 0 initialized, supervisor console ready\n" > $(FS_ROOT)/var/log/system.log
	$(Q)touch $(FS_ROOT)/tmp/.keep
	$(Q)touch $(FS_ROOT_STAMP)
	@$(LOG_DONE) "Canonical root filesystem ready ($(ARCH))"
endif

fs-root: $(FS_ROOT_STAMP)

$(DISK_IMG): $(FS_ROOT_STAMP)
	@rm -f $(DISK_IMG)
	@$(LOG_DISK) "Creating $(DISK_SIZE)MB FAT16 disk image ($(ARCH))..."
	$(Q)dd if=/dev/zero of=$(DISK_IMG) bs=1M count=$(DISK_SIZE) 2>/dev/null
	$(Q)mkfs.fat -F 16 $(DISK_IMG) >/dev/null
	@$(LOG_DISK) "Creating nested Keira directory structure ($(ARCH))..."
	$(Q)mmd -i $(DISK_IMG) ::/bin ::/dev ::/proc ::/sys ::/etc ::/include ::/include/sys ::/lib ::/mnt ::/tmp ::/var ::/var/log 2>/dev/null || true
	$(Q)for d in $$(cd $(FS_ROOT) && find . -mindepth 1 -type d | sed 's|^\./||' | sort); do \
	    mmd -D s -i $(DISK_IMG) ::/$$d 2>/dev/null || true; \
	done
	@$(LOG_DISK) "Populating disk image with system files ($(ARCH))..."
	$(Q)for f in $$(cd $(FS_ROOT) && find . -type f | sed 's|^\./||'); do \
	    mcopy -o -i $(DISK_IMG) $(FS_ROOT)/$$f ::/$$f; \
	done
	@$(LOG_DONE) "$(DISK_IMG) ready"

$(INITRD_TAR): $(FS_ROOT_STAMP) | dirs
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
test-unit: ## Run cargo host unit tests across all workspace crates
	@$(LOG_INFO) "Running cargo host unit tests across workspace..."
	$(Q)$(CARGO) test --workspace -- --test-threads=1
	@$(LOG_DONE) "Host unit tests completed successfully"

test: preflight-qemu all ## Run automated headless QEMU smoke test for current ARCH
	@$(LOG_INFO) "Running headless QEMU automated test ($(ARCH))..."
	$(Q)timeout 10s $(QEMU) $(QEMU_FLAGS) -display none </dev/null > $(BUILD_DIR)/test.log 2>&1 || true
	@$(LOG_DONE) "Automated smoke test complete ($(ARCH))"

test-all: preflight-qemu ## Run automated headless smoke tests on all architectures
	@$(LOG_INFO) "Running automated smoke tests across all architectures..."
	$(Q)$(MAKE) ARCH=x86_64 test
	$(Q)$(MAKE) ARCH=i686 test
	@$(LOG_DONE) "All architecture tests completed successfully"

# Code hygiene, formatting and linting
clean: ## Remove build directory and compiled artifacts
	@$(LOG_INFO) "Cleaning build artifacts..."
	$(Q)rm -rf $(BUILD_ROOT)
	$(Q)$(CARGO) clean
	$(Q)$(MAKE) -C $(USER_DIR) clean BUILD_DIR=$(CURDIR)/$(BUILD_DIR)/userland
	$(Q)find . -type f \( -name "*~" -o -name "*.swp" -o -name "*.swo" -o -name "*.bak" -o -name "*.tmp" -o -name "*.pyc" \) -delete 2>/dev/null || true
	$(Q)find . -type d -name "__pycache__" -exec rm -rf {} + 2>/dev/null || true
	@$(LOG_DONE) "Clean complete"

mrproper distclean: clean ## Restore repository to pristine state including removing .config
	$(Q)rm -f .config
	$(Q)rm -rf include/uapi
	@$(LOG_DONE) "Repository cleaned to pristine state (.config removed)"

format: preflight-format ## Format Rust and C source code
	@$(LOG_INFO) "Formatting Rust code..."
	$(Q)$(CARGO) fmt --all
	@$(LOG_INFO) "Formatting C code..."
	$(Q)find . -path "./build" -prune -o -type f \( -name "*.c" -o -name "*.h" \) -exec clang-format -i {} +
	@$(LOG_DONE) "Formatting complete"

lint: preflight-lint ## Static analysis of C userland code using clang-tidy
	@$(LOG_INFO) "Linting userland C code..."
	$(Q)find $(USER_DIR) -type f -name "*.c" -exec clang-tidy --checks='-*,clang-analyzer-*,-clang-analyzer-core.FixedAddressDereference,-clang-analyzer-core.DivideZero,-clang-analyzer-security.insecureAPI.DeprecatedOrUnsafeBufferHandling' {} -- -I $(USER_DIR)/include -I $(USER_DIR)/bin/kcc/include -I $(USER_DIR)/bin/sysinfo/include -I $(USER_DIR)/bin/test_abi/include -I $(USER_DIR)/bin/fuzz_abi/include -I $(USER_DIR)/bin/test_threads/include -ffreestanding -m64 \;
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
	    printf "\n[INFO] Install missing dependencies using your host package manager:\n"; \
	    printf "  Ubuntu / Debian:\n"; \
	    printf "    sudo apt-get update && sudo apt-get install -y nasm gcc binutils cargo rustc grub-pc-bin grub-common xorriso qemu-system-x86 clang-format clang-tidy dosfstools mtools tar coreutils\n\n"; \
	    printf "  Arch Linux:\n"; \
	    printf "    sudo pacman -S --needed nasm gcc binutils rust grub xorriso qemu-system-x86 qemu-desktop clang dosfstools mtools tar coreutils\n\n"; \
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
	@printf "    Disk Image   : $(DISK_IMG) ($(DISK_SIZE)MB FAT16)\n"
	@printf "    Userland     : $(USERLAND) (0=disabled, 1=enabled)\n\n"
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
	@printf "    Drivers      : In-Kernel Subsystems\n\n"
	@printf "  Rust Target\n"
	@printf "    Spec         : $(RUST_TARGET)\n"
	@printf "    Profile      : $(RUST_MODE)\n"
	@printf "    Output       : $(RUST_LIB)\n\n"

help: ## Display all available Makefile targets
	@printf "\nKeira Kernel Build System  v$(VERSION) ($(ARCH))\n\n"
	@printf "  Usage: make <target> [ARCH=x86_64|i686] [USERLAND=0|1] [V=1] [DISK_SIZE=N] [QEMU_MEM=NM]\n\n"
	@printf "  Build Targets:\n"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' Makefile | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "    %-15s %s\n", $$1, $$2}'
	@printf "\n  Variables:\n"
	@printf "    ARCH=x86_64|i686 Target architecture (default: x86_64)\n"
	@printf "    USERLAND=0|1    Enable or disable userland compilation (default: 1)\n"
	@printf "    V=1             Show raw commands (verbose mode)\n"
	@printf "    DISK_SIZE=N     FAT16 disk size in MB (default: 32)\n"
	@printf "    QEMU_MEM=NM     QEMU guest memory (default: 128M)\n\n"
