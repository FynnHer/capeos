// src/memory.rs

use x86_64::{
    structures::paging::PageTable,
    VirtAddr,
    PhysAddr,
};

// Returns mutable reference to the active lvl 4 page table 
// The level 4 page table is the root of the paging hierarchy in x86_64 architecture

// unsafe function because the caller must guarantee that the complete physical memory
// is mapped to virtual memory at the passed offset. functoin must only be called once
// to avoid aliasing mutable references (which is undefined behavior in Rust)

pub unsafe fn active_level_4_table(physical_memory_offset: VirtAddr)
    -> &'static mut PageTable
{
    // Cr3 register holds the physical address of the active level 4 page table
    use x86_64::registers::control::Cr3;
    // read return the physical frame of the level 4 page table and some flags
    let (level_4_table_frame, _) = Cr3::read();

    // get the physical address
    let phys = level_4_table_frame.start_address();
    // calculate the virtual address by adding the physical memory offset
    let virt = physical_memory_offset + phys.as_u64();
    // convert the virtual address to a mutable pointer to a PageTable
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    // dereference the pointer to get a mutable reference to the PageTable
    unsafe { &mut *page_table_ptr }
}

// translate a given virtual addresss to the mapped physical address or None if not mapped

// this function is unsafe because the caller must guarantee that hte complete physical memory
// is mapped to virtual memory at the passed offset

pub unsafe fn translate_addr(addr: VirtAddr, physical_memory_offset: VirtAddr)
    -> Option<PhysAddr>
    {
        translate_addr_inner(addr, physical_memory_offset)
    }

// private function that is called by 'translate_addr' to limit the scope of unsafe

// this function is safe to limit the scope of unsafe bc rust treats
// the whole body of unsafe functions as unsafe. this function must only be reachable
// through "unsafe fn" from outside this module

fn translate_addr_inner(addr: VirtAddr, physical_memory_offset: VirtAddr)
    -> Option<PhysAddr>
{
    use x86_64::structures::paging::page_table::FrameError;
    use x86_64::registers::control::Cr3;

    // read the active lvl 4 frame from the cr3 register
    let (level_4_table_frame, _) = Cr3::read();

    let table_indexes = [
        addr.p4_index(),
        addr.p3_index(),
        addr.p2_index(),
        addr.p1_index(),
    ];
    let mut frame = level_4_table_frame;

    // traverse the multi-level page table
    for &index in &table_indexes {
        // convert the frame into a page table refernce
        let virt = physical_memory_offset + frame.start_address().as_u64();
        let table_ptr: *const PageTable = virt.as_ptr();
        let table = unsafe { &*table_ptr };

        // read the page table entry at the given index
        let entry = &table[index];
        // update the frame
        frame = match entry.frame() {
            Ok(frame) => frame,
            Err(FrameError::FrameNotPresent) => return None,
            Err(FrameError::HugeFrame) => panic!("huge pages not supported"),
        };
    }

    // calculate the physical address by adding the page offset
    Some(frame.start_address() +u64::from(addr.page_offset()))
}