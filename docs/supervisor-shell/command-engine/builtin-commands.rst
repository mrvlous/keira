.. SPDX-License-Identifier: GPL-2.0-only


System Information & Control Commands
=====================================

The ``sys`` command suite provides emergency telemetry, power management and hardware diagnostics for the Keira Kernel.

Command Reference
=================

.. list-table::
   :header-rows: 1
   :widths: 12 25 111 206

   * - Command
     - Syntax
     - Description
     - Flags / Options
   * - ``aslrinfo``
     - ``aslrinfo [-v] [0|1|2]``
     - Display and configure Address Space Layout Randomization (ASLR) and hardware entropy status (alias: ``aslr``)
     - ``-v, --verbose``: Display per-task randomized layout table ``-h, --help``: Usage info
   * - ``buddyinfo``
     - ``buddyinfo [-z <zone>] [-v]``
     - Display binary buddy allocator free page fragmentation by order (alias: ``buddy``)
     - ``-z, --zone <name>``: Filter by zone ``-v, --verbose``: Chunk sizes and capacities ``-h, --help``: Usage info
   * - ``cpu``
     - ``cpu``
     - Display processor model, feature flags, APIC clock, context switches and ticks
     - ``-h, --help``
   * - ``dcache``
     - ``dcache``
     - Display VFS dentry cache lookup metrics, positive/negative hits and capacity
     - ``-h, --help``
   * - ``daemons``
     - ``daemons [-h] [-f] [-r]``
     - Display runtime operational telemetry for Ring 0 background kernel daemons (alias: ``kthreads``)
     - ``-f, --flush``: Writeback pass ``-r, --reclaim``: Reclamation pass ``-h, --help``: Usage info
   * - ``devices``
     - ``devices``
     - Probe and display PCI device bus topology and configuration space (alias: ``pci``)
     - ``-h, --help``
   * - ``disk``
     - ``disk``
     - Inspect physical ATA/SATA block devices, geometry and partition layout
     - ``-h, --help``
   * - ``drivers``
     - ``drivers``
     - Display registered kernel device drivers and subsystem controller bindings
     - ``-h, --help``
   * - ``help``
     - ``help``
     - List all available Ring 0 supervisor console commands and interactive usage
     - None
   * - ``icache``
     - ``icache``
     - Display VFS inode cache telemetry, active entries and allocation statistics
     - ``-h, --help``
   * - ``insmod``
     - ``insmod <file.ko | name> [size]``
     - Load a Ring 0 relocatable ELF kernel module (``.ko``) or register a named module slot
     - ``-h, --help``
   * - ``irq``
     - ``irq``
     - Display IDT interrupt vector routing and processed hit counters
     - ``-h, --help``
   * - ``lsmod``
     - ``lsmod``
     - List active loaded kernel modules, text sizes, reference counts and memory addresses
     - ``-h, --help``
   * - ``memory``
     - ``memory [-t|-p|-c|-l|-z]``
     - Display physical memory (PMM) frames, zones, kernel heap and Copy-On-Write statistics (alias: ``mem``)
     - ``-z, --zone``: Zone watermarks ``-c, --cow``: Copy-On-Write sharing and fault metrics ``-p, --paging``: Paging, COW and #PF telemetry ``-l, --slab``: Magazine slab cache stats ``-t, --test``: Heap stress test ``-h, --help``: Usage info
   * - ``modinfo``
     - ``modinfo <name>``
     - Inspect metadata, state, size and virtual address bindings for a loaded kernel module
     - ``-h, --help``
   * - ``mount``
     - ``mount``
     - Display Virtual Filesystem mount table, filesystem types and root bindings
     - ``-h, --help``
   * - ``pidinfo``
     - ``pidinfo``
     - Display process hierarchy, task states, scheduling metrics and PID table (alias: ``procs``)
     - ``-h, --help``
   * - ``pipes``
     - ``pipes``
     - Display anonymous pipe buffer queues, active reader/writer counts and throughput metrics
     - ``-h, --help``
   * - ``pollinfo``
     - ``pollinfo``
     - Display live I/O multiplexing poll descriptor tables and event telemetry (alias: ``poll``)
     - ``-h, --help``
   * - ``power``
     - ``power [status|shutdown]``
     - Query ACPI S0-S5 states, motherboard MADT topology or soft-off power down (aliases: ``poweroff``, ``shutdown``)
     - ``-h, --help``
   * - ``regs``
     - ``regs``
     - Display CPU control registers (CR0-CR4) and execution flags
     - ``-h, --help``
   * - ``reset``
     - ``reset``
     - Reboot kernel and motherboard via PS/2 controller pulse (alias: ``reboot``)
     - ``-h, --help``
   * - ``rlimit``
     - ``rlimit [-v]``
     - Display process resource limits, soft and hard limits and sandbox quota utilization (alias: ``limit``)
     - ``-v, --verbose``: Display per-task resource utilization and quota limits ``-h, --help``: Usage info
   * - ``rmmod``
     - ``rmmod <name>``
     - Unload and detach a loaded Ring 0 kernel module by name when reference count is zero
     - ``-h, --help``
   * - ``rnginfo``
     - ``rnginfo [-v]``
     - Display silicon hardware entropy source, random generator subsystems and live entropy sample (alias: ``entropy``)
     - ``-v, --verbose``: Harvest and display 16-byte silicon entropy sample and CPU RNG features ``-h, --help``: Usage info
   * - ``run``
     - ``run <path> [args...]``
     - Load and execute a Ring 3 ELF binary or shell script from the filesystem
     - None
   * - ``secinfo``
     - ``secinfo [-v]``
     - Display hardware exploit mitigations, memory boundary protections and #PF telemetry (alias: ``mitigations``)
     - ``-v, --verbose``: Display VMM fault counters ``-h, --help``: Usage info
   * - ``selectinfo``
     - ``selectinfo``
     - Display live fd_set bitmask multiplexing telemetry and event status (alias: ``select``)
     - ``-h, --help``
   * - ``sh``
     - ``sh``
     - Launch an interactive Ring 3 userspace shell session
     - None
   * - ``slabinfo``
     - ``slabinfo``
     - Display kernel slab allocator caches, object sizes and active memory allocations (alias: ``slab``)
     - ``-h, --help``
   * - ``smp``
     - ``smp``
     - Inspect multi-core APIC topology and secondary processor status
     - ``-h, --help``
   * - ``sync``
     - ``sync``
     - Flush dirty filesystem cache sectors to physical disk
     - ``-h, --help``
   * - ``syslog``
     - ``syslog [-c]``
     - Display live kernel diagnostic ring buffer records
     - ``-c, --clear``: Clear ring buffer ``-h, --help``: Usage info
   * - ``system``
     - ``system``
     - Print kernel version, architecture, compiler target and build info
     - ``-h, --help``
   * - ``unixsock``
     - ``unixsock``
     - Display UNIX domain socket endpoints, backlog queues and bound path hierarchy (alias: ``unix``)
     - ``-h, --help``
   * - ``unwind``
     - ``unwind``
     - Display stack frame trace and symbol unwind diagnostic test
     - ``-h, --help``
   * - ``vmainfo``
     - ``vmainfo [-a] [pid]``
     - Display Virtual Memory Areas (VMAs), page mappings and address space layout (alias: ``vma``)
     - ``-a, --all``: Inspect VMAs for all active tasks ``-h, --help``: Usage info
   * - ``waitqueue``
     - ``waitqueue [-v]``
     - Display kernel wait queues and condition sleep synchronization metrics (alias: ``wq``)
     - ``-v, --verbose``: Show waiting task PIDs per queue ``-h, --help``: Usage info
   * - ``watchpoint``
     - ``watchpoint``
     - Inspect hardware debug registers (DR0-DR3) and watchpoint traps
     - ``-h, --help``
   * - ``wipe``
     - ``wipe``
     - Clear and reset the VGA text console display buffer
     - ``-h, --help``
   * - ``workqueue``
     - ``workqueue [-v|-f|-t]``
     - Display deferred kernel workqueues and ``[kworker/0:0]`` execution statistics (alias: ``kworker``)
     - ``-v, --verbose``: List pending work descriptors ``-f, --flush``: Flush queued jobs ``-t, --test``: Run probe job ``-h, --help``: Usage info
