// SPDX-License-Identifier: GPL-2.0-only

//! Implementation of the 'help' shell command.

use keira_io::vga;

pub fn run(_parts: &mut core::str::SplitWhitespace) {
    let bg = vga::Color::Black;
    let fg = vga::Color::White;

    vga::set_color(fg, bg);
    vga::print_str("Keira Kernel Diagnostic & Emergency Debugger (Ring 0 v0.8.0)\n");
    vga::print_str("Built-in Commands:\n");
    vga::print_str("  system         Display system summary, kernel version and uptime\n");
    vga::print_str("  cpu            Display CPU model, features and cache topology\n");
    vga::print_str("  smp            Display symmetric multiprocessing core status\n");
    vga::print_str("  memory         Display physical and virtual memory usage (mem)\n");
    vga::print_str(
        "  slabinfo       Display slab memory allocator caches and hit metrics (slab)\n",
    );
    vga::print_str("  devices        Enumerate PCI buses and peripheral hardware (pci)\n");
    vga::print_str("  drivers        List registered block, char and network drivers\n");
    vga::print_str("  regs           Dump CPU general-purpose and control registers\n");
    vga::print_str("  irq            Display interrupt request lines and handler counters\n");
    vga::print_str("  pidinfo [pid]  Inspect active process hierarchy and kernel task state\n");
    vga::print_str("  secinfo        Inspect CPU hardware mitigations and memory security\n");
    vga::print_str("  aslrinfo [opt] Inspect and configure ASLR layout and entropy engine\n");
    vga::print_str("  rnginfo        Inspect silicon hardware entropy and random generator\n");
    vga::print_str("  rlimit         Inspect process resource limits and sandbox quotas\n");
    vga::print_str("  unixsock       Display active UNIX domain sockets and IPC channels (unix)\n");
    vga::print_str("  vmainfo [pid]  Inspect virtual memory areas and process mappings (vma)\n");
    vga::print_str(
        "  pollinfo       Display POSIX I/O multiplexing telemetry and metrics (poll)\n",
    );
    vga::print_str(
        "  selectinfo     Display POSIX I/O multiplexing telemetry and metrics (select)\n",
    );
    vga::print_str("Emergency Control:\n");
    vga::print_str("  disk           Inspect storage drives and block device geometry\n");
    vga::print_str("  dcache         Inspect VFS in-memory dentry cache telemetry and hits\n");
    vga::print_str(
        "  icache         Inspect VFS in-memory inode cache telemetry and dirty state\n",
    );
    vga::print_str("  mount [vol]    Display mounted filesystems or mount storage volume\n");
    vga::print_str("  pipes          Inspect active IPC pipe streams and buffer allocations\n");
    vga::print_str("  sync           Flush dirty block cache buffers to non-volatile disk\n");
    vga::print_str("  syslog         Display circular kernel log buffer messages (dmesg)\n");
    vga::print_str("  unwind         Print kernel execution stack trace backtrace\n");
    vga::print_str("  watchpoint     Configure memory access watchpoints and breakpoints\n");
    vga::print_str("  wipe           Clear and reset the VGA text console display buffer\n");
    vga::print_str("Execution & Power:\n");
    vga::print_str("  sh             Launch canonical POSIX userspace shell in Ring 3\n");
    vga::print_str("  run <path>     Execute freestanding ELF binary from filesystem\n");
    vga::print_str("  reset          Reboot system via ACPI or 8042 keyboard controller\n");
    vga::print_str("  power          Power off system via ACPI shutdown\n");
    vga::print_str("  help           Display this reference manual\n");
    vga::print_str("Userspace Integration:\n");
    vga::print_str("  Core Utilities (/bin): 42 native ELF binaries execute via sh or run\n");
}
