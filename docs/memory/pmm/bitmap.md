<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Physical Memory Manager (PMM) & Frame Bitmap

The Physical Memory Manager (PMM) tracks physical RAM allocations at page granularity (4096 bytes per frame) using an efficient bit array allocator.

---

## 1. Frame Allocation Mathematics

Physical memory is mapped 1:1 into a flat array of 64-bit words:
$$\text{Frame Index} = \frac{\text{Physical Address}}{4096}$$
$$\text{Bitmap Word Index} = \lfloor \text{Frame Index} / 64 \rfloor$$
$$\text{Bit Offset} = \text{Frame Index} \pmod{64}$$

* Bit value `0`: Frame is free and available for allocation.
* Bit value `1`: Frame is allocated or reserved by hardware/BIOS.

---

## 2. Frame Allocator Implementation (`crates/mem/src/pmm/`)

```rust
pub struct FrameAllocator {
    bitmap: &'static mut [u64],
    total_frames: usize,
    used_frames: usize,
}

impl FrameAllocator {
    /// Allocates a single 4 KiB physical frame.
    pub fn alloc_frame(&mut self) -> Option<usize> {
        for (word_idx, word) in self.bitmap.iter_mut().enumerate() {
            if *word != u64::MAX {
                let bit_idx = (!*word).trailing_zeros() as usize;
                *word |= 1 << bit_idx;
                self.used_frames += 1;
                let frame_idx = word_idx * 64 + bit_idx;
                return Some(frame_idx * 4096);
            }
        }
        None // Out of physical memory (OOM)
    }

    /// Releases an allocated frame back to the pool.
    pub fn free_frame(&mut self, phys_addr: usize) {
        let frame_idx = phys_addr / 4096;
        let word_idx = frame_idx / 64;
        let bit_idx = frame_idx % 64;
        self.bitmap[word_idx] &= !(1 << bit_idx);
        self.used_frames -= 1;
    }
}
```

---

## 3. Reserved Regions

During bootstrap, the following ranges are marked permanently reserved:
1. `0x00000000` -- `0x000FFFFF` (First 1 MiB: Real-Mode IVT, BDA, EBDA, Video RAM).
2. Kernel ELF Code and Data segments.
3. Multiboot2 Information Structure and modules (initrd).
4. Physical frames mapped to the PMM bitmap itself.
