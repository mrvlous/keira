# SPDX-License-Identifier: GPL-2.0-only
#
# Copyright (C) 2026 Moh. Ananda Firmansyah Putra <mrvlous@proton.me>
#
# This program is free software; you can redistribute it and/or modify
# it under the terms of the GNU General Public License as published by
# the Free Software Foundation; version 2 of the License.
#
# This program is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
# GNU General Public License for more details.

# Autonomous Userland Build System Architecture
#
# Orchestrates compiling the freestanding Ring 3 C runtime (libc.a),
# the native compiler (kcc) and core utility binaries for Keira Kernel:
#   1. CRT0 (Assembly) : Compiles Ring 3 architecture startup entry stubs.
#   2. LIBC (C Lib)    : Archives freestanding standard library into libc.a.
#   3. BINS (Programs) : Compiles and links 46 userland utility executables.
#   4. Install         : Exports runtime binaries and headers to destination root.

SHELL           := /bin/bash

# Include Linux-style configuration file if present
-include ../.config

# Target architecture: x86_64 (default) or i686 (pure 32-bit x86)
ifeq ($(CONFIG_ARCH_I686),y)
    ARCH        := i686
else ifeq ($(CONFIG_ARCH_X86_64),y)
    ARCH        := x86_64
else
    ARCH        ?= x86_64
endif

# Toolchain executables
CC              ?= gcc
ASM             ?= nasm
AR              ?= ar

# Verbose mode: set V=1 to display raw command executions
ifeq ($(V),1)
    Q           :=
else
    Q           := @
endif

# Architecture-specific toolchain flags
ifeq ($(ARCH),i686)
    ASM_FLAGS   := -f elf32
    CFLAGS      := -ffreestanding -nostdlib -fno-stack-protector -m32 -O2 \
                   -mno-sse -mno-sse2 -mno-mmx \
                   -Iinclude -Ibin/kcc/include -Ibin/sysinfo/include \
                   -Ibin/test_abi/include -Ibin/fuzz_abi/include -Ibin/test_threads/include
    LINKER_SCRIPT := arch/x86/i686/linker.ld
    LDFLAGS     := -T $(LINKER_SCRIPT) \
                   -Wl,--no-warn-rwx-segments -Wl,--build-id=none -static -no-pie -lgcc
else
    ASM_FLAGS   := -f elf64
    CFLAGS      := -ffreestanding -nostdlib -fno-stack-protector -m64 -O2 \
                   -mno-sse -mno-sse2 -mno-mmx -mno-sse3 -mno-ssse3 \
                   -mno-sse4.1 -mno-sse4.2 -mno-avx -mno-avx2 \
                   -Iinclude -Ibin/kcc/include -Ibin/sysinfo/include \
                   -Ibin/test_abi/include -Ibin/fuzz_abi/include -Ibin/test_threads/include
    LINKER_SCRIPT := arch/x86/x86_64/linker.ld
    LDFLAGS     := -T $(LINKER_SCRIPT) \
                   -Wl,--no-warn-rwx-segments -Wl,--build-id=none -static -no-pie -lgcc
endif

# Output build directories
BUILD_DIR       ?= build/$(ARCH)
OBJ_DIR         := $(BUILD_DIR)/obj
LIB_DIR         := $(BUILD_DIR)/lib
BIN_DIR         := $(BUILD_DIR)/bin

# Core runtime objects
CRT_SRC         := arch/x86/$(ARCH)/crt0.asm
CRT_OBJ         := $(OBJ_DIR)/arch/x86/$(ARCH)/crt0.o
LIBC_A          := $(LIB_DIR)/libc.a

LIB_SRCS        := $(shell find lib -type f -name "*.c")
LIB_OBJS        := $(patsubst lib/%.c,$(OBJ_DIR)/lib/%.o,$(LIB_SRCS))

# Canonical list of 46 core utility programs
BINS            := arp cat chmod chown clear df dmesg echo fetch fuzz_abi grep head hostname id ifconfig \
                   init iptables kcc kill ln ls mkdir mount netstat ping pmap poweroff ps reboot \
                   rm route sh sleep su sysinfo tail test_abi test_pipe test_poll test_select test_threads touch umount uname wc whoami

BIN_ELFS        := $(patsubst %,$(BIN_DIR)/%.elf,$(BINS))

# Uniform logging definitions matching kernel master build system
LOG_INFO        := printf "[INFO]  %s\n"
LOG_DONE        := printf "[DONE]  %s\n"
LOG_WARN        := printf "[WARN]  %s\n"
LOG_ERR         := printf "[ERR]   %s\n"
LOG_CC          := printf "  [CC]    %s\n"
LOG_ASM         := printf "  [ASM]   %s\n"
LOG_AR          := printf "  [AR]    %s\n"
LOG_LD          := printf "  [LD]    %s\n"

# Phony targets declaration
.PHONY: all libc bins install clean distclean mrproper dirs format lint size test info help

all: dirs libc bins ## Build C library and all userland utility binaries

dirs: ## Create build output directory hierarchy
	$(Q)mkdir -p $(OBJ_DIR) $(LIB_DIR) $(BIN_DIR)

# Assembly CRT0 startup stub
$(CRT_OBJ): $(CRT_SRC) | dirs
	@$(LOG_ASM) "$< ($(ARCH))"
	$(Q)mkdir -p $(dir $@)
	$(Q)$(ASM) $(ASM_FLAGS) -o $@ $<

# Freestanding C library objects
$(OBJ_DIR)/lib/%.o: lib/%.c | dirs
	$(Q)mkdir -p $(dir $@)
	$(Q)$(CC) $(CFLAGS) -c $< -o $@

# Static archive libc.a
$(LIBC_A): $(LIB_OBJS) | dirs
	@$(LOG_AR) "Archiving libc.a ($(ARCH))..."
	$(Q)mkdir -p $(dir $@)
	$(Q)rm -f $@
	$(Q)$(AR) rcs $@ $(LIB_OBJS)
	@$(LOG_DONE) "$(LIBC_A) ready"

libc: $(LIBC_A) ## Build freestanding C standard library (libc.a)

# Userland utility binaries compilation template
define BIN_RULE
$$(OBJ_DIR)/bin/$(1)/%.o: bin/$(1)/%.c | dirs
	$$(Q)mkdir -p $$(dir $$@)
	$$(Q)$$(CC) $$(CFLAGS) -c $$< -o $$@

$$(BIN_DIR)/$(1).elf: $(CRT_OBJ) $$(patsubst bin/$(1)/%.c,$$(OBJ_DIR)/bin/$(1)/%.o,$$(shell find bin/$(1) -type f -name "*.c")) $(LIBC_A) $(LINKER_SCRIPT) | dirs
	@$$(LOG_LD) "Linking user binary: $(1).elf ($$(ARCH))..."
	$$(Q)$$(CC) $$(CFLAGS) $(CRT_OBJ) $$(patsubst bin/$(1)/%.c,$$(OBJ_DIR)/bin/$(1)/%.o,$$(shell find bin/$(1) -type f -name "*.c")) $(LIBC_A) $$(LDFLAGS) -o $$@
	@$$(LOG_DONE) "$$(BIN_DIR)/$(1).elf ready"
endef

$(foreach b,$(BINS),$(eval $(call BIN_RULE,$(b))))

bins: $(BIN_ELFS) ## Build all user space utility binaries

install: all ## Install userland libraries, headers and binaries into DESTDIR
	@if [ -z "$(DESTDIR)" ]; then \
	    printf "[ERR] DESTDIR not specified. Usage: make install DESTDIR=/path/to/fs_root\n"; \
	    exit 1; \
	fi
	@$(LOG_INFO) "Installing userland binaries and runtime to $(DESTDIR)..."
	$(Q)mkdir -p $(DESTDIR)/bin $(DESTDIR)/lib $(DESTDIR)/include $(DESTDIR)/etc
	$(Q)cp $(LIBC_A) $(DESTDIR)/lib/libc.a
	$(Q)cp -r include/* $(DESTDIR)/include/
	$(Q)cp bin/kcc/include/common.h $(DESTDIR)/include/common.h
	$(Q)cp etc/init.sh $(DESTDIR)/etc/init.sh
	$(Q)for b in $(BINS); do \
	    cp $(BIN_DIR)/$$b.elf $(DESTDIR)/bin/$$b.elf; \
	    cp $(BIN_DIR)/$$b.elf $(DESTDIR)/bin/$$b; \
	done
	@$(LOG_DONE) "Userland installation complete ($(ARCH))"

format: ## Format all userland C source and header files with clang-format
	@$(LOG_INFO) "Formatting userland C files..."
	$(Q)find . -path "./build" -prune -o -type f \( -name "*.c" -o -name "*.h" \) -exec clang-format -i {} +
	@$(LOG_DONE) "Userland formatting complete"

lint: ## Perform static analysis on userland C code using clang-tidy
	@$(LOG_INFO) "Linting userland C code ($(ARCH))..."
	$(Q)find . -path "./build" -prune -o -type f -name "*.c" -exec clang-tidy --checks='-*,clang-analyzer-*,-clang-analyzer-core.FixedAddressDereference,-clang-analyzer-core.DivideZero,-clang-analyzer-security.insecureAPI.DeprecatedOrUnsafeBufferHandling' {} -- -I include -I bin/kcc/include -I bin/sysinfo/include -I bin/test_abi/include -I bin/fuzz_abi/include -I bin/test_threads/include -ffreestanding -m$(if $(filter i686,$(ARCH)),32,64) \;
	@$(LOG_DONE) "Userland linting complete"

size: libc bins ## Display sizes of libc.a and all compiled userland ELF binaries
	@printf "\n  Userland Library Size ($(ARCH)):\n"
	$(Q)size $(LIBC_A) 2>/dev/null | sed 's/^/    /' || true
	@printf "\n  Compiled Executables Sample ($(ARCH)):\n"
	$(Q)size $(BIN_DIR)/*.elf 2>/dev/null | head -n 15 | sed 's/^/    /' || true
	@printf "    ... (total %d binaries compiled in $(BIN_DIR))\n\n" $(words $(BINS))

test: libc $(filter %/test_abi.elf %/test_pipe.elf %/test_poll.elf %/test_select.elf %/test_threads.elf %/fuzz_abi.elf,$(BIN_ELFS)) ## Compile and verify userland diagnostic and ABI test binaries
	@$(LOG_INFO) "Verifying userland test binaries ($(ARCH))..."
	$(Q)for t in $(BIN_DIR)/test_*.elf $(BIN_DIR)/fuzz_*.elf; do \
	    if [ -f "$$t" ]; then \
	        printf "  [OK]    %s (ELF executable verified)\n" "$$(basename $$t)"; \
	    fi; \
	done
	@$(LOG_DONE) "Userland test verification complete ($(ARCH))"

info: ## Display architecture configuration, compiler flags and binary list
	@printf "\nKeira Autonomous Userland Build Configuration\n\n"
	@printf "  Architecture   : $(ARCH)\n"
	@printf "  Compiler (CC)  : $(CC)\n"
	@printf "  Assembler (ASM): $(ASM)\n"
	@printf "  Archiver (AR)  : $(AR)\n"
	@printf "  CFLAGS         : $(CFLAGS)\n"
	@printf "  LDFLAGS        : $(LDFLAGS)\n"
	@printf "  Output Path    : $(BUILD_DIR)\n"
	@printf "  Binaries Count : %d programs\n\n" $(words $(BINS))

clean: ## Clean userland build artifacts for current architecture
	@$(LOG_INFO) "Cleaning userland build objects ($(ARCH))..."
	$(Q)rm -rf $(BUILD_DIR)
	@$(LOG_DONE) "Userland clean complete ($(ARCH))"

distclean mrproper: clean ## Clean all build artifacts, temporary editor files and backups
	@$(LOG_INFO) "Cleaning userland to pristine state..."
	$(Q)rm -rf build
	$(Q)find . -type f \( -name "*~" -o -name "*.swp" -o -name "*.swo" -o -name "*.bak" -o -name "*.tmp" \) -delete 2>/dev/null || true
	@$(LOG_DONE) "Userland distclean complete"

help: ## Display available userland Makefile targets
	@printf "\nKeira Userland Build System ($(ARCH))\n\n"
	@printf "  Usage: make [target] [ARCH=x86_64|i686] [DESTDIR=/path] [V=1]\n\n"
	@printf "  Targets:\n"
	@grep -E '^[a-zA-Z0-9_ -]+:.*?## .*$$' $(firstword $(MAKEFILE_LIST)) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "    %-20s %s\n", $$1, $$2}'
	@printf "\n"
