#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Pos(pub usize);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FuncPtr(pub u32);

impl std::fmt::Display for Pos {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
// impl std::fmt::Display for Size {
//     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//         write!(f, "{}", self.0)
//     }
// }
impl std::fmt::Display for FuncPtr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub struct SymbolTable {
    string_to_id: dashmap::DashMap<String, crate::value::VarId>,
    id_to_string: dashmap::DashMap<crate::value::VarId, String>,
}

impl SymbolTable {
    pub fn new() -> SymbolTable {
        SymbolTable{ string_to_id: dashmap::DashMap::new(), id_to_string: dashmap::DashMap::new() }
    }
    // TODO
    // fn get_from_string(&mut self, string: &str) -> dashmap::mapref::one::Ref<String, VarId>
    // {
    //     if self.string_to_id.contains_key(string) {
    //         let id = self.string_to_id.get(string)?;
    //         self.id_to_string[id.value()] = string.to_string();
    //         id
    //     } else {
    //         VarId(self.string_to_id.len() as u32)
    //     }
    // }
}

pub struct FuncInfo {
    pub code: std::sync::Arc<Vec<u8>>,
    pub labels: Vec<u64>,
    // pub ret_type: crate::value::ValueType
    // pub arity: usize,
    // pub args: std::sync::Arc<crate::value::ValueType>,
    pub constants: Vec<crate::value::Var>
}
impl std::fmt::Debug for FuncInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FuncInfo")
            .field("code", &format_args!("{:02X?}", self.code))
            .field("constants", &self.constants)
            .finish()
    }
}

pub struct Env {
    pub memory_pool: std::sync::Arc<std::sync::RwLock<crate::memory::MemoryPool>>,
    /// 局部变量表：模拟 JVM 栈帧，索引就是 VarId.0
    pub vars: Vec<Option<crate::value::ValueSlot>>,
    pub parent: Option<std::sync::Arc<std::sync::RwLock<Env>>>,
    pub funcs: dashmap::DashMap<FuncPtr, std::sync::Arc<FuncInfo>>,
}
impl Env {
    pub fn new(
        memory_pool: std::sync::Arc<std::sync::RwLock<crate::memory::MemoryPool>>,
        parent: Option<std::sync::Arc<std::sync::RwLock<Env>>>,
    ) -> Self {
        Self {
            memory_pool,
            vars: Vec::new(),
            parent,
            funcs: Default::default(),
        }
    }

    /// 确保局部变量表长度足够
    fn ensure_capacity(&mut self, index: usize) {
        if index >= self.vars.len() {
            self.vars.resize_with(index + 1, || None);
        }
    }

    /// 查找变量槽（沿作用域链向上）
    pub fn lookup_slot(&self, id: &crate::value::VarId) -> Option<crate::value::ValueSlot> {
        if let Some(slot) = self.vars.get(id.0 as usize).and_then(|s| s.clone()) {
            return Some(slot);
        }
        if let Some(parent) = &self.parent {
            return parent.read().unwrap().lookup_slot(id);
        }
        None
    }

    /// 获取变量值：基本类型直接返回，引用类型通过 TypedPtr 从堆中取
    pub fn get_var(&self, id: &crate::value::VarId) -> Result<crate::value::Var, crate::exceptions::Error> {
        match self.lookup_slot(id).ok_or_else(|| crate::exceptions::Error::NotFound(format!("Var {id}")))? {
            crate::value::ValueSlot::Primitive(v) => Ok(v),
            crate::value::ValueSlot::Reference(ptr) => self.with_var(ptr, |v| v.clone()),
        }
    }

    /// 设置变量值：基本类型直接写入槽，引用类型通过 TypedPtr 写入堆
    pub fn set_var_value(
        &mut self,
        id: crate::value::VarId,
        value: crate::value::Var,
    ) -> Result<(), crate::exceptions::Error> {
        let index = id.0 as usize;
        // 确保索引在范围内
        if index >= self.vars.len() {
            return Err(crate::exceptions::Error::NotFound(format!("Failed to set var {id}")));
        }

        // 取出当前槽位（不释放，后面可能还要用）
        let slot = self.vars[index].as_ref().cloned()
            .ok_or_else(|| crate::exceptions::Error::NotFound(format!("Failed to set var {id}")))?;

        match slot {
            crate::value::ValueSlot::Primitive(_) => {
                // 基本类型：直接替换槽位中的值
                self.vars[index] = Some(crate::value::ValueSlot::Primitive(value));
                Ok(())
            }
            crate::value::ValueSlot::Reference(ptr) => {
                // 引用类型：通过 TypedPtr 修改堆内存中的值
                self.with_var_mut(ptr, |var| *var = value)
            }
        }
    }

    /// 设置引用类型的变量槽（仅用于 String、Ptr 等引用类型）
    /// TODO: 支持拷贝类型
    pub fn set_var_ref(
        &mut self,
        id: crate::value::VarId,
        ptr: crate::value::TypedPtr,
    ) -> Result<(), crate::exceptions::Error> {
        if !ptr.ty.is_reference_type() {
            return Err(crate::exceptions::Error::InvalidOperation(
                "set_var_ref can only be used with reference types".into(),
            ));
        }

        let index = id.0 as usize;
        if index >= self.vars.len() {
            return Err(crate::exceptions::Error::NotFound(format!(
                "Failed to set var {id}: index out of bounds"
            )));
        }

        // 直接写入引用类型槽位，覆盖旧值（若旧值也是引用，不会自动释放堆内存，需调用者负责）
        self.vars[index] = Some(crate::value::ValueSlot::Reference(ptr));
        Ok(())
    }

    /// 通过 TypedPtr 获取值的只读引用
    pub fn with_var<F, R>(&self, ptr: crate::value::TypedPtr, f: F) -> Result<R, crate::exceptions::Error>
    where
        F: FnOnce(&crate::value::Var) -> R,
    {
        let pool_guard = self.memory_pool.read()
            .map_err(|e| crate::exceptions::Error::InvalidOperation(format!("Lock poisoned: {}", e)))?;
        let var = pool_guard.get(ptr.pos)?;
        Ok(f(var))
    }

    /// 通过 TypedPtr 获取值的可变引用
    pub fn with_var_mut<F, R>(&self, ptr: crate::value::TypedPtr, f: F) -> Result<R, crate::exceptions::Error>
    where
        F: FnOnce(&mut crate::value::Var) -> R,
    {
        let mut pool_guard = self.memory_pool.write()
            .map_err(|e| crate::exceptions::Error::InvalidOperation(format!("Lock poisoned: {}", e)))?;
        let var = pool_guard.get_mut(ptr.pos)
            .map_err(|_| crate::exceptions::Error::NotFound("position invalid".into()))?;
        Ok(f(var))
    }

    pub fn remove_var(&mut self, id: crate::value::VarId) -> Result<(), crate::exceptions::Error> {
        let index = id.0 as usize;
        if index >= self.vars.len() {
            return Err(crate::exceptions::Error::NotFound(format!(
                "Delete undefined var {id}"
            )));
        }

        match self.vars[index].take() {
            Some(_) => Ok(()),
            None => Err(crate::exceptions::Error::NotFound(format!(
                "Delete undefined var {id}"
            ))),
        }
    }

    /// 删除变量槽并释放引用类型指向的堆内存
    pub fn drop_var(&mut self, id: crate::value::VarId) -> Result<(), crate::exceptions::Error> {
        let index = id.0 as usize;
        if index >= self.vars.len() {
            return Err(crate::exceptions::Error::NotFound(format!(
                "Delete undefined var and its memory {id}"
            )));
        }

        let slot = self.vars[index].take().ok_or_else(|| {
            crate::exceptions::Error::NotFound(format!(
                "Delete undefined var and its memory {id}"
            ))
        })?;

        // 只有引用类型才需要释放堆内存
        if let crate::value::ValueSlot::Reference(ptr) = slot {
            let mut pool = self.memory_pool.write().map_err(|e| {
                crate::exceptions::Error::InvalidOperation(format!("Lock poisoned: {}", e))
            })?;
            pool.dealloc(ptr.pos)?; // 假设 dealloc 返回 Result<(), Error>
        }

        Ok(())
    }

    /// 插入一个变量槽（基本类型或引用类型），返回被替换的旧槽（如果有）
    pub fn insert_slot(
        &mut self,
        id: crate::value::VarId,
        slot: crate::value::ValueSlot,
    ) -> Option<crate::value::ValueSlot> {
        let index = id.0 as usize;
        self.ensure_capacity(index);
        let old = self.vars[index].take();
        self.vars[index] = Some(slot);
        old
    }

    /// 插入一个引用类型变量（如 String、Ptr），返回被替换的旧槽
    pub fn insert_var(
        &mut self,
        id: crate::value::VarId,
        ptr: crate::value::TypedPtr,
    ) -> Option<crate::value::ValueSlot> {
        self.insert_slot(id, crate::value::ValueSlot::Reference(ptr))
    }

    /// 插入一个基本类型变量，返回被替换的旧槽
    pub fn insert_primitive(
        &mut self,
        id: crate::value::VarId,
        value: crate::value::Var,
    ) -> Option<crate::value::ValueSlot> {
        self.insert_slot(id, crate::value::ValueSlot::Primitive(value))
    }

    pub fn get_func(
        &self,
        id: &FuncPtr,
    ) -> Result<std::sync::Arc<FuncInfo>, crate::exceptions::Error> {
        if let Some(func) = self.funcs.get(id) {
            return Ok(func.clone());
        }
        if let Some(parent) = &self.parent {
            return parent.read().unwrap().get_func(id);
        }

        // 没找到
        Err(crate::exceptions::Error::NotFound(format!(
            "Function {id} not found"
        )))
    }

    #[inline]
    /// 注册函数
    pub fn register_func(
        &self,
        id: FuncPtr,
        input: Vec<u8>,
        labels: Vec<u64>,
        constants: Vec<crate::value::Var>
    ) -> Result<(), crate::exceptions::Error> {  // 或者自定义错误类型
        match self.funcs.entry(id) {
            // 键已存在 -> 直接报错，不读取 input（保持零副作用）
            dashmap::Entry::Occupied(_) => Err(crate::exceptions::Error::Duplicated(format!("func {id} already registered"))),

            // 键不存在 -> 单次查找定位后，读取并插入
            dashmap::Entry::Vacant(entry) => {
                // let mut bytes = Vec::new();
                // match input.read_to_end(&mut bytes) {
                //     Ok(_) => {},
                //     Err(e) => {
                //         return Err(crate::exceptions::Error::from(e))
                //     }
                // }

                // entry.insert(std::io::BufReader::new(std::io::Cursor::new(bytes)));
                let func_info = FuncInfo {
                    code: std::sync::Arc::new(input),
                    labels,
                    constants
                };
                entry.insert(std::sync::Arc::new(func_info));
                Ok(())
            }
        }
    }
}
