#![allow(unused)]

use std::ptr;

const MIN_ARENA_SIZE: usize = 64;

pub struct MvmAllocator {
    start: *mut u8,
    arena: *mut AllocatorArena
}

struct AllocatorArena {
    pub base: *mut AllocatorBlock,
    pub size: usize,

    pub next: *mut AllocatorArena,
}

struct AllocatorBlock {
    pub size: usize,
    pub free: bool,

    pub next: *mut AllocatorBlock
}

impl MvmAllocator {
    pub fn new(mem_ptr: *mut u8) -> Self {
        Self {
            start: mem_ptr,
            arena: AllocatorArena::new(mem_ptr)
        }
    }

    pub fn allocate(&mut self, size: usize) -> *mut u8 {
        let block = unsafe { (*self.arena).allocate_block(size) };

        AllocatorBlock::get_data_ptr(block)
    }
}

impl AllocatorArena {
    pub fn new(ptr: *mut u8) -> *mut Self {
        let arena = ptr as *mut AllocatorArena;
        let base = unsafe { arena.add(1) } as *mut u8;

        unsafe {
            (*arena).size = MIN_ARENA_SIZE;
            (*arena).base = AllocatorBlock::new(base, 8, true);
            (*arena).next = ptr::null_mut();
        }
        
        arena
    }

    pub fn allocate_block(&mut self, size: usize) -> *mut AllocatorBlock {
        unsafe {
            if (*self.base).free && (*self.base).size <= size {
                return self.base
            }

            let arena_size = self.size;
            let arena_base = self.base;

            let mut ptr = self.base;

            while (ptr.sub(arena_base as usize) as usize) < arena_size {
                if (*ptr).free && (*ptr).size <= size {
                    return ptr
                }

                if !(*ptr).next.is_null() {
                    ptr = (*ptr).next;
                    continue;
                }

                let mut block_ptr = (ptr.add(1)) as *mut u8;
                block_ptr = block_ptr.add((*ptr).size);

                if (block_ptr.sub(arena_base as usize) as usize) >= arena_size {
                    return ptr::null_mut();
                }

                let block = AllocatorBlock::new(block_ptr, size, false);

                (*ptr).next = block;

                return block;
            }
        }

        return ptr::null_mut();
    }

    pub fn deallocate_block(&mut self, ptr: *mut AllocatorBlock) {
        unsafe {
            (*ptr).free = true;
        }
    }
}

impl AllocatorBlock {
    pub fn new(ptr: *mut u8, size: usize, free: bool) -> *mut Self {
        let block = ptr as *mut AllocatorBlock;

        unsafe {
            (*block).size = size;
            (*block).free = free;
            (*block).next = ptr::null_mut();
        }

        block
    }

    pub fn get_data_ptr(ptr: *mut AllocatorBlock) -> *mut u8 {
        unsafe { ptr.add(1) as *mut u8 }
    }

    pub fn from_data_ptr(ptr: *mut u8) -> *mut AllocatorBlock {
        unsafe { (ptr as *mut AllocatorBlock).sub(1) }
    }
}
