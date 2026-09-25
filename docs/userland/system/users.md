<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Multi-User Credentials & Authentication

Keira implements a multi-user credential system adhering to POSIX standards.

---

## User & Group Identifiers

* **Superuser**: UID `0` (`root`), GID `0` (`root`). Possesses unrestricted permissions to hardware, kernel modules, and raw disk blocks.
* **Standard Users**: UID $\ge 1000$. Restricted by file permission masks and process isolation.
* **System Accounts**: UID `1`--`999` reserved for system daemons (e.g. `daemon`, `nobody`).

---

## Configuration Files

* `/etc/passwd`: `username:x:uid:gid:gecos:home_dir:shell`
* `/etc/group`: `groupname:x:gid:member1,member2`
