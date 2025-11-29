// src/allocator/fixed_size_block.rs

/// Imports
/// -----------
use alloc::alloc::Layout;
use core::ptr;
use super::Locked;
use alloc::alloc::GlobalAlloc;
use core::{mem, ptr::NonNull};

/// Node for a linked list of free blocks
/// 
/// Each node represents a free block and points to the next free block. Size of the block
/// is determined by the block size associated with the list head.
struct ListNode {
    next: Option<&'static mut ListNode>,
}

/// The block sizes to use.
///
/// The sizes must each be power of 2 bc they are alays used for alignment.
const BLOCK_SIZES: &[usize] = &[8, 16, 32, 64, 128, 256, 512, 1024, 2048];

/// A fixed size block allocator with a fallback linked list allocator.
/// 
/// Stores a linked list of free blocks for each block size in BLOCK_SIZES
/// and uses a linked list allocator as a fallback for larger allocations.
pub struct FixedSizeBlockAllocator {
    list_heads: [Option<&'static mut ListNode>; BLOCK_SIZES.len()],
    fallback_allocator: linked_list_allocator::Heap,
}

impl FixedSizeBlockAllocator {
    /// Creates an empty fixed size block allocator.
    pub const fn new() -> Self {
        const EMPTY: Option<&'static mut ListNode> = None;
        FixedSizeBlockAllocator {
            list_heads: [EMPTY; BLOCK_SIZES.len()],
            fallback_allocator: linked_list_allocator::Heap::empty(),
        }
    }

    /// Initilize the allocator with the given heap bounds.
    /// 
    /// # Safety
    /// This function is unsafe because the caller must guarantee that the given
    /// heap bounds are valid and that the memory is not used for anything else.
    /// This function must only be called once.
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        unsafe {
            self.fallback_allocator.init(heap_start, heap_size);
        }
    }

    /// Allocates using the fallback allocator.
    /// 
    /// Return a pointer to the allocated memory or null if allocation failed.
    fn fallback_alloc(&mut self, layout: Layout) -> *mut u8 {
        match self.fallback_allocator.allocate_first_fit(layout) {
            Ok(ptr) => ptr.as_ptr(),
            Err(_) => ptr::null_mut(),
        }
    }
}

/// Choose an appropriate block size for the given layout.
/// 
/// Return an index into the BLOCK_SIZES array.
fn list_index(layout: &Layout) -> Option<usize> {
    let required_size = layout.size().max(layout.align());
    BLOCK_SIZES.iter().position(|&s| s >= required_size)
}

/// Implement GlobalAlloc for FixedSizeBlockAllocator
/// 
/// FixedSizeBlockAllocator is wrapped in a Locked<T> to allow mutable access
/// in a thread-safe manner.
unsafe impl GlobalAlloc for Locked<FixedSizeBlockAllocator> {
    /// Allocates a block of memory with the given layout.
    /// 
    /// Returns a pointer to the allocated memory or null if allocation failed.
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut allocator = self.lock(); // get a mutable reference
        match list_index(&layout) { // find suitable block size
            Some(index) => { // suitable block size found
                match allocator.list_heads[index].take() { // take the head of the list
                    Some(node) => { // block exists in list => use it
                        allocator.list_heads[index] = node.next.take(); // update head to next node
                        node as *mut ListNode as *mut u8 // return pointer to block
                    }
                    None => {
                        // no block exists in list => allocate a new block
                        let block_size = BLOCK_SIZES[index];
                        // only works if all block sizes are power of 2
                        let block_align = block_size;
                        let layout = Layout::from_size_align(block_size, block_align).unwrap();
                        allocator.fallback_alloc(layout)
                    }
                }
            }
            None => {
                // not suitable block size found => use fallback allocator
                allocator.fallback_alloc(layout)
            }
        }
    }

    /// Deallocates the memory at the given pointer with the given layout.
    /// 
    ///
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        
        let mut allocator = self.lock(); // get a mutable reference
        match list_index(&layout) { // find suitable block size
            Some(index) => { // suitable block size found
                // create a new node for the freed block
                let new_node = ListNode {
                    next: allocator.list_heads[index].take(),
                };
                // verify that the block has size and alignment required for storing a Node
                assert!(mem::size_of::<ListNode>() <= BLOCK_SIZES[index]);
                assert!(mem::align_of::<ListNode>() <= BLOCK_SIZES[index]);
                // write the new node into the freed block
                let new_node_ptr = ptr as *mut ListNode;
                unsafe {
                    new_node_ptr.write(new_node);
                    allocator.list_heads[index] = Some(&mut *new_node_ptr);
                }
            }
            None => {
                // not suitable block size found => use fallback allocator
                let ptr = NonNull::new(ptr).unwrap();
                unsafe {
                    allocator.fallback_allocator.deallocate(ptr, layout);
                }
            }        
        }
    }
}