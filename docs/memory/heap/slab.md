<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Slab Cache Object Allocator

For frequent, fixed-size kernel structures (`task_struct`, `vma`, `file_descriptor`), the slab cache provides $O(1)$ allocation without fragmentation.
