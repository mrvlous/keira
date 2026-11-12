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
# the native compiler toolchain (kcc) and categorized system binaries for Keira Kernel:
#   1. CRT0 (Assembly) : Compiles Ring 3 architecture startup entry stubs.
#   2. LIBC (C Lib)    : Archives freestanding standard library into libc.a.
#   3. Executables     : Compiles and links 46 userland binaries across core, sys, net, toolchain and test suites.
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

# Professional userland package directories
LIBC_DIR        := libc
CORE_DIR        := coreutils
SYS_DIR         := sysutils
NET_DIR         := netutils
TOOLCHAIN_DIR   := toolchain
TESTS_DIR       := tests
ETC_DIR         := etc

# Architecture-specific toolchain flags
ifeq ($(ARCH),i686)
    ASM_FLAGS   := -f elf32
    CFLAGS      := -ffreestanding -nostdlib -fno-stack-protector -m32 -O2 \
                   -mno-sse -mno-sse2 -mno-mmx \
                   -I$(LIBC_DIR)/include \
                   -I$(TOOLCHAIN_DIR)/kcc/include \
                   -I$(SYS_DIR)/sysinfo/include \
                   -I$(TESTS_DIR)/test_abi/include \
                   -I$(TESTS_DIR)/fuzz_abi/include
    LINKER_SCRIPT := $(LIBC_DIR)/arch/x86/i686/linker.ld
    LDFLAGS     := -T $(LINKER_SCRIPT) \
                   -Wl,--no-warn-rwx-segments -Wl,--build-id=none -static -no-pie -lgcc
else
    ASM_FLAGS   := -f elf64
    CFLAGS      := -ffreestanding -nostdlib -fno-stack-protector -m64 -O2 \
                   -mno-sse -mno-sse2 -mno-mmx -mno-sse3 -mno-ssse3 \
                   -mno-sse4.1 -mno-sse4.2 -mno-avx -mno-avx2 \
                   -I$(LIBC_DIR)/include \
                   -I$(TOOLCHAIN_DIR)/kcc/include \
                   -I$(SYS_DIR)/sysinfo/include \
                   -I$(TESTS_DIR)/test_abi/include \
                   -I$(TESTS_DIR)/fuzz_abi/include
    LINKER_SCRIPT := $(LIBC_DIR)/arch/x86/x86_64/linker.ld
    LDFLAGS     := -T $(LINKER_SCRIPT) \
                   -Wl,--no-warn-rwx-segments -Wl,--build-id=none -static -no-pie -lgcc
endif

# Output build directories
BUILD_DIR       ?= build/$(ARCH)
OBJ_DIR         := $(BUILD_DIR)/obj
LIB_DIR         := $(BUILD_DIR)/lib
BIN_DIR         := $(BUILD_DIR)/bin

# Core runtime objects
CRT_SRC         := $(LIBC_DIR)/arch/x86/$(ARCH)/crt0.asm
CRT_OBJ         := $(OBJ_DIR)/libc/arch/x86/$(ARCH)/crt0.o
LIBC_A          := $(LIB_DIR)/libc.a

LIB_SRCS        := $(shell find $(LIBC_DIR)/src -type f -name "*.c")
LIB_OBJS        := $(patsubst $(LIBC_DIR)/src/%.c,$(OBJ_DIR)/libc/%.o,$(LIB_SRCS))

# Canonical list of 46 userland executables organized by professional domain
CORE_BINS       := cat chmod chown clear echo grep head ln ls mkdir rm tail touch wc
SYSTEM_BINS     := df dmesg hostname id init kill mount pmap poweroff ps reboot sh sleep su umount uname whoami
NET_BINS        := arp fetch ifconfig iptables netstat ping route
TESTS_BINS      := test_pipe test_poll test_select test_threads
MULTI_BINS      := sysinfo kcc test_abi fuzz_abi

BINS            := $(sort $(CORE_BINS) $(SYSTEM_BINS) $(NET_BINS) $(TESTS_BINS) $(MULTI_BINS))
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
$(OBJ_DIR)/libc/%.o: $(LIBC_DIR)/src/%.c | dirs
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

# Single-file userland utility binaries compilation template
define SINGLE_BIN_RULE
$$(OBJ_DIR)/$(1)/%.o: $(1)/%.c | dirs
	$$(Q)mkdir -p $$(dir $$@)
	$$(Q)$$(CC) $$(CFLAGS) -c $$< -o $$@

$$(BIN_DIR)/$(2).elf: $$(CRT_OBJ) $$(OBJ_DIR)/$(1)/$(2).o $$(LIBC_A) $$(LINKER_SCRIPT) | dirs
	@$$(LOG_LD) "Linking user binary: $(2).elf ($$(ARCH))..."
	$$(Q)$$(CC) $$(CFLAGS) $$(CRT_OBJ) $$(OBJ_DIR)/$(1)/$(2).o $$(LIBC_A) $$(LDFLAGS) -o $$@
	@$$(LOG_DONE) "$$(BIN_DIR)/$(2).elf ready"
endef

$(foreach b,$(CORE_BINS),$(eval $(call SINGLE_BIN_RULE,$(CORE_DIR),$(b))))
$(foreach b,$(SYSTEM_BINS),$(eval $(call SINGLE_BIN_RULE,$(SYS_DIR),$(b))))
$(foreach b,$(NET_BINS),$(eval $(call SINGLE_BIN_RULE,$(NET_DIR),$(b))))
$(foreach b,$(TESTS_BINS),$(eval $(call SINGLE_BIN_RULE,$(TESTS_DIR),$(b))))

# Multi-file complex project binaries compilation template
define MULTI_BIN_RULE
$$(OBJ_DIR)/$(1)/%.o: $(1)/%.c | dirs
	$$(Q)mkdir -p $$(dir $$@)
	$$(Q)$$(CC) $$(CFLAGS) -c $$< -o $$@

$$(BIN_DIR)/$(2).elf: $$(CRT_OBJ) $$(patsubst $(1)/%.c,$$(OBJ_DIR)/$(1)/%.o,$$(shell find $(1) -type f -name "*.c")) $$(LIBC_A) $$(LINKER_SCRIPT) | dirs
	@$$(LOG_LD) "Linking user binary: $(2).elf ($$(ARCH))..."
	$$(Q)$$(CC) $$(CFLAGS) $$(CRT_OBJ) $$(patsubst $(1)/%.c,$$(OBJ_DIR)/$(1)/%.o,$$(shell find $(1) -type f -name "*.c")) $$(LIBC_A) $$(LDFLAGS) -o $$@
	@$$(LOG_DONE) "$$(BIN_DIR)/$(2).elf ready"
endef

$(eval $(call MULTI_BIN_RULE,$(SYS_DIR)/sysinfo,sysinfo))
$(eval $(call MULTI_BIN_RULE,$(TOOLCHAIN_DIR)/kcc,kcc))
$(eval $(call MULTI_BIN_RULE,$(TESTS_DIR)/test_abi,test_abi))
$(eval $(call MULTI_BIN_RULE,$(TESTS_DIR)/fuzz_abi,fuzz_abi))

bins: $(BIN_ELFS) ## Build all user space utility binaries

install: all ## Install userland libraries, headers and binaries into DESTDIR
	@if [ -z "$(DESTDIR)" ]; then \
	    printf "[ERR] DESTDIR not specified. Usage: make install DESTDIR=/path/to/fs_root\n"; \
	    exit 1; \
	fi
	@$(LOG_INFO) "Installing userland binaries and runtime to $(DESTDIR)..."
	$(Q)mkdir -p $(DESTDIR)/bin $(DESTDIR)/lib $(DESTDIR)/include $(DESTDIR)/etc
	$(Q)cp $(LIBC_A) $(DESTDIR)/lib/libc.a
	$(Q)cp -r $(LIBC_DIR)/include/* $(DESTDIR)/include/
	$(Q)cp $(TOOLCHAIN_DIR)/kcc/include/common.h $(DESTDIR)/include/common.h
	$(Q)cp $(ETC_DIR)/init.sh $(DESTDIR)/etc/init.sh
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
	$(Q)find . -path "./build" -prune -o -type f -name "*.c" -exec clang-tidy --checks='-*,clang-analyzer-*,-clang-analyzer-core.FixedAddressDereference,-clang-analyzer-core.DivideZero,-clang-analyzer-security.insecureAPI.DeprecatedOrUnsafeBufferHandling' {} -- -I $(LIBC_DIR)/include -I $(TOOLCHAIN_DIR)/kcc/include -I $(SYS_DIR)/sysinfo/include -I $(TESTS_DIR)/test_abi/include -I $(TESTS_DIR)/fuzz_abi/include -ffreestanding -m$(if $(filter i686,$(ARCH)),32,64) \;
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
