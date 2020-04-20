use x86_64::VirtAddr;
use x86_64::structures::tss::TaskStateSegment;
use x86_64::structures::gdt::{GlobalDescriptorTable, Descriptor, SegmentSelector};
use lazy_static::lazy_static;

// set up GDT, with the TSS and kernel_code_segment entries
lazy_static! {
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
        (gdt, Selectors { code_selector, tss_selector })
    };
}

// used to access the code_selector and tss_selector variables
// in the init function
struct Selectors {
    code_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

// load our GDT, reloading the cs segment register and loading our new TSS
pub fn init() {
    use x86_64::instructions::segmentation::set_cs;
    use x86_64::instructions::tables::load_tss;

    GDT.0.load();
    unsafe {
        set_cs(GDT.1.code_selector);
        load_tss(GDT.1.tss_selector);
    }
}

// set index for Double Fault Interrupt Stack Table
// zero was chosen here arbitrarily
pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

// set up our Task State Segment with our
// Double Fault Interrupt Stack Table, needed
// to help prevent Triple Faults during Stack Overflows
lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = VirtAddr::from_ptr(unsafe{ &STACK });
            let stack_end = stack_start + STACK_SIZE;
            stack_end
        };
        tss
    };
}