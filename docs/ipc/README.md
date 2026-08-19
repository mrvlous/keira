<!--
SPDX-License-Identifier: GPL-2.0-only

Keira Kernel - Operating System Kernel
Copyright (C) 2026 Moh. Ananda Firmansyah Putra
-->

# Inter-Process Communication & Asynchronous I/O

Welcome to the IPC and Asynchronous I/O documentation section for Keira Kernel.

## Documents

* [Asynchronous Kernel I/O Engine (io_uring)](iouring.md): Zero-copy ring buffer I/O (`sys_io_uring_setup`).
* [Fast Userspace Mutex (Futex)](futex.md): Atomic userspace locking and kernel wait queue synchronization (`sys_futex`).
* [Epoll Scalable I/O Event Engine](epoll.md): Scalable O(1) event multiplexing descriptors (`sys_epoll_create`/`sys_epoll_ctl`).
* [EventFD & SignalFD Subsystem](eventfd.md): Counter notification descriptors (`sys_eventfd`) and POSIX signal routing (`sys_signalfd`).
* [POSIX Message Queue IPC Subsystem](mqueue.md): In-kernel priority message queues (`sys_mq_open`).
* [Zero-Copy Kernel Pipe Splice](splice.md): In-kernel page swapping between file descriptors (`sys_splice`/`sys_vmsplice`).
* [POSIX Shared Memory IPC & Semaphore Subsystem](shm_sem.md): Shared physical memory pages (`shmget`/`shmat`) and counting semaphores (`sys_shm_sem`).
* [POSIX Real-Time Signal Engine & Process Job Control](signal.md): POSIX signals (`SIGKILL`, `SIGTERM`, `SIGINT`) and terminal job control (`sys_kill`).
