<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Page Fault Handling & Demand Paging

Hardware page faults (`Vector 14 / #PF`) are trapped and resolved by the kernel.

---

## Fault Resolution

1. Read faulting linear address from CPU register `CR2`.
2. Inspect the active process VMA list to verify if the address falls within a valid mapping.
3. If valid, allocate a physical frame from the PMM, map it into the page table, and resume execution.
4. For user stack auto-growth within the stack boundary, newly allocated pages enforce $W \oplus X$ protections (`PAGE_NO_EXECUTE` on x86_64) preventing stack-based code injection.
5. If invalid or violating page protections, deliver a `SIGSEGV` signal to the offending process.

---

## Page Fault & Paging Telemetry

The VMM telemetry engine (`crates/mem/src/vmm/fault/handler.rs`) tracks hardware page fault occurrences:
* `vmm_get_fault_stats()`: Computes `(total_faults, cow_faults, stack_growths, demand_pages, violations, tlb_flushes)`.
* Tracks Copy-on-Write (COW) duplications, stack expansions, demand paging allocations, protection faults (SIGSEGV), and TLB page invalidations via `invlpg`.
* Accessible in userspace via the `memory -p` or `memory --paging` shell command.

---

## Copy-on-Write (COW) Fault Resolution

1. On a write fault to a present page where `PAGE_COW` (bit 9) is set:
2. If `pmm::frame_refcount(old_frame) <= 1`, sole ownership is restored: simply clear `PAGE_COW`, set `PAGE_WRITABLE`, and flush TLB via `invlpg`.
3. If `pmm::frame_refcount(old_frame) > 1`:
   - Allocate a new physical frame from PMM.
   - Copy 4096 bytes from the faulting page to the new frame.
   - Update the PTE with the new physical frame, set `PAGE_WRITABLE`, and clear `PAGE_COW`.
   - Call `pmm::release_frame(old_frame)` to decrement old frame reference count.
   - Flush TLB via `invlpg` and resume execution.
