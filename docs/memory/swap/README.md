<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Swap Partition & Paging Engine

When physical RAM is depleted, the swap engine (`crates/mem/src/swap/pager/`) evicts inactive pages to disk.
