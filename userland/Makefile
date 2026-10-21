# SPDX-License-Identifier: GPL-2.0-only
#
# Keira Kernel - Freestanding Kernel from Scratch
# Copyright (C) 2026 Moh. Ananda Firmansyah Putra
#
# This program is free software; you can redistribute it and/or modify
# it under the terms of the GNU General Public License as published by
# the Free Software Foundation; version 2 of the License.
#
# Independent autonomous Makefile for Ring 3 C runtime libraries and core utilities.

ARCH            ?= x86_64
CC              ?= gcc
ASM             ?= nasm
AR              ?= ar

# Verbose mode
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

# Output directories
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

# Canonical list of 30 core utility programs
BINS            := arp cat clear df dmesg fetch fuzz_abi hostname id ifconfig \
                   init iptables kcc kill ln ls mkdir netstat ping ps \
                   rm route sh su sysinfo test_abi test_threads touch uname whoami

BIN_ELFS        := $(patsubst %,$(BIN_DIR)/%.elf,$(BINS))

# Logging macros
LOG_INFO        := printf "  [USER-INFO] %s\n"
LOG_CC          := printf "  [USER-CC]   %s\n"
LOG_ASM         := printf "  [USER-ASM]  %s\n"
LOG_AR          := printf "  [USER-AR]   %s\n"
LOG_LD          := printf "  [USER-LD]   %s\n"
LOG_DONE        := printf "[USER-DONE]  %s\n"

.PHONY: all libc bins install clean dirs help

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

# Userland utility binaries
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

clean: ## Clean userland build artifacts
	@$(LOG_INFO) "Cleaning userland build objects..."
	$(Q)rm -rf $(BUILD_DIR)
	@$(LOG_DONE) "Userland clean complete"

help: ## Display userland Makefile targets
	@printf "\nKeira Userland Build System ($(ARCH))\n\n"
	@printf "  Usage: make <target> [ARCH=x86_64|i686] [DESTDIR=/path] [V=1]\n\n"
	@printf "  Targets:\n"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' Makefile | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "    %-15s %s\n", $$1, $$2}'
	@printf "\n"
