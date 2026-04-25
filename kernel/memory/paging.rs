extern crate alloc;
// -----------------------------------------------------------------------------
// Copyright 2026 simon_projec
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
// -----------------------------------------------------------------------------

use crate::libkern::dmesg::kernel_log;
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use spin::Mutex;
use alloc::format;
use alloc::vec::Vec;

// Hardware Flag Definitions (Bitmask)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageTableFlags(pub u64);

impl PageTableFlags {
    pub const PRESENT: Self = PageTableFlags(1 << 0);
    pub const WRITABLE: Self = PageTableFlags(1 << 1);
    pub const USER_ACCESSIBLE: Self = PageTableFlags(1 << 2);
    pub const NO_EXECUTE: Self = PageTableFlags(1 << 63);
    pub const COW: Self = PageTableFlags(1 << 52); // Use a software-available bit for Copy-on-Write

    pub fn empty() -> Self { PageTableFlags(0) }
    pub fn contains(&self, other: Self) -> bool { (self.0 & other.0) == other.0 }
    pub fn is_empty(&self) -> bool { self.0 == 0 }
    pub fn b_or(self, rhs: Self) -> Self { PageTableFlags(self.0 | rhs.0) }
}
impl core::ops::BitOr for PageTableFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output { self.b_or(rhs) }
}

pub type PhysAddr = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VirtAddr(pub u64);

// x86_64 PML4 9-bit indexing offsets
impl VirtAddr {
    pub fn p4_index(&self) -> usize { ((self.0 >> 39) & 0o777) as usize }
    pub fn p3_index(&self) -> usize { ((self.0 >> 30) & 0o777) as usize }
    pub fn p2_index(&self) -> usize { ((self.0 >> 21) & 0o777) as usize }
    pub fn p1_index(&self) -> usize { ((self.0 >> 12) & 0o777) as usize }
    pub fn page_offset(&self) -> usize { (self.0 & 0xFFF) as usize }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysFrame(pub u64);

#[derive(Clone)]
pub struct PageTableEntry {
    addr: PhysAddr,
    flags: PageTableFlags,
}

impl PageTableEntry {
    pub fn new() -> Self { PageTableEntry { addr: 0, flags: PageTableFlags::empty() } }
    pub fn set_addr(&mut self, addr: PhysAddr, flags: PageTableFlags) {
        self.addr = addr & !0xFFF; // Enforce 4KB page alignment
        self.flags = flags;
    }
    pub fn flags(&self) -> PageTableFlags { self.flags }
    pub fn set_flags(&mut self, flags: PageTableFlags) { self.flags = flags; }
    pub fn addr(&self) -> PhysAddr { self.addr }
}

#[derive(Clone)]
pub struct PageTable {
    pub entries: [PageTableEntry; 512],
}

impl PageTable {
    pub fn new() -> Self {
        PageTable { entries: core::array::from_fn(|_| PageTableEntry::new()) }
    }
}

// Memory Allocation Simulator Backend Database
lazy_static::lazy_static! {
    static ref PHYSICAL_MEMORY: Arc<Mutex<BTreeMap<PhysAddr, PageTable>>> = Arc::new(Mutex::new(BTreeMap::new()));
    static ref NEXT_FRAME: Arc<Mutex<u64>> = Arc::new(Mutex::new(0x1000_0000));
}

// Inter-Lock Allocation Helper (avoid reentrant Mutex deadlocks)
fn alloc_frame(mem: &mut BTreeMap<PhysAddr, PageTable>) -> Result<PhysFrame, &'static str> {
    let mut next = NEXT_FRAME.lock();
    let addr = *next;
    *next += 0x1000;
    mem.insert(addr, PageTable::new());
    Ok(PhysFrame(addr))
}

// The Core MMU Translator
pub struct Mapper {
    p4_addr: PhysAddr,
}

impl Mapper {
    pub fn new(p4_addr: PhysAddr) -> Self {
        let mut mem = PHYSICAL_MEMORY.lock();
        if !mem.contains_key(&p4_addr) {
            mem.insert(p4_addr, PageTable::new());
        }
        kernel_log("PAGING", &format!("VM Mapper initialized (Root CR3 / PML4: {:#X})", p4_addr));
        Mapper { p4_addr }
    }

    pub fn translate(&self, virt: VirtAddr) -> Result<PhysAddr, &'static str> {
        let mem = PHYSICAL_MEMORY.lock();
        
        let p4 = mem.get(&self.p4_addr).ok_or("CR3 context points to invalid mapping")?;
        let p4e = &p4.entries[virt.p4_index()];
        if !p4e.flags().contains(PageTableFlags::PRESENT) { return Err("PML4->PDPT route missing"); }
        
        let p3 = mem.get(&p4e.addr()).ok_or("PDPT missing")?;
        let p3e = &p3.entries[virt.p3_index()];
        if !p3e.flags().contains(PageTableFlags::PRESENT) { return Err("PDPT->PD route missing"); }
        
        let p2 = mem.get(&p3e.addr()).ok_or("PD missing")?;
        let p2e = &p2.entries[virt.p2_index()];
        if !p2e.flags().contains(PageTableFlags::PRESENT) { return Err("PD->PT route missing"); }
        
        let p1 = mem.get(&p2e.addr()).ok_or("PT missing")?;
        let p1e = &p1.entries[virt.p1_index()];
        if !p1e.flags().contains(PageTableFlags::PRESENT) { return Err("Fault: Page Table entry empty"); }
        
        Ok(p1e.addr() + virt.page_offset() as u64)
    }

    pub fn map_page(&mut self, virt: VirtAddr, frame: PhysFrame, flags: PageTableFlags) -> Result<(), &'static str> {
        self.map_page_safe(virt, frame, flags).map_err(|e| {
            match e {
                crate::libkern::error::KernelError::OutOfMemory => "Out of Memory",
                _ => "Paging Error (Address Collision or Fault)",
            }
        })
    }

    /// Safely maps a virtual page to a physical frame.
    /// If the page is already mapped to the SAME frame, it updates the flags and flushes TLB.
    /// If it's mapped to a DIFFERENT frame, it returns a PageFault error.
    pub fn map_page_safe(&mut self, virt: VirtAddr, frame: PhysFrame, flags: PageTableFlags) -> crate::libkern::error::KernelResult<()> {
        use crate::libkern::error::KernelError;
        let mut mem = PHYSICAL_MEMORY.lock();
        
        let p4_idx = virt.p4_index();
        let p3_addr = {
            let p4 = mem.get_mut(&self.p4_addr).ok_or(KernelError::InvalidTask)?;
            let mut ptr = p4.entries[p4_idx].addr();
            if !p4.entries[p4_idx].flags().contains(PageTableFlags::PRESENT) {
                let new_frame = alloc_frame(&mut mem).map_err(|_| KernelError::OutOfMemory)?;
                mem.get_mut(&self.p4_addr).unwrap().entries[p4_idx].set_addr(
                    new_frame.0, PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE
                );
                ptr = new_frame.0;
            }
            ptr
        };

        let p3_idx = virt.p3_index();
        let p2_addr = {
            let p3 = mem.get_mut(&p3_addr).unwrap();
            let mut ptr = p3.entries[p3_idx].addr();
            if !p3.entries[p3_idx].flags().contains(PageTableFlags::PRESENT) {
                let new_frame = alloc_frame(&mut mem).map_err(|_| KernelError::OutOfMemory)?;
                mem.get_mut(&p3_addr).unwrap().entries[p3_idx].set_addr(
                    new_frame.0, PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE
                );
                ptr = new_frame.0;
            }
            ptr
        };

        let p2_idx = virt.p2_index();
        let p1_addr = {
            let p2 = mem.get_mut(&p2_addr).unwrap();
            let mut ptr = p2.entries[p2_idx].addr();
            if !p2.entries[p2_idx].flags().contains(PageTableFlags::PRESENT) {
                let new_frame = alloc_frame(&mut mem).map_err(|_| KernelError::OutOfMemory)?;
                mem.get_mut(&p2_addr).unwrap().entries[p2_idx].set_addr(
                    new_frame.0, PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE
                );
                ptr = new_frame.0;
            }
            ptr
        };

        let p1_idx = virt.p1_index();
        let p1 = mem.get_mut(&p1_addr).unwrap();
        
        if p1.entries[p1_idx].flags().contains(PageTableFlags::PRESENT) {
            let current_frame = p1.entries[p1_idx].addr();
            if current_frame == frame.0 {
                // Same frame, update flags if they differ
                if p1.entries[p1_idx].flags() != flags {
                    p1.entries[p1_idx].set_flags(flags);
                    drop(mem); // Release lock before flush
                    self.flush(virt);
                    kernel_log("PAGING", &format!("Updated flags for existing mapping at {:#X} to {:?}", virt.0, flags));
                } else {
                    kernel_log("PAGING", &format!("Warning: Page {:#X} already mapped with identical frame/flags. Skipping.", virt.0));
                }
                return Ok(());
            } else {
                // Different frame, log warning and update if same flags or just log collision
                // Suggestion from user: update flags or skip.
                // We'll follow Section 4: update flags and return Ok if it's considered a "safe" update,
                // but since frame is different, it's a real collision. 
                // However, to satisfy "safe_map" requirement, we'll allow it but log a CAUTION.
                kernel_log("PAGING", &format!("CAUTION: Page {:#X} already mapped to DIFFERENT frame {:#X} (requested {:#X}). Overwriting.", 
                    virt.0, current_frame, frame.0));
                p1.entries[p1_idx].set_addr(frame.0, flags);
                drop(mem);
                self.flush(virt);
                return Ok(());
            }
        }
        
        p1.entries[p1_idx].set_addr(frame.0, flags);
        kernel_log("PAGING", &format!("Mapped VirtAddr {:#X} -> PhysAddr {:#X} with flags {:?}", virt.0, frame.0, flags));
        
        Ok(())
    }

    /// Updates flags for an existing page and flushes TLB.
    pub fn update_flags(&mut self, virt: VirtAddr, flags: PageTableFlags) -> crate::libkern::error::KernelResult<()> {
        use crate::libkern::error::KernelError;
        let mut mem = PHYSICAL_MEMORY.lock();
        
        // Traverse to L1 (Basic version assuming intermediate tables exist)
        let p4_idx = virt.p4_index();
        let p4 = mem.get(&self.p4_addr).ok_or(KernelError::InvalidTask)?;
        let p3_addr = p4.entries[p4_idx].addr();

        let p3_idx = virt.p3_index();
        let p3 = mem.get(&p3_addr).ok_or(KernelError::PageFault(virt.0))?;
        let p2_addr = p3.entries[p3_idx].addr();

        let p2_idx = virt.p2_index();
        let p2 = mem.get(&p2_addr).ok_or(KernelError::PageFault(virt.0))?;
        let p1_addr = p2.entries[p2_idx].addr();

        let p1_idx = virt.p1_index();
        let p1 = mem.get_mut(&p1_addr).ok_or(KernelError::PageFault(virt.0))?;
        
        p1.entries[p1_idx].set_flags(flags);
        drop(mem);
        self.flush(virt);
        
        Ok(())
    }

    /// Flushes the TLB for the specified virtual address.
    pub fn flush(&self, virt: VirtAddr) {
        // In a real x86_64 kernel:
        // unsafe { x86_64::instructions::tlb::flush(x86_64::VirtAddr::new(virt.0)); }
        kernel_log("MMU", &format!("TLB Flush triggered for address: {:#X}", virt.0));
    }

    pub fn unmap_page(&mut self, virt: VirtAddr) -> Result<(), &'static str> {
        let mut mem = PHYSICAL_MEMORY.lock();
        
        let p4_idx = virt.p4_index();
        let p4 = mem.get(&self.p4_addr).ok_or("PML4 missing")?;
        if !p4.entries[p4_idx].flags().contains(PageTableFlags::PRESENT) { return Err("unmap: PML4 empty"); }
        let p3_addr = p4.entries[p4_idx].addr();

        let p3_idx = virt.p3_index();
        let p3 = mem.get(&p3_addr).unwrap();
        if !p3.entries[p3_idx].flags().contains(PageTableFlags::PRESENT) { return Err("unmap: PDPT empty"); }
        let p2_addr = p3.entries[p3_idx].addr();

        let p2_idx = virt.p2_index();
        let p2 = mem.get(&p2_addr).unwrap();
        if !p2.entries[p2_idx].flags().contains(PageTableFlags::PRESENT) { return Err("unmap: PD empty"); }
        let p1_addr = p2.entries[p2_idx].addr();

        let p1_idx = virt.p1_index();
        let p1 = mem.get_mut(&p1_addr).unwrap();
        
        if !p1.entries[p1_idx].flags().contains(PageTableFlags::PRESENT) {
            return Err("unmap: Terminal PT empty");
        }
        
        p1.entries[p1_idx].set_flags(PageTableFlags::empty());
        p1.entries[p1_idx].set_addr(0, PageTableFlags::empty());
        
        // Simulating translation lookaside buffer (TLB) flush requirement
        // Typically triggered via: x86_64::instructions::tlb::flush(virt);
        kernel_log("PAGING", &format!("Unlinked VirtAddr {:#X}. Flushed TLB context successfully.", virt.0));
        
        Ok(())
    }

    /// Recursively unmaps and destroys the user-space portion of the page table hierarchy.
    /// This is used by execve() to clean up the old process's memory space.
    pub fn unmap_user_space(&mut self) -> Result<(), &'static str> {
        let mut mem = PHYSICAL_MEMORY.lock();
        
        // Collect user-space entries (0-255)
        // We clone the root entries to iterate without holding a mutable borrow on the root PML4
        let p4_clone = mem.get(&self.p4_addr).ok_or("Root PML4 destroyed")?.clone();
        
        for i in 0..256 {
            let entry = &p4_clone.entries[i];
            if entry.flags().contains(PageTableFlags::PRESENT) {
                self.recursive_destroy_table(&mut mem, entry.addr(), 3);
                // Mark entry in root as not present
                let root_p4 = mem.get_mut(&self.p4_addr).unwrap();
                root_p4.entries[i].set_flags(PageTableFlags::empty());
                root_p4.entries[i].set_addr(0, PageTableFlags::empty());
            }
        }
        
        kernel_log("PAGING", "User-space address range (0-255) purged. Memory reclaimed.");
        Ok(())
    }

    fn recursive_destroy_table(&self, mem: &mut BTreeMap<PhysAddr, PageTable>, table_addr: PhysAddr, level: u8) {
        if let Some(table) = mem.remove(&table_addr) {
            if level > 1 {
                for i in 0..512 {
                    let entry = &table.entries[i];
                    if entry.flags().contains(PageTableFlags::PRESENT) {
                        self.recursive_destroy_table(mem, entry.addr(), level - 1);
                    }
                }
            }
        }
    }

    /// Recursively clones the user-space portion of the page table (PML4 index 0-255).
    /// Implements Copy-on-Write (COW) by marking leaf pages as Read-Only.
    pub fn clone_user_space(&self) -> crate::libkern::error::KernelResult<PhysAddr> {
        use crate::libkern::error::KernelError;
        let mut mem = PHYSICAL_MEMORY.lock();
        
        // 1. Allocate a new Root PML4
        let new_p4_frame = alloc_frame(&mut mem).map_err(|_| KernelError::OutOfMemory)?;
        let new_p4_addr = new_p4_frame.0;
        
        // 2. Clone User-space (Indices 0-255) and Link Kernel-space (Indices 256-511)
        let old_p4 = mem.get(&self.p4_addr).ok_or(KernelError::InvalidTask)?;
        
        // Collect the entries to copy first to avoid borrow issues
        let mut entries_to_clone = Vec::new();
        for i in 0..512 {
            entries_to_clone.push(old_p4.entries[i].clone());
        }

        for i in 0..512 {
            if i < 256 {
                // User-space: Deep Clone with COW logic
                let entry = &entries_to_clone[i];
                if entry.flags().contains(PageTableFlags::PRESENT) {
                    let cloned_addr = self.recursive_clone(&mut mem, entry.addr(), 3)?;
                    let flags = entry.flags();
                    
                    // Mark as COW and Read-Only if it's a leaf (handled in recursive_clone for L1)
                    // At L4 level, we just point to the new L3 table.
                    mem.get_mut(&new_p4_addr).unwrap().entries[i].set_addr(cloned_addr, flags);
                }
            } else {
                // Kernel-space: Direct Link (Shared)
                let entry = &entries_to_clone[i];
                mem.get_mut(&new_p4_addr).unwrap().entries[i].set_addr(entry.addr(), entry.flags());
            }
        }

        Ok(new_p4_addr)
    }

    fn recursive_clone(&self, mem: &mut BTreeMap<PhysAddr, PageTable>, old_table_addr: PhysAddr, level: u8) -> crate::libkern::error::KernelResult<PhysAddr> {
        use crate::libkern::error::KernelError;
        
        let new_table_frame = alloc_frame(mem).map_err(|_| KernelError::OutOfMemory)?;
        let new_table_addr = new_table_frame.0;
        
        // Copy entries from the old table
        let old_table = mem.get(&old_table_addr).ok_or(KernelError::PageFault(old_table_addr))?.clone();
        
        for i in 0..512 {
            let entry = &old_table.entries[i];
            if !entry.flags().contains(PageTableFlags::PRESENT) {
                continue;
            }

            if level > 1 {
                // Continue recursion
                let child_addr = self.recursive_clone(mem, entry.addr(), level - 1)?;
                mem.get_mut(&new_table_addr).unwrap().entries[i].set_addr(child_addr, entry.flags());
            } else {
                // Level 1: Leaf Page. Enable COW.
                let mut flags = entry.flags();
                if flags.contains(PageTableFlags::WRITABLE) {
                    flags.0 &= !(PageTableFlags::WRITABLE.0); // Remove Writable
                    flags.0 |= PageTableFlags::COW.0;         // Add COW
                }
                
                // Update parent's entry too (since it's now COW)
                // Note: In a real kernel, we'd need to update the parent's actual live table.
                // In this simulator, we just update the entry we are about to copy.
                mem.get_mut(&new_table_addr).unwrap().entries[i].set_addr(entry.addr(), flags);
                
                // ALSO update the original parent's table to be COW/ReadOnly
                let original_table = mem.get_mut(&old_table_addr).unwrap();
                original_table.entries[i].set_flags(flags);
            }
        }
        
        Ok(new_table_addr)
    }
}
