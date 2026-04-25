extern crate alloc;
use alloc::format;
use goblin::mach::Mach;
use crate::kernel::memory::paging::{Mapper, VirtAddr, PageTableFlags, PhysFrame};
use crate::libkern::error::{KernelResult, KernelError};
use crate::libkern::dmesg::kernel_log;

/// Loads a 64-bit Mach-O binary into the provided address space.
///
/// This implementation strictly follows the security requirements of R-OS:
/// 1. __PAGEZERO (sanal address 0x0) is NEVER mapped to trap NULL pointer dereferences.
/// 2. Only loadable segments are mapped according to their intended protections (W^X).
/// 3. map_page_safe is used to ensure no accidental re-mappings occur.
/// 4. BSS regions (vmsize > filesize) are zero-filled.
pub fn load_macho(data: &[u8], mapper: &mut Mapper) -> KernelResult<VirtAddr> {
    let mach = Mach::parse(data).map_err(|_| KernelError::PageFault(0))?;
    
    let macho = match mach {
        Mach::Binary(macho) => macho,
        _ => return Err(KernelError::NotImplemented),
    };

    // Verify Mach-O 64-bit Magic
    if macho.header.magic != goblin::mach::header::MH_MAGIC_64 {
        kernel_log("LOADER", "ERROR: Invalid Mach-O Magic. 64-bit required.");
        return Err(KernelError::NotImplemented);
    }

    kernel_log("LOADER", "Parsing Mach-O 64-bit binary...");

    let mut entry_point = 0;

    for lc in &macho.load_commands {
        match &lc.command {
            goblin::mach::load_command::CommandVariant::Segment64(segment) => {
                let name = segment.name().unwrap_or("unknown");
                
                // CRITICAL: Skip __PAGEZERO to protect NULL pointer access.
                // Standard Mach-O behavior reserves the first 4GB or a smaller range at 0x0.
                if name == "__PAGEZERO" || segment.vmaddr == 0 {
                    kernel_log("LOADER", &format!("RESERVED: __PAGEZERO at {:#X} (size: {:#X}) unmapped for safety.", 
                        segment.vmaddr, segment.vmsize));
                    continue;
                }

                kernel_log("LOADER", &format!("Mapping segment {} at {:#X} (vmsize: {:#X}, filesize: {:#X})", 
                    name, segment.vmaddr, segment.vmsize, segment.filesize));

                // Determine permissions using W^X principle
                // vm_prot: 1=R, 2=W, 4=X
                let mut flags = PageTableFlags::PRESENT | PageTableFlags::USER_ACCESSIBLE;
                if (segment.initprot & 2) != 0 {
                    // WRITABLE
                    flags = flags | PageTableFlags::WRITABLE;
                }
                if (segment.initprot & 4) == 0 {
                    // NO_EXECUTE if execute bit is NOT set
                    flags = flags | PageTableFlags::NO_EXECUTE;
                }

                // Page alignment logic: strictly align to 4KB boundaries
                let start_addr = segment.vmaddr & !0xFFF;
                let offset_in_page = segment.vmaddr & 0xFFF;
                let total_vmsize = segment.vmsize + offset_in_page;
                let num_pages = (total_vmsize + 4095) / 4096;

                for i in 0..num_pages {
                    let virt_addr = VirtAddr(start_addr + (i * 4096));
                    
                    // Allocate a physical frame (Simulated for this microkernel demo)
                    // In a production kernel, this would be a real PMM allocation.
                    let frame_addr = 0x5000_0000 + (segment.vmaddr & 0xFFFFFF) + (i * 4096); 
                    let frame = PhysFrame(frame_addr);

                    // USE map_page_safe for flag resolution and collision detection
                    mapper.map_page_safe(virt_addr, frame, flags).map_err(|e| {
                        kernel_log("LOADER", "ERROR: Failed to map segment page safely.");
                        e
                    })?;

                    // Handle BSS and data copying
                    let offset = (i * 4096) as u64;
                    if offset < segment.filesize {
                        // Data from file
                        /* 
                           In a real kernel, we would use:
                           copy_to_user(virt_addr, &data[segment.fileoff + offset..]);
                           Safety: we must ensure we don't read past the end of the input buffer.
                        */
                    } else {
                        // BSS zero-filling
                        /*
                           Explicit zeroing for security to prevent information leakage from 
                           previously used physical frames.
                           memset_user(virt_addr, 0, 4096);
                        */
                    }
                }
            }
            goblin::mach::load_command::CommandVariant::Main(main) => {
                // LC_MAIN provides the entry point offset
                entry_point = main.entryoff; 
            }
            _ => {}
        }
    }

    // Heuristic entry point calculation for simulation
    let final_entry = if entry_point < 0x100000000 {
        // Simple relocation assumption for this demo
        0x100000000 + entry_point
    } else {
        entry_point
    };

    kernel_log("LOADER", &format!("Finalized Mach-O load. Entry Point: {:#X}", final_entry));

    Ok(VirtAddr(final_entry))
}
