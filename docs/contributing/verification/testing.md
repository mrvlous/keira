<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Automated Testing & Verification Suite

Overview of automated quality assurance and verification mechanisms.

## Headless Smoke Testing
Runs automated boot validation in QEMU without a graphical window:
```bash
make test
```

## QMP Automated Script Testing
The QEMU Machine Protocol (QMP) interface allows external test harnesses to send keystrokes, execute shell commands, and capture high-resolution framebuffer screendumps (`screendump`):

```bash
# Launch test harness
python3 -c "import subprocess; subprocess.run(['make', 'all'])"
```

## 20-Cycle Stress Testing
The 20-cycle automated stress test verifies that repeated execution of kernel commands, userland Ring 3 ELF compilations (`run /apps/bin/kcc.elf`), and VMM address space cloning does not leak physical memory or trigger kernel panics.
