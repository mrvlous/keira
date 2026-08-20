<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Unified Error Taxonomy

Documentation for error definitions in [`crates/core/src/error.rs`](../../../crates/core/src/error.rs).

## `KernelError` Variants

- `OutOfMemory`: Physical page or kernel heap exhaustion.
- `InvalidArgument`: Out-of-bounds syscall parameter or invalid pointer.
- `PermissionDenied`: Ring 3 access violation or MAC security denial.
- `NotFound`: File, directory, device, or task PID not found.
- `DeviceBusy`: Hardware resource or I/O controller lock acquisition failure.
- `TimedOut`: Hardware polling loop or network handshake timeout.
- `NotSupported`: Unimplemented system call vector or unsupported filesystem layout.
- `IoError`: Low-level IDE, AHCI, NVMe, or network hardware controller error.
