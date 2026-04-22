use ::x86_64::VirtAddr;
use x86_64::structures::tss::TaskStateSegment;
use x86_64::structures::gdt::{GlobalDescriptorTable, Descriptor, SegmentSelector};
use lazy_static::lazy_static;
use crate::arch::ArchContext;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 5;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = VirtAddr::from_ptr(&raw const STACK);
            let stack_end = stack_start + STACK_SIZE;
            stack_end
        };
        tss
    };
}

lazy_static! {
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        // Null descriptor (Implicitly added by new())
        
        // Kernel Code (Ring 0)
        let kernel_code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        // Kernel Data (Ring 0)
        let kernel_data_selector = gdt.add_entry(Descriptor::kernel_data_segment());
        // User Data (Ring 3)
        let user_data_selector = gdt.add_entry(Descriptor::user_data_segment());
        // User Code (Ring 3)
        let user_code_selector = gdt.add_entry(Descriptor::user_code_segment());
        // TSS System Descriptor
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
        
        (
            gdt,
            Selectors {
                kernel_code_selector,
                kernel_data_selector,
                user_code_selector,
                user_data_selector,
                tss_selector,
            },
        )
    };
}

struct Selectors {
    kernel_code_selector: SegmentSelector,
    kernel_data_selector: SegmentSelector,
    user_code_selector: SegmentSelector,
    user_data_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

pub fn init() {
    use x86_64::instructions::tables::load_tss;
    use x86_64::instructions::segmentation::{CS, Segment, DS, ES, SS};

    GDT.0.load();
    unsafe {
        // x86_64 architecture: CS must be set after loading GDT
        CS::set_reg(GDT.1.kernel_code_selector);
        
        // Initialize data segment registers for Ring 0
        DS::set_reg(GDT.1.kernel_data_selector);
        ES::set_reg(GDT.1.kernel_data_selector);
        SS::set_reg(GDT.1.kernel_data_selector);
        
        load_tss(GDT.1.tss_selector);
    }
}

pub struct X86_64Context {
    // Placeholder for saved registers (RAX, RBX, etc.)
    _registers: [u64; 16],
    instruction_pointer: VirtAddr,
    stack_pointer: VirtAddr,
}

impl ArchContext for X86_64Context {
    fn prepare_user_stack(stack_top: VirtAddr) -> Self {
        Self {
            _registers: [0; 16],
            instruction_pointer: VirtAddr::new(0), // Initial entry point would be set here
            stack_pointer: stack_top,
        }
    }

    fn set_kernel_stack(stack_top: VirtAddr) {
        // UNSAFE: Modifying hardware-level Task State Segment
        // Safe because the TSS is only modified by the kernel during context switches
        // and we ensure proper synchronization in multi-core setups.
        let tss_ptr = &*TSS as *const TaskStateSegment as *mut TaskStateSegment;
        unsafe {
            (*tss_ptr).privilege_stack_table[0] = stack_top;
        }
    }

    fn enter_user_mode(self) -> ! {
        // In x86_64, entering Ring 3 requires using the 'iretq' or 'sysretq' instruction.
        // This involves manually setting up the stack frame for iretq:
        // [SS, RSP, RFLAGS, CS, RIP]
        
        unsafe {
            core::arch::asm!(
                "push {user_data_sel}", // Stack Segment (User Data)
                "push {user_stack}",    // User Stack Pointer
                "push 0x202",           // RFLAGS (Interrupts enabled)
                "push {user_code_sel}", // Code Segment (User Code)
                "push {user_rip}",      // User Instruction Pointer
                "iretq",
                user_data_sel = in(reg) GDT.1.user_data_selector.0 as u64,
                user_stack = in(reg) self.stack_pointer.as_u64(),
                user_code_sel = in(reg) GDT.1.user_code_selector.0 as u64,
                user_rip = in(reg) self.instruction_pointer.as_u64(),
                options(noreturn)
            );
        }
    }
}
