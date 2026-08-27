<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Keira Kernel SPDX License Architecture

To maintain clarity, auditability, and automated toolchain compliance, Keira Kernel adopts the **SPDX (Software Package Data Exchange)** machine-readable licensing specification modeled after the **Linux Kernel** and **REUSE 3.0** open-source standards.

---

## Directory Structure

```
LICENSES/
├── README.md                  # This documentation guide
├── preferred/                 # Canonical full texts for preferred licenses
│   ├── GPL-2.0                # GNU General Public License, Version 2 (GPL-2.0-only)
│   └── MIT                    # The MIT License (MIT)
└── exceptions/                # Legal license exceptions
    └── Linux-syscall-note     # Standard System Call Exception Note
```

---

## Subdirectory Classifications

### 1. `LICENSES/preferred/`
Contains licenses that are officially supported and encouraged for kernel core code, drivers, and userland SDK components:
* **[`GPL-2.0`](preferred/GPL-2.0)**: The primary copyleft license covering the kernel core (`crates/*`), bootstrap assembly, and memory management.
* **[`MIT`](preferred/MIT)**: Permissive license option for userland C SDK headers (`user/include/`) and standalone toolchain utilities.

### 2. `LICENSES/exceptions/`
Contains license exception texts that modify or grant additional permissions to base licenses:
* **[`Linux-syscall-note`](exceptions/Linux-syscall-note)**: Clarifies that userland applications executing system calls against Keira Kernel are not classified as derivative works under the GPL.

---

## Header Formatting Rules for Source Code

Every source file in the repository must include a machine-readable SPDX identifier tag in its leading comments:

### Rust Source Files (`.rs`):
```rust
// SPDX-License-Identifier: GPL-2.0-only
```

### C & Header Files (`.c`, `.h`):
```c
/* SPDX-License-Identifier: GPL-2.0-only */
```

### Assembly Files (`.asm`, `.inc`):
```nasm
; SPDX-License-Identifier: GPL-2.0-only
```

### Markdown Documentation (`.md`):
```markdown
<!-- SPDX-License-Identifier: GPL-2.0-only -->
```

---

## Contribution & Copyright Attribution

When creating new standalone files, authors must specify their full legal name without email addresses:
```text
Copyright (C) 2026 <Author Full Name>
```

Email addresses and contact details are maintained exclusively in [`MAINTAINERS`](../MAINTAINERS) and [`CREDITS`](../CREDITS).
