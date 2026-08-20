<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Subsystem Isolation & Architecture Review Rubric

Review criteria for proposed changes to Keira Kernel.

## Review Principles

1. **Unidirectional Layering**: Lower-level crates (`core`, `arch`, `mem`) must NEVER import from higher-level crates (`fs`, `net`, `task`, `shell`).
2. **Single Responsibility**: Decompose large modules into granular sub-files (`mod.rs`, `types.rs`, `operations.rs`).
3. **No Dead Code / Scaffolding**: Avoid empty files, empty directories, or placeholder stubs without concrete implementations.
4. **Zero Userland Bloat**: The userland footprint is restricted to clean, standalone C tools (`gcc.elf`).
