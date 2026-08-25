<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Git Workflow & Pull Request Guidelines

Guidelines for branches, commits, and pull requests in Keira Kernel.

## Branch Naming Conventions
- `feature/<crate>-<short-description>`: E.g., `feature/net-ipv6-support`
- `fix/<subsystem>-<issue>`: E.g., `fix/vmm-page-unmap`
- `refactor/<target>`: E.g., `refactor/ipc-uring-queue`
- `docs/<topic>`: E.g., `docs/contributing-guides`

## Commit Message Format (Conventional Commits)
Each commit must follow the standard convention:
```
<type>(<scope>): <short imperative summary>

[optional detailed description explaining WHY, not just what]
```

### Allowed Types:
- `feat`: New driver, syscall, command, or kernel feature.
- `fix`: Bug fix, race condition resolution, or panic prevention.
- `refactor`: Code reorganization with zero behavior change.
- `perf`: Optimization in memory allocation, TLB invalidation, or packet processing.
- `docs`: Documentation addition or clarification.
- `style`: Formatting, whitespace, or rustfmt fixes.
- `test`: Automated QEMU test harness or unit test additions.

## Pull Request Checklist
Before opening a PR, ensure:
1. `make check` passes with all tools detected.
2. `cargo check --workspace -Zjson-target-spec -Zbuild-std=core,compiler_builtins --target targets/x86/x86_64-keira-none.json` produces **0 errors and 0 warnings**.
3. `make all` and `make test` pass cleanly.
4. All new files contain clean GPL-2.0-only license headers without email addresses.
