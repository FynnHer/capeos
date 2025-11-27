// src/gdt.rs

use x86_64::VirtAddr;
use x86_64::structures::tss::TaskStateSegment;
use lazy_static::lazy_static;

use x86_64::structures::gdt::{GlobalDescriptorTable, Descriptor};

use x86_64::structures::gdt::SegmentSelector;

// Index of the Interrupt Stack Table entry for double faults
pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;


lazy_static! { // Initialize the Task State Segment (TSS)
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new(); // Create a new TSS
        // Set up the stack for double fault exceptions
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = { // Allocate stack for double fault handler
            const STAKC_SIZE: usize = 4096 * 5; // 20 KB stack
            static mut STACK: [u8; STAKC_SIZE] = [0; STAKC_SIZE]; // Static stack allocation

            let stack_start = VirtAddr::from_ptr(&raw const STACK); // Get starting virtual address of the stack
            let stack_end = stack_start + STAKC_SIZE; // Calculate the end address of the stack
            stack_end // Return the top of the stack (stacks grow downwards)
        };
        tss // Return the initialized TSS
    };
}

lazy_static! {
    // a Global descriptor Table is a data structure used by Intel x86-family processors
    // to define the characteristics of the various memory areas used during program execution,
    // including the base address, the size, and access privileges like executability and writability
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new(); // Create a new GDT
        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment()); // Add kernel code segment
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS)); // Add TSS segment
        (gdt, Selectors { code_selector, tss_selector }) // Return the GDT and selectors
    };
}

struct Selectors {
    code_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}





pub fn init() {
    use x86_64::instructions::tables::load_tss;
    use x86_64::instructions::segmentation::{CS, Segment};

    GDT.0.load(); // Load the GDT
    unsafe {
        CS::set_reg(GDT.1.code_selector);  // Set the code segment register
        load_tss(GDT.1.tss_selector); // Load the TSS
    }
}