// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Structural consistency verification between active VMAs and hardware page table state.

use super::super::table::{
    get_pte_in_pml4, PAGE_NO_EXECUTE, PAGE_PRESENT, PAGE_USER, PAGE_WRITABLE,
};
#[cfg(not(test))]
use super::super::table::{PAGE_HUGE, PTE_ADDR_MASK};
use super::descriptor::{MAX_VMAS, PROT_EXEC, PROT_WRITE};
#[cfg(not(test))]
use super::descriptor::{MMAP_END, MMAP_START};
use super::table::VMA_TABLE;
use crate::pmm;

/// Verifies that all active VMAs match underlying page table permissions and flags.
///
/// Ensures:
/// 1. Forward consistency: VMA permissions (read, write, execute) reflect exactly in leaf PTEs.
/// 2. Reverse consistency: No orphan mapped pages exist within user MMAP space without an enclosing VMA.
///
/// # Safety
///
/// Traverses raw physical memory page tables based on `pml4_phys`.
pub unsafe fn verify_vma_pte_invariants(pml4_phys: u64) -> Result<(), &'static str> {
    if pml4_phys == 0 {
        return Ok(());
    }

    for i in 0..MAX_VMAS {
        let vma = VMA_TABLE[i];
        if vma.is_active && vma.pml4_phys == pml4_phys {
            let mut vaddr = vma.start;
            while vaddr < vma.end {
                if let Some(pte) = get_pte_in_pml4(pml4_phys, vaddr) {
                    if (pte & PAGE_PRESENT) != 0 {
                        if (pte & PAGE_USER) == 0 {
                            return Err("VMA invariant violation: PTE missing PAGE_USER flag");
                        }
                        if (vma.prot & PROT_WRITE) != 0 && (pte & PAGE_WRITABLE) == 0 {
                            return Err(
                                "VMA invariant violation: Writable VMA has non-writable PTE",
                            );
                        }
                        if (vma.prot & PROT_WRITE) == 0 && (pte & PAGE_WRITABLE) != 0 {
                            return Err("VMA invariant violation: Read-only VMA has writable PTE");
                        }
                        if (vma.prot & PROT_EXEC) == 0 && (pte & PAGE_NO_EXECUTE) == 0 {
                            return Err(
                                "VMA invariant violation: Non-executable VMA has executable PTE",
                            );
                        }
                    }
                }
                vaddr += pmm::PAGE_SIZE;
            }
        }
    }

    #[cfg(not(test))]
    {
        let pml4 = pml4_phys as *const u64;

        for pml4_idx in 1..256 {
            let pml4_entry = *pml4.add(pml4_idx);
            if (pml4_entry & PAGE_PRESENT) == 0 {
                continue;
            }
            let pdpt_phys = pml4_entry & PTE_ADDR_MASK;
            let pdpt = pdpt_phys as *const u64;

            for pdpt_idx in 0..512 {
                let pdpt_entry = *pdpt.add(pdpt_idx);
                if (pdpt_entry & PAGE_PRESENT) == 0 {
                    continue;
                }
                if (pdpt_entry & PAGE_HUGE) != 0 {
                    let page_start = ((pml4_idx as u64) << 39) | ((pdpt_idx as u64) << 30);
                    let page_end = page_start + 0x4000_0000;
                    if page_start < MMAP_END && page_end > MMAP_START {
                        let mut in_vma = false;
                        for i in 0..MAX_VMAS {
                            let v = VMA_TABLE[i];
                            if v.is_active
                                && v.pml4_phys == pml4_phys
                                && page_start < v.end
                                && page_end > v.start
                            {
                                in_vma = true;
                                break;
                            }
                        }
                        if !in_vma {
                            return Err("VMA invariant violation: Orphan 1GB huge page overlaps mmap region without active VMA");
                        }
                    }
                    continue;
                }

                let pd_phys = pdpt_entry & PTE_ADDR_MASK;
                let pd = pd_phys as *const u64;

                for pd_idx in 0..512 {
                    let pd_entry = *pd.add(pd_idx);
                    if (pd_entry & PAGE_PRESENT) == 0 {
                        continue;
                    }
                    if (pd_entry & PAGE_HUGE) != 0 {
                        let page_start = ((pml4_idx as u64) << 39)
                            | ((pdpt_idx as u64) << 30)
                            | ((pd_idx as u64) << 21);
                        let page_end = page_start + 0x20_0000;
                        if page_start < MMAP_END && page_end > MMAP_START {
                            let mut in_vma = false;
                            for i in 0..MAX_VMAS {
                                let v = VMA_TABLE[i];
                                if v.is_active
                                    && v.pml4_phys == pml4_phys
                                    && page_start < v.end
                                    && page_end > v.start
                                {
                                    in_vma = true;
                                    break;
                                }
                            }
                            if !in_vma {
                                return Err("VMA invariant violation: Orphan 2MB huge page overlaps mmap region without active VMA");
                            }
                        }
                        continue;
                    }

                    let pt_phys = pd_entry & PTE_ADDR_MASK;
                    let pt = pt_phys as *const u64;

                    for pt_idx in 0..512 {
                        let pt_entry = *pt.add(pt_idx);
                        if (pt_entry & PAGE_PRESENT) != 0 && (pt_entry & PAGE_USER) != 0 {
                            let page_start = ((pml4_idx as u64) << 39)
                                | ((pdpt_idx as u64) << 30)
                                | ((pd_idx as u64) << 21)
                                | ((pt_idx as u64) << 12);
                            let page_end = page_start + pmm::PAGE_SIZE;
                            if page_start < MMAP_END && page_end > MMAP_START {
                                let mut in_vma = false;
                                for i in 0..MAX_VMAS {
                                    let v = VMA_TABLE[i];
                                    if v.is_active
                                        && v.pml4_phys == pml4_phys
                                        && page_start < v.end
                                        && page_end > v.start
                                    {
                                        in_vma = true;
                                        break;
                                    }
                                }
                                if !in_vma {
                                    return Err("VMA invariant violation: Orphan PTE exists in mmap region without active VMA");
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
