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
    // pub ret_type: crate::value::ValueType
    // pub arity: usize,
    // pub args: std::sync::Arc<crate::value::ValueType>,
    pub constants: std::sync::Arc<[crate::value::Var]>
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
    pub vars: dashmap::DashMap<crate::value::VarId, crate::value::TypedPtr>,
    pub parent: Option<std::sync::Arc<std::sync::RwLock<Env>>>,
    pub funcs: dashmap::DashMap<FuncPtr, std::sync::Arc<FuncInfo>>,
}
impl Env {
    #[inline]
    pub fn new(
        memory_pool: std::sync::Arc<std::sync::RwLock<crate::memory::MemoryPool>>, parent: Option<std::sync::Arc<std::sync::RwLock<Env>>>
    ) -> Self {
        Self {
            memory_pool,
            vars: Default::default(),
            parent,
            funcs: Default::default(),
        }
    }

    #[inline]
    pub fn as_mut_ref(&mut self) -> &mut Env {
        self
    }

    #[inline]
    /// 查找变量（沿作用域链向上）
    pub fn lookup_var(&self, id: &crate::value::VarId) -> Option<crate::value::TypedPtr> {
        if let Some(ptr) = self.vars.get(id) {
            return Some(ptr.clone());
        }
        if let Some(parent) = &self.parent {
            return parent.read().unwrap().lookup_var(id);
        }
        None
    }

    #[inline]
    /// 通过 TypedPtr 获取值的引用（只读）
    pub fn with_var<F, R>(&self, ptr: crate::value::TypedPtr, f: F) -> Result<R, crate::exceptions::Error>
    where
        F: FnOnce(&crate::value::Var) -> R,
    {
        let pool_guard = self.memory_pool.read()
            .map_err(|e| crate::exceptions::Error::InvalidOperation(format!("Lock poisoned: {}", e)))?;
        let var = pool_guard.get(ptr.pos)?;
        Ok(f(var))
    }

    #[inline]
    pub fn set_var_value(
        &self,
        id: crate::value::VarId,
        value: crate::value::Var,
    ) -> Result<(), crate::exceptions::Error> {
        // 只在当前环境 vars 中查找
        let ptr = self.vars.get(&id)
            .map(|r| r.value().clone())
            .ok_or_else(|| crate::exceptions::Error::NotFound(format!("Failed to set var {id} since the current env doesn't define {id}")))?;
        self.with_var_mut(ptr, |var| *var = value)
    }

    #[inline]
    pub fn set_var_pos(
        &self,
        id: crate::value::VarId,
        ptr: crate::value::TypedPtr,
    ) -> Result<(), crate::exceptions::Error> {
        self.vars.insert(id, ptr);
        Ok(())
    }

    #[inline]
    pub fn get_var(&self, id: &crate::value::VarId) -> Result<crate::value::Var, crate::exceptions::Error> {
        let ptr = self.lookup_var(id).ok_or_else(|| crate::exceptions::Error::NotFound(format!("Var {id}")))?;
        self.with_var(ptr, |var| var.clone())
    }

    /// 删除变量但不释放内存块
    #[inline]
    pub fn remove_var(&self, id: crate::value::VarId) -> Result<(), crate::exceptions::Error> {
        match self.vars.remove(&id) {
            Some(_) => Ok(()),
            None => Err(crate::exceptions::Error::NotFound(format!("Delete undefined var {id}"))),
        }
    }

    /// 删除变量且释放内存块
    #[inline]
    pub fn drop_var(&self, id: crate::value::VarId) -> Result<(), crate::exceptions::Error> {
        match self.vars.remove(&id) {
            Some(var) => {
                self.memory_pool.write().unwrap().dealloc(var.1.pos)
            },
            None => Err(crate::exceptions::Error::NotFound(format!("Delete undefined var and its memory {id}"))),
        }
    }

    #[inline]
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

    #[inline]
    pub fn insert_var(&mut self, id: crate::value::VarId, ptr: crate::value::TypedPtr) -> Option<crate::value::TypedPtr> {
        self.vars.insert(id, ptr)
    }

    pub fn get_func(
        &self,
        id: &FuncPtr,
    ) -> Result<
        std::sync::Arc<FuncInfo>,
        crate::exceptions::Error,
    > {
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
                    constants: std::sync::Arc::from(constants)
                };
                entry.insert(std::sync::Arc::new(func_info));
                Ok(())
            }
        }
    }
}
