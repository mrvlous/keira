# SPDX-License-Identifier: GPL-2.0-only

# Keira Kernel Master Root Kbuild Manifest
#
# Central entry point for kernel build manifests across architecture trees
# and core static library crates.

# Architecture-specific bootstrap and kernel low-level assembly
include arch/x86/Kbuild

# Hyper-modular Ring 0 static library crates
include crates/Kbuild
