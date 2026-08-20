<!-- SPDX-License-Identifier: GPL-2.0-only -->

# `keira-ipc` - Inter-Process Communication

The `keira-ipc` crate implements anonymous pipes, zero-copy splicing, POSIX shared memory, `io_uring` ring queues, `eventfd`, `epoll`, POSIX message queues, and Fast Userspace Mutexes (Futex).

## Submodules

- [`pipe.md`](pipe.md): Anonymous FIFOs and zero-copy `splice`.
- [`shm.md`](shm.md): POSIX shared memory segments and semaphores.
- [`uring.md`](uring.md): `io_uring` submission and completion queues.
- [`event.md`](event.md): `eventfd`, `signalfd`, and `epoll`.
- [`mqueue.md`](mqueue.md): POSIX priority message queues.
- [`futex.md`](futex.md): Fast Userspace Mutex wait queues.
