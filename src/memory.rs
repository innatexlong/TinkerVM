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

    pub fn extend(&mut self, len: usize) -> Result<(), crate::exceptions::Error> {
        self.resize(self.mem_blocks.len() + len)?;
        Ok(())
    }
    pub fn resize(&mut self, new_len: usize) -> Result<(), crate::exceptions::Error> {
        let old_len = self.mem_blocks.len();

        if new_len < old_len {
            return Err(crate::exceptions::Error::InvalidSize(
                "Shrinking is not allowed".to_string()
            ));
        } else if new_len == old_len {
            return Ok(());
        } else if new_len > self.max_capacity {
            return Err(crate::exceptions::Error::OutOfMemory(
                format!("Exceeds max_capacity {}", self.max_capacity)
            ));
        }

        // 1. 扩展 Vec。如果 capacity 不足，会触发堆重分配。
        //    但因为存的是指针（8字节），拷贝速度极快。
        self.mem_blocks.resize_with(new_len, || None);

        // 2. 🔥 核心修正：只追加新索引，绝不修改 free_list 原有内容。
        for idx in old_len..new_len {
            self.free_list.push(crate::env::Pos(idx));
        }

        Ok(())
    }

    /// 重置到指定容量（谨慎使用）
    /// 会丢弃所有已有数据，仅用于初始化或彻底重置
    pub fn reset(&mut self, capacity: usize) {
        self.mem_blocks.clear();
        self.mem_blocks.resize_with(capacity, || None);
        self.free_list.clear();
        self.free_list.extend((0..capacity).map(crate::env::Pos));
        // 注意：min_capacity / max_capacity 也需要同步更新，否则后续 alloc 检查会出错
    }

    pub fn alloc(&mut self) -> Result<crate::env::Pos, crate::exceptions::Error> {
        if let Some(pos) = self.free_list.pop() {
            self.mem_blocks[pos.0] = Some(Box::new(MemBlock::Null));
            return Ok(pos);
        }

        // 需要扩容
        let current_len = self.mem_blocks.len();
        let remaining = self.max_capacity - current_len;
        if remaining == 0 {
            return Err(crate::exceptions::Error::OutOfMemory(
                "free_list has been exhausted before allocating".to_string(),
            ));
        }

        // 每次扩容 min(64, remaining) 个块
        let grow_by = std::cmp::min(64, remaining);
        let new_len = current_len + grow_by;

        // 扩展 mem_blocks，新增块为 None
        self.mem_blocks.resize(new_len, None);

        // 将新位置加入 free_list（注意：新索引从 current_len 开始）
        for idx in current_len..new_len {
            self.free_list.push(crate::env::Pos(idx));
        }

        // 现在 free_list 一定非空，递归调用或直接弹出
        let pos = self.free_list.pop().unwrap();
        self.mem_blocks[pos.0] = Some(Box::new(MemBlock::Null));
        Ok(pos)
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
