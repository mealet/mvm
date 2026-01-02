#![allow(unused)]

use super::error::MvmError;
use thiserror::Error;

#[derive(Debug)]
pub struct MvmAllocator {
    mem_start: usize,
    mem_end: usize,

    allocated: Vec<AllocatorUnit>,
}

#[derive(Debug, Error)]
pub enum AllocatorError {
    #[error("allocator is out of memory units")]
    OutOfMemory,

    #[error("invalid pointer is being freed")]
    InvalidFreePointer,

    #[error("restricted unit access is not allowed")]
    RestrictedUnitAccess,
}

#[derive(Debug)]
struct AllocatorUnit {
    pub address: usize,
    pub size: usize,
    pub free: bool,
    pub restricted: bool,
}

impl MvmAllocator {
    pub fn new(mem_start: usize, mem_end: usize) -> Self {
        let global_unit = AllocatorUnit {
            address: mem_start,
            size: mem_end - mem_start,
            free: true,
            restricted: true,
        };

        let allocated = vec![global_unit];

        Self {
            mem_start,
            mem_end,
            allocated,
        }
    }

    pub fn allocate(&mut self, size: usize) -> Result<usize, AllocatorError> {
        const ALLOCATOR_ALIGN: usize = 4;

        let aligned_size = (size + (ALLOCATOR_ALIGN - 1)) & !(ALLOCATOR_ALIGN - 1);
        let mut idx = 0;

        while let Some(unit) = self.allocated.get_mut(idx) {
            // exact length unit

            if unit.size == aligned_size && unit.free {
                unit.free = false;
                return Ok(unit.address);
            }

            // bigger unit (splitting)

            if unit.size > aligned_size && unit.free {
                let left_size = aligned_size;
                let right_size = unit.size - left_size;

                let left_addr = unit.address;
                let right_addr = left_addr + left_size;

                let right_restricted = unit.restricted;

                self.allocated.splice(
                    idx..(idx + 1),
                    [
                        AllocatorUnit {
                            address: left_addr,
                            size: left_size,
                            free: false,
                            restricted: false,
                        },
                        AllocatorUnit {
                            address: right_addr,
                            size: right_size,
                            free: true,
                            restricted: right_restricted
                        },
                    ],
                );

                return Ok(left_addr);
            }

            // otherwise trying to merge units

            if unit.free {
                let current_addr = unit.address;
                let current_size = unit.size;

                let next = self.allocated.get_mut(idx + 1);

                if let Some(next) = next
                    && next.free
                {
                    let merged_size = current_size + next.size;
                    let merged_unit = AllocatorUnit {
                        address: current_addr,
                        size: merged_size,
                        free: true,
                        restricted: next.restricted
                    };

                    self.allocated.splice(idx..(idx + 2), [merged_unit]);
                    continue;
                }
            }

            idx += 1;
        }

        Err(AllocatorError::OutOfMemory)
    }

    pub fn deallocate(&mut self, ptr: usize) -> Result<(), AllocatorError> {
        if ptr < self.mem_start || ptr > self.mem_end {
            return Err(AllocatorError::InvalidFreePointer);
        }

        let mut iterator = self.allocated.iter_mut();

        while let Some(unit) = iterator.next() {
            if unit.address == ptr {
                if unit.restricted {
                    return Err(AllocatorError::RestrictedUnitAccess);
                }

                unit.free = true;
                return Ok(());
            }

            if unit.address > ptr {
                break;
            }
        }

        Err(AllocatorError::InvalidFreePointer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MEM_START: usize = 0;
    const MEM_END: usize = 32;
    const MEM_LEN: usize = MEM_END - MEM_START;
    const ALLOCA_LEN: usize = 4;

    #[test]
    fn allocator_base_test() {
        let mut allocator = MvmAllocator::new(MEM_START, MEM_END);
        let ptr = allocator.allocate(ALLOCA_LEN);
        
        assert_eq!(ptr.unwrap(), 0);
        assert_eq!(allocator.allocated[0].size, ALLOCA_LEN);
        assert_eq!(allocator.allocated[0].address, 0);
        assert_eq!(allocator.allocated[0].free, false);

        assert_eq!(allocator.allocated[1].size, MEM_LEN - ALLOCA_LEN);
        assert_eq!(allocator.allocated[1].address, ALLOCA_LEN);
        assert_eq!(allocator.allocated[1].free, true);
    }

    #[test]
    fn allocator_double_test() {
        let mut allocator = MvmAllocator::new(MEM_START, MEM_END);

        let ptr1 = allocator.allocate(ALLOCA_LEN);
        let ptr2 = allocator.allocate(ALLOCA_LEN);

        assert_eq!(ptr1.unwrap(), 0);
        assert_eq!(allocator.allocated[0].size, ALLOCA_LEN);
        assert_eq!(allocator.allocated[0].address, 0);
        assert_eq!(allocator.allocated[0].free, false);

        assert_eq!(ptr2.unwrap(), ALLOCA_LEN);
        assert_eq!(allocator.allocated[1].size, ALLOCA_LEN);
        assert_eq!(allocator.allocated[1].address, ALLOCA_LEN);
        assert_eq!(allocator.allocated[1].free, false);

        assert_eq!(allocator.allocated[2].size, MEM_LEN - (ALLOCA_LEN * 2));
        assert_eq!(allocator.allocated[2].address, ALLOCA_LEN * 2);
        assert_eq!(allocator.allocated[2].free, true);
    }

    #[test]
    fn allocator_merge_test() {
        let mut allocator = MvmAllocator::new(MEM_START, MEM_END);

        let ptr = allocator.allocate(ALLOCA_LEN);
        let _ = allocator.allocate(ALLOCA_LEN);

        // At this point we have 2 allocated units with the same size.
        // Now we're going to manually free them and try to allocate new one with double size.
        // It should merge previous units and return their ptr.

        allocator.allocated[0].free = true;
        allocator.allocated[1].free = true;

        let new_ptr = allocator.allocate(ALLOCA_LEN * 2);

        assert_eq!(new_ptr.unwrap(), ptr.unwrap());
        assert_eq!(allocator.allocated[0].address, 0);
        assert_eq!(allocator.allocated[0].size, ALLOCA_LEN * 2);
        assert_eq!(allocator.allocated[0].free, false);
    }


    #[test]
    fn allocator_dealloc_test() {
        let mut allocator = MvmAllocator::new(MEM_START, MEM_END);

        let ptr = allocator.allocate(ALLOCA_LEN).unwrap();

        assert_eq!(ptr, 0);
        assert_eq!(allocator.allocated[0].address, 0);
        assert_eq!(allocator.allocated[0].size, ALLOCA_LEN);
        assert_eq!(allocator.allocated[0].free, false);

        let result = allocator.deallocate(ptr);

        assert!(result.is_ok());
        assert_eq!(allocator.allocated[0].address, 0);
        assert_eq!(allocator.allocated[0].size, ALLOCA_LEN);
        assert_eq!(allocator.allocated[0].free, true);
    }

    #[test]
    fn allocator_realloc_test() {
        let mut allocator = MvmAllocator::new(MEM_START, MEM_END);

        let ptr = allocator.allocate(ALLOCA_LEN).unwrap();

        assert_eq!(ptr, 0);
        assert_eq!(allocator.allocated[0].address, 0);
        assert_eq!(allocator.allocated[0].size, ALLOCA_LEN);
        assert_eq!(allocator.allocated[0].free, false);

        let result = allocator.deallocate(ptr);

        assert!(result.is_ok());
        assert_eq!(allocator.allocated[0].address, 0);
        assert_eq!(allocator.allocated[0].size, ALLOCA_LEN);
        assert_eq!(allocator.allocated[0].free, true);

        let new_ptr = allocator.allocate(ALLOCA_LEN).unwrap();

        assert_eq!(new_ptr, 0);
        assert_eq!(allocator.allocated[0].address, 0);
        assert_eq!(allocator.allocated[0].size, ALLOCA_LEN);
        assert_eq!(allocator.allocated[0].free, false);
    }

    #[test]
    fn allocator_out_of_memory_test() {
        let mut allocator = MvmAllocator::new(MEM_START, MEM_END);

        let ptr = allocator.allocate(MEM_LEN + 1);

        assert!(
            matches!(
                ptr,
                Err(AllocatorError::OutOfMemory)
            )
        );
    }

    #[test]
    fn allocator_invalid_free_ptr_test() {
        let mut allocator = MvmAllocator::new(MEM_START, MEM_END);

        assert!(
            matches!(
                allocator.deallocate(1),
                Err(AllocatorError::InvalidFreePointer)
            )
        );
    }

    #[test]
    fn allocator_restricted_access_test() {
        let mut allocator = MvmAllocator::new(MEM_START, MEM_END);

        assert!(
            matches!(
                allocator.deallocate(0),
                Err(AllocatorError::RestrictedUnitAccess)
            )
        );
    }
}
