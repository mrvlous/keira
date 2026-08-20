<!--
SPDX-License-Identifier: GPL-2.0-only

Keira Kernel - Operating System Kernel
Copyright (C) 2026 Moh. Ananda Firmansyah Putra
-->

# Contribution and Patch Submission Workflow

Thank you for your interest in contributing to **Keira Kernel**! This guide walks you through the recommended workflow for developing features, fixing bugs, testing changes, and submitting patches or pull requests.

---

## 1. Development Lifecycle Overview

```
+----------------+      +-------------------+      +--------------------+
|  1. Fork &     | ---> |  2. Implement &   | ---> |  3. Format & Lint  |
|  Branch        |      |  Test Locally     |      |  (make format)     |
+----------------+      +-------------------+      +--------------------+
                                                             |
                                                             v
+----------------+      +-------------------+      +--------------------+
|  6. Review &   | <--- |  5. Open Pull     | <--- |  4. Pre-Flight     |
|  Merge         |      |  Request          |      |  Check (make test) |
+----------------+      +-------------------+      +--------------------+
```

---

## 2. Step-by-Step Workflow

### Step 1: Fork and Clone
1. Fork the repository on GitHub: [https://github.com/mrvlous/keira](https://github.com/mrvlous/keira)
2. Clone your fork locally:
   ```bash
   git clone https://github.com/<your-username>/keira.git
   cd keira
   ```
3. Set up the upstream remote:
   ```bash
   git remote add upstream https://github.com/mrvlous/keira.git
   git fetch upstream
   ```

### Step 2: Create a Feature Branch
Always create a dedicated topic branch from the latest `main`:
```bash
git checkout -b feat/your-feature-name
# or for bug fixes:
git checkout -b fix/your-bugfix-name
```

### Step 3: Implement and Test Locally
* Compile and test the full system in QEMU:
  ```bash
  make run
  ```
* Run automated headless smoke tests:
  ```bash
  make test
  ```
* Inspect kernel binary size and section breakdown:
  ```bash
  make size
  ```

### Step 4: Format Code and Add License Headers
Before committing, ensure your code adheres to project formatting rules:
```bash
# Auto-format all Rust and C code
make format

# Verify build toolchain dependencies
make check
```

* **License Headers**: Ensure every newly created file begins with the appropriate GPL-2.0-only license header with **your name/entity and year** on the Copyright line (see [Coding Style Guidelines](style.md)).
* **Contributor Attribution**: Feel free to add your name to [CREDITS](../../CREDITS)!

### Step 5: Commit Your Changes
Use concise, descriptive commit messages adhering to conventional commit conventions or the release format:
```bash
# For feature additions:
git commit -m "feat(sched): implement priority boost for interactive tasks"

# For bug fixes:
git commit -m "fix(vmm): preserve MMIO page table mappings during PML4 clone"

# For documentation updates:
git commit -m "docs(contributing): clarify license header copyright attribution"
```

### Step 6: Push and Open a Pull Request
1. Push your branch to your GitHub fork:
   ```bash
   git push origin feat/your-feature-name
   ```
2. Open a Pull Request against `upstream/main` on GitHub.
3. Provide a clear summary of what changed, why it was needed, and how it was tested.

---

## 3. Pre-Flight Pull Request Checklist

Before submitting your PR, verify the following checklist:

- [ ] `make check` succeeds with all dependencies satisfied (`[OK]`).
- [ ] `make format` has been executed to ensure clean formatting.
- [ ] `make test` passes without errors.
- [ ] All new files have the standard GPL-2.0-only license header with proper copyright attribution.
- [ ] All public Rust items (`struct`, `enum`, `fn`) have doc comments (`///`).
- [ ] Code comments use formal technical English without non-English phrases or decorative dividers (`---`, `===`).
- [ ] No regression in freestanding `no_std` kernel builds (`cargo check` produces 0 errors and 0 warnings).

---

## 4. Core Development Commandments

1. **Strict Freestanding Execution**: Never introduce dependencies on standard OS host runtimes (`std`, dynamic shared libraries, or host libc).
2. **Deterministic Memory Safety**: Always unmap virtual pages and free physical frames upon process or buffer lifecycle completion.
3. **Graceful Error Handling**: Avoid panicking in kernel interrupt handlers; always return descriptive `Result<T, &'static str>` or error codes.
4. **Preserve System Architecture**: Maintain modular isolation between architecture code (`arch/`), hardware drivers (`drivers/`), kernel core (`kernel/`), and user space (`user/`).
