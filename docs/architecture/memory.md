# Memory Management Subsystem

This document describes the memory management architecture of Keira Kernel, split into Physical Memory Management, Virtual Memory Management, and the kernel bump heap allocator.

## 1. Physical Memory Manager (PMM)
The PMM, implemented in [pmm.rs](../../kernel/src/mem/pmm.rs), oversees page frame allocation (4096-byte blocks).

### Initialization and Parsing
During boot, `pmm::init` parses the Multiboot2 information structure to read the memory map tag (tag type 6).
*   **Memory Discovery**: The manager loops through memory map entries, identifying usable RAM regions and ignoring reserved regions (ACPI, memory-mapped I/O, etc.).
*   **Free Frame Stack**: Usable page frames are tracked using a stack of physical frame addresses. The stack pointer is stored in the kernel data section to avoid memory allocation before the allocator is initialized.
*   **Kernel Exclusion**: Physical frames containing the kernel executable code, stack, or the initrd image are excluded from the allocation stack.

### APIs
*   `pub fn alloc_frame() -> Option<u64>`: Pops a free physical frame address from the stack.
*   `pub fn free_frame(frame_addr: u64)`: Pushes a physical address back onto the stack of free frames.
*   `pub fn is_frame_page_aligned(phys_addr: u64) -> bool`: Validates if a physical address is 4KB page aligned.
*   `pub fn validate_phys_frame(phys_addr: u64) -> bool`: Validates if a physical frame address resides within free memory boundaries.

---

## 2. Virtual Memory Manager (VMM)
The VMM, implemented in [vmm.rs](../../kernel/src/mem/vmm.rs), implements 4-level paging to translate 64-bit virtual memory addresses into physical memory locations.

### 4-Level Page Table Structure
The translation uses the following hierarchy:
1.  **PML4 (Page Map Level 4)**: Index extracted via `(virtual_addr >> 39) & 0x1FF`.
2.  **PDPT (Page Directory Pointer Table)**: Index extracted via `(virtual_addr >> 30) & 0x1FF`.
3.  **PD (Page Directory)**: Index extracted via `(virtual_addr >> 21) & 0x1FF`.
4.  **PT (Page Table)**: Index extracted via `(virtual_addr >> 12) & 0x1FF`.

Each table contains 512 entries (8 bytes each), fitting exactly inside one 4096-byte frame.

### APIs
*   `pub unsafe fn active_pml4() -> u64`: Reads the active PML4 table address directly from the CR3 register.
*   `pub unsafe fn map_page(virtual_addr: u64, physical_addr: u64, flags: u64) -> Result<(), &'static str>`: Maps a virtual page to a physical frame. If intermediate tables (PDPT, PD, PT) do not exist, it allocates physical frames via the PMM and inserts them into the hierarchy with write and user access flags set.
*   `pub unsafe fn unmap_page(virtual_addr: u64) -> Result<(), &'static str>`: Clears the page table entry mapping the virtual address, and invalidates the page entry in the processor TLB (Translation Lookaside Buffer) using the assembly instruction `invlpg`.
*   `pub unsafe fn clone_kernel_pml4() -> Result<u64, &'static str>`: Creates a new PML4 table for a user process. It copies all higher-half kernel entries (to keep kernel code mapped in user space) while leaving the lower half clear for the user program space.
*   `pub unsafe fn mmap_anonymous(addr: u64, len: usize, prot: u64) -> Result<u64, &'static str>`: Dynamically allocates and maps contiguous virtual memory pages for Ring 3 process heap/stack requests (`sys_mmap`).
*   `pub unsafe fn munmap_pages(addr: u64, len: usize) -> Result<(), &'static str>`: Unmaps virtual memory page regions and frees physical page frames (`sys_munmap`).

### Copy-on-Write (CoW) Page Allocation
When a process executes `sys_fork` (Syscall 30), physical page frames are not duplicated upfront:
*   **Read-Only CoW Mapping**: Child and parent process page tables map identical physical frames marked as Read-Only with custom `COW` bit flags (Bit 9 in x86_64 page table entries).
*   **Write Fault Handling**: On a write attempt to a CoW page, Page Fault (Vector 14) allocates a new physical frame, copies page contents, remaps as Read-Write, and resumes task execution seamlessly.

---

## 3. Kernel Bump Heap Allocator
For early kernel driver memory requests, a sequential bump allocator is implemented in C ([heap.c](../../mm/heap.c)).

### Design Specifications
*   **16-Byte Alignment**: Allocations are rounded up to 16-byte boundaries (using `(size + 15) & ~15`) to comply with the AMD64 ABI and SIMD alignment requirements.
*   **Bump Mechanics**: The allocator retains a `heap_next` pointer. Every allocation increments `heap_next` by the requested size.
*   **Allocation Tracking**: Each `kmalloc` call increments a global `g_alloc_count` counter and updates `g_peak_used` to track peak heap consumption.
*   **Memory Recovery**: The allocator does not reclaim individual blocks. Releasing memory via `kfree` is a no-op; memory is only reclaimed during a complete reset of the allocator.

### APIs
*   `void heap_init(void *start, size_t size)`: Sets the physical bounds of the C kernel heap.
*   `void *kmalloc(size_t size)`: Allocates a contiguous block of memory. Returns `NULL` if the requested size exceeds the remaining heap capacity.
*   `void kfree(void *ptr)`: Release memory stub.
*   `size_t heap_get_alloc_count(void)`: Returns the total number of allocation requests since boot.
*   `size_t heap_get_peak(void)`: Returns the peak heap usage in bytes.
