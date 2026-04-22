extern crate alloc;
use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;
use crate::kernel::memory::paging::{Mapper, VirtAddr, PageTableFlags, PhysFrame};
use crate::libkern::dmesg::kernel_log;

pub const USER_STACK_TOP: u64 = 0x0000_7FFF_FFFF_F000;
pub const USER_STACK_SIZE: usize = 16 * 1024; // 16 KB (4 pages)
pub const PAGE_SIZE: usize = 4096;

pub struct UserStack {
    pub top: VirtAddr,
    pub size: usize,
}

/// Allocates and maps a user-mode stack in the current address space.
pub fn allocate_user_stack(mapper: &mut Mapper) -> Result<UserStack, &'static str> {
    let stack_bottom = USER_STACK_TOP - USER_STACK_SIZE as u64;
    
    // x86_64 Paging flags for a secure user-mode stack:
    // PRESENT: Page is active.
    // WRITABLE: Stack needs to be writable.
    // USER_ACCESSIBLE: Memory must be accessible from Ring 3.
    // NO_EXECUTE: Prevent execution of code on the stack (NX bit).
    let flags = PageTableFlags::PRESENT 
        | PageTableFlags::WRITABLE 
        | PageTableFlags::USER_ACCESSIBLE 
        | PageTableFlags::NO_EXECUTE;

    kernel_log("PROC", &format!("Allocating 16KB User Stack at {:#X}", USER_STACK_TOP));

    // Allocate 4 physical frames and map them to the stack's virtual address range
    for i in 0..(USER_STACK_SIZE / PAGE_SIZE) {
        let virt = VirtAddr(stack_bottom + (i * PAGE_SIZE) as u64);
        
        // Simulating physical frame allocation. 
        let frame = PhysFrame(0x6000_0000 + (i * PAGE_SIZE) as u64); 
        
        // Use map_page_safe to ensure we don't clobber existing memory
        mapper.map_page_safe(virt, frame, flags).map_err(|_| "Failed to map stack safely")?;
    }

    Ok(UserStack {
        top: VirtAddr(USER_STACK_TOP),
        size: USER_STACK_SIZE,
    })
}

/// Prepares the user stack according to the C ABI (System V x86_64).
/// 
/// Layout from top to bottom:
/// - Environment strings (null-terminated)
/// - Argument strings (null-terminated)
/// - Environment pointers (ending with NULL)
/// - Argument pointers (ending with NULL)
/// - argc (number of arguments)
pub fn prepare_user_stack(stack: &UserStack, args: Vec<String>, envs: Vec<String>) -> Result<VirtAddr, &'static str> {
    let mut sp = stack.top.0;

    // 1. Push environment strings
    let mut env_ptrs = Vec::new();
    for env in envs.iter().rev() {
        let bytes = env.as_bytes();
        sp -= (bytes.len() + 1) as u64; // +1 for null terminator
        /* In a real kernel: write_to_virt(sp, bytes); write_to_virt(sp + bytes.len(), 0); */
        env_ptrs.push(sp);
    }

    // 2. Push argument strings
    let mut arg_ptrs = Vec::new();
    for arg in args.iter().rev() {
        let bytes = arg.as_bytes();
        sp -= (bytes.len() + 1) as u64; // +1 for null terminator
        /* In a real kernel: write_to_virt(sp, bytes); write_to_virt(sp + bytes.len(), 0); */
        arg_ptrs.push(sp);
    }

    // 3. Align stack to 8 bytes for pointers
    sp &= !0x7;

    // 4. Push envp pointers (NULL terminated)
    sp -= 8; // NULL
    /* In a real kernel: write_to_virt(sp, 0u64); */
    
    for _ptr in env_ptrs {
        sp -= 8;
        /* In a real kernel: write_to_virt(sp, ptr); */
    }
    let _envp = sp;

    // 5. Push argv pointers (NULL terminated)
    sp -= 8; // NULL
    /* In a real kernel: write_to_virt(sp, 0u64); */
    
    for _ptr in arg_ptrs {
        sp -= 8;
        /* In a real kernel: write_to_virt(sp, ptr); */
    }
    let argv = sp;

    // 6. Push argc (as a 64-bit value to maintain alignment)
    sp -= 8;
    /* In a real kernel: write_to_virt(sp, args.len() as u64); */

    // 7. Ensure 16-byte alignment before jumping to user mode (Required by ABI)
    sp &= !0xF;

    kernel_log("PROC", &format!("User Stack prepared. Final SP: {:#X}, argc: {}, argv: {:#X}", 
        sp, args.len(), argv));

    Ok(VirtAddr(sp))
}
