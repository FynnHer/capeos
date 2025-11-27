// main.rs

#![no_std] // disable the standard library
#![no_main] // disable rust level entry points

// Enable custom test framework feature
#![feature(custom_test_frameworks)]
#![test_runner(capeos::test_runner)]

#![reexport_test_harness_main = "test_main"]


use core::panic::PanicInfo;
use capeos::println;
use bootloader::{BootInfo, entry_point};


/// This function is called on panic.
/// Panic handler for not test mode
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    capeos::hlt_loop();
}
/// Panic handler for test mode
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    capeos::test_panic_handler(info)
}

// Entry point for the OS
// Define the entry point for the bootloader bc _start doesnt verify the signature of boot_info
entry_point!(kernel_main);
// previous #[unsafe(no_mangle)] pub extern "C" fn _start

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    // Entry point of the program
    use capeos::memory::active_level_4_table;
    use x86_64::VirtAddr;
    use x86_64::structures::paging::PageTable;
    use capeos::memory::translate_addr;

    println!("Hello CapeOS{}", "!");

    capeos::init(); // initialize interrupts via lib.rs (calling init_idt etc from interrupts.rs)
    /* 
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let l4_table = unsafe { active_level_4_table(phys_mem_offset) };

    for ( i , entry ) in l4_table.iter().enumerate() {
        if !entry.is_unused() {
            println!("L4 Entry {}: {:?}", i, entry);

            // look at the l3 page table
            let phys = entry.frame().unwrap().start_address();
            let virt = boot_info.physical_memory_offset + phys.as_u64();
            let ptr = VirtAddr::new(virt).as_mut_ptr();
            let l3_table: &PageTable = unsafe { &*ptr };

            for ( j, entry) in l3_table.iter().enumerate() {
                if !entry.is_unused() {
                    println!("  L3 Entry {}: {:?}", j, entry);
                }
            }

        }
    }
    */

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);

    let addresses = [
        // the identity mapped vga buffer page
        0xb8000,
        // some code page
        0x201008,
        // some stack page
        0x100_0020_1a10,
        // vir addr mapped to physical addr 0
        boot_info.physical_memory_offset,
    ];

    for &address in &addresses {
        let virt = VirtAddr::new(address);
        let phys = unsafe { translate_addr(virt, phys_mem_offset)};
        println!("{:?} -> {:?}", virt, phys);
    }

    #[cfg(test)]
    test_main();

    println!("It did not crash!");
    capeos::hlt_loop();
} 
