<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Git Workflow, Commit Standards & Pull Requests

This document specifies branch naming conventions, Conventional Commit formatting, and pull request review standards for Keira Kernel.

---

## Branch Naming Conventions

* `feature/<domain>-<short-description>`: E.g., `feature/net-ipv6-support`
* `fix/<subsystem>-<issue>`: E.g., `fix/vmm-page-unmap`
* `refactor/<target>`: E.g., `refactor/ipc-uring-queue`
* `docs/<topic>`: E.g., `docs/contributing-guides`

---

## Commit Message Format (Conventional Commits)

Each commit must follow the standard conventional commit format:
```text
<type>(<scope>): <short imperative summary>

[optional detailed description explaining WHY, not just what]
```

### Allowed Commit Types:
* `feat`: New driver, syscall, shell command, or kernel subsystem feature.
* `fix`: Bug fix, race condition resolution, or panic prevention.
* `refactor`: Code reorganization with zero behavior change.
* `perf`: Performance optimization in memory allocation, TLB, or packets.
* `docs`: Documentation addition or clarification.
* `style`: Formatting, whitespace, or rustfmt fixes.
* `test`: Automated QEMU test harness or unit test additions.

---

## Architecture & Code Review Principles

1. **Unidirectional Layering**: Lower-level crates (`core`, `arch`, `mem`) must NEVER import from higher-level crates (`fs`, `net`, `task`, `shell`).
2. **Single Responsibility**: Decompose large modules into granular sub-files (`mod.rs`, `types.rs`, `operations.rs`).
3. **No Dead Code / Scaffolding**: Avoid empty files, empty directories, or placeholder stubs without concrete implementations.
4. **Zero Userland Bloat**: The userland footprint is restricted to clean, standalone C tools (`kcc.elf`).

---

## Release Versioning & Git Tagging Lifecycle

Keira Kernel follows strict **Semantic Versioning (`MAJOR.MINOR.PATCH`)** for its release lifecycle:

* **Baseline (`0.1.0`)**: Represents the unified foundation release featuring 100% pure Rust modular architecture, dual-architecture parity (`x86_64` & `i686`), Ring 3 isolation, freestanding POSIX C SDK, native in-kernel C compiler (`kcc`), FAT16 filesystem, and TCP/IP stack.
* **Patch Releases (`0.1.x`)**: Reserved for backward-compatible bug fixes, driver optimizations, and security hardening.
* **Minor Releases (`0.x.0`)**: Introduced when major kernel milestones are achieved (e.g., graphical window manager, extended filesystem write support, or cross-architecture porting).
* **Major Releases (`x.0.0`)**: Reserved for frozen ABI stability milestones.

### Release Git Tagging

Release tags use the standard prefix `v` (e.g., `v0.1.0`). Tags are created only on major milestones and release commits.

---

## Pull Request Checklist

Before opening a pull request, ensure:
1. `make check` passes with all tools detected.
2. `cargo check --workspace -Zjson-target-spec -Zbuild-std=core,compiler_builtins --target targets/x86/x86_64-keira-none.json` produces **0 errors and 0 warnings**.
3. `make all` and `make test` pass cleanly.
4. All new files contain clean GPL-2.0-only license headers with the author's full name (no email addresses in headers).
