<!-- SPDX-License-Identifier: GPL-2.0-only -->

# SQE & CQE Ring Structure

* **SQ (Submission Queue)**: Userland submits I/O requests (`read`, `write`, `fsync`).
* **CQ (Completion Queue)**: Kernel writes completed results with status codes.
