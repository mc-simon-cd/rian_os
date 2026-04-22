use x86_64::structures::tss::TaskStateSegment;
use x86_64::structures::gdt::{GlobalDescriptorTable, Descriptor, SegmentSelector};
use lazy_static::lazy_static;
use crate::arch::ArchContext;
use crate::kernel::memory::paging::VirtAddr;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 5;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = ::x86_64::VirtAddr::from_ptr(&raw const STACK);
            let stack_end = stack_start + STACK_SIZE;
            stack_end
        };
        tss
    };
}

lazy_static! {
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        let kernel_code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        let kernel_data_selector = gdt.add_entry(Descriptor::kernel_data_segment());
        let user_data_selector = gdt.add_entry(Descriptor::user_data_segment());
        let user_code_selector = gdt.add_entry(Descriptor::user_code_segment());
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
        CS::set_reg(GDT.1.kernel_code_selector);
        DS::set_reg(GDT.1.kernel_data_selector);
        ES::set_reg(GDT.1.kernel_data_selector);
        SS::set_reg(GDT.1.kernel_data_selector);
        load_tss(GDT.1.tss_selector);
    }
}

pub struct X86_64Context {
    _registers: [u64; 16],
    instruction_pointer: u64,
    stack_pointer: u64,
}

impl ArchContext for X86_64Context {
    fn save(&mut self) {
        // Placeholder for register saving logic
    }

    fn restore(&self) {
        // Placeholder for register restoration
    }

    fn prepare_user_stack(stack_top: VirtAddr) -> Self {
        Self {
            _registers: [0; 16],
            instruction_pointer: 0,
            stack_pointer: stack_top.0,
        }
    }

    fn set_kernel_stack(stack_top: VirtAddr) {
        let tss_ptr = &*TSS as *const TaskStateSegment as *mut TaskStateSegment;
        unsafe {
            (*tss_ptr).privilege_stack_table[0] = ::x86_64::VirtAddr::new(stack_top.0);
        }
    }

    fn enter_user_mode(&self) -> ! {
        unsafe {
            core::arch::asm!(
                "push {user_data_sel}",
                "push {user_stack}",
                "push 0x202",
                "push {user_code_sel}",
                "push {user_rip}",
                "iretq",
                user_data_sel = in(reg) GDT.1.user_data_selector.0 as u64,
                user_stack = in(reg) self.stack_pointer,
                user_code_sel = in(reg) GDT.1.user_code_selector.0 as u64,
                user_rip = in(reg) self.instruction_pointer,
                options(noreturn)
            );
        }
    }
}
