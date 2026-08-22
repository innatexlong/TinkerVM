pub type MemBlock = crate::value::Var;

pub struct MemoryPool {
    mem_blocks: Vec<Option<Box<MemBlock>>>,
    free_list: Vec<crate::env::Pos>,
    min_capacity: usize,
    max_capacity: usize,
}

impl MemoryPool {
    #[must_use]
    pub fn new(min_capacity: usize, max_capacity: usize) -> Self {
        assert!(min_capacity <= max_capacity, "invalid capacity range [{min_capacity}..{max_capacity}]");
        let mid_capacity = (min_capacity + max_capacity) / 2;
        Self {
            min_capacity,
            max_capacity,
            mem_blocks: vec![None; mid_capacity],
            free_list: {
                let mut vec: Vec<crate::env::Pos> = Vec::with_capacity(mid_capacity);
                let mut i = crate::env::Pos(0usize);
                while i.0 < mid_capacity {
                    vec.push(i);
                    i.0 += 1usize;
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
        Err(crate::exceptions::Error::WildPointer(format!("{}", pos.0)))
    }
    pub fn get_mut(&mut self, pos: crate::env::Pos) -> Result<&mut MemBlock, crate::exceptions::Error>{
        if pos.0 < self.mem_blocks.len() {
            return match self.mem_blocks[pos.0] {
                Some(ref mut block) => Ok(block),
                None => Err(crate::exceptions::Error::SegFault(format!("Wild pointer {}", pos.0))),
            }
        }
        Err(crate::exceptions::Error::WildPointer(format!("{}", pos.0)))
    }
    pub fn set(&mut self, pos: crate::env::Pos, value: MemBlock) -> Result<(), crate::exceptions::Error> {
        match self.mem_blocks.get_mut(pos.0) {
            Some(None) => Err(crate::exceptions::Error::SegFault("write to unallocated memory".to_string())),
            Some(block @ Some(_)) => { *block = Some(Box::from(value)); Ok(()) }
            None => Err(crate::exceptions::Error::WildPointer(format!("{}", pos.0)))
        }
    }
    pub fn alloc(&mut self) -> Result<crate::env::Pos, crate::exceptions::Error> {
        match self.free_list.pop() {
            Some(pos) => {
                self.mem_blocks[pos.0] = Some(
                    Box::new(MemBlock::Null)
                );
                Ok(pos)
            },
            None => {
                let remaining = self.max_capacity - self.mem_blocks.len();
                if remaining == 0 { Err(crate::exceptions::Error::OutOfMemory("free_list has been exhausted before allocating".to_string())) }
                else { Ok(crate::env::Pos(std::cmp::min(64, remaining))) }
            }  // TODO: Dynamic length of MemoryPool
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
