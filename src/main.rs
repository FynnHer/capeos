// main.rs

#![no_std] // disable the standard library
#![no_main] // disable rust level entry points

// Enable custom test framework feature
#![feature(custom_test_frameworks)]
#![test_runner(capeos::test_runner)]

#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use alloc::{boxed::Box, vec, vec::Vec, rc::Rc};
use core::panic::PanicInfo;
use capeos::{println, task::{keyboard, executor::Executor}};
use bootloader::{BootInfo, entry_point};
use capeos::task::{Task,};

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
    use capeos::memory;
    use capeos::memory::BootInfoFrameAllocator;
    use capeos::allocator;
    use x86_64::{VirtAddr};

    println!("Hello CapeOS{}", "!");

    capeos::init(); // initialize interrupts via lib.rs (calling init_idt etc from interrupts.rs)
    

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    // init a mapper
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe {
        BootInfoFrameAllocator::init(&boot_info.memory_map)
    };

    // initialize the heap
    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("heap initialization failed");


    let heap_value = Box::new(41);
    println!("heap_value at {:p}", heap_value);

    // create a dynamically sized vector
    let mut vec = Vec::new();
    for i in 0..500 {
        vec.push(i);
    }
    println!("vec at {:p}", vec.as_slice());

    // create a reference counted vector
    let reference_counted = Rc::new(vec![1, 2, 3]);
    let cloned_reference = reference_counted.clone();
    println!("current reference count is {}", Rc::strong_count(&cloned_reference));
    core::mem::drop(reference_counted);
    println!("reference count is {} now", Rc::strong_count(&cloned_reference));

    let mut executor = Executor::new();
    executor.spawn(Task::new(example_task()));
    executor.spawn(Task::new(keyboard::print_keypresses()));
    #[cfg(test)]
    executor.spawn(Task::new(invoke_test_main()));
    executor.run();

    
    //#[cfg(test)]
    //test_main();

    println!("It did not crash!");
    //capeos::hlt_loop();
} 

#[cfg(test)]
async fn invoke_test_main() {
    test_main();
}

async fn async_number() -> u32 {
    42
}

async fn example_task() {
    let number = async_number().await;
    println!("async number: {}", number);
}