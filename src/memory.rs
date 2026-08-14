pub type MemBlock = crate::value::Var;

pub struct MemoryPool {
    mem_blocks: Vec<Option<MemBlock>>,
    free_list: Vec<crate::env::Pos>,
}

impl MemoryPool {
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        Self {
            mem_blocks: vec![None; capacity],
            free_list: {
                let mut vec: Vec<crate::env::Pos> = Vec::with_capacity(capacity);
                let mut i = crate::env::Pos(0usize);
                while i < capacity {
                    vec.push(i);
                    i += 1usize;
                }
                vec
            }
        }
    }
    pub fn get(&self, pos: crate::env::Pos) -> Result<&MemBlock, crate::exceptions::Error>{
        if pos.0 < self.mem_blocks.len() {
            return match self.mem_blocks[pos.0] {
                Some(ref block) => Ok(block),
                None => Err(crate::exceptions::Error::SegFault(format!("Wild pointer {}", pos.0))),
            }
        }
        Err(crate::exceptions::Error::InvalidPointer(format!("{}", pos.0)))
    }
    pub fn get_mut(&mut self, pos: crate::env::Pos) -> Result<&mut MemBlock, crate::exceptions::Error>{
        if pos.0 < self.mem_blocks.len() {
            return match self.mem_blocks[pos.0] {
                Some(ref mut block) => Ok(block),
                None => Err(crate::exceptions::Error::SegFault(format!("Wild pointer {}", pos.0))),
            }
        }
        Err(crate::exceptions::Error::InvalidPointer(format!("{}", pos.0)))
    }
    pub fn set(&mut self, pos: crate::env::Pos, value: MemBlock) -> Result<(), crate::exceptions::Error> {
        match self.mem_blocks.get_mut(pos.0) {
            Some(None) => Err(crate::exceptions::Error::SegFault("write to unallocated memory".to_string())),
            Some(block @ Some(_)) => { *block = Some(value); Ok(()) }
            None => Err(crate::exceptions::Error::InvalidPointer(format!("{}", pos.0)))
        }
    }
    pub fn alloc(&mut self) -> Result<crate::env::Pos, crate::exceptions::Error> {
        match self.free_list.pop() {
            Some(pos) => {
                self.mem_blocks[pos.0] = Some(
                    MemBlock::Null
                );
                Ok(pos)
            },
            None => Err(crate::exceptions::Error::OutOfMemory("free_list has been exhausted before allocating".to_string()))  // TODO: Dynamic length of MemoryPool
        }
    }
    pub fn dealloc(&mut self, pos: crate::env::Pos) -> Result<(), crate::exceptions::Error> {
        match self.mem_blocks.get_mut(pos.0) {
            Some(block_opt) => {
                match block_opt {
                    Some(_) => {
                        *block_opt = None;
                        self.free_list.push(pos);
                        Ok(())
                    }
                    None => Err(crate::exceptions::Error::OutOfMemory("can't deallocate memory".to_string()))
                }
            },
            None => Err(crate::exceptions::Error::SegFault(format!("Dealloc wild pointer {}", pos.0)))
        }
    }
}
