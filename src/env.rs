#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Pos(pub usize);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FuncPtr(pub u32);

impl PartialEq<usize> for Pos {
    fn eq(&self, other: &usize) -> bool {
        other == &self.0
    }
}
impl PartialOrd<usize> for Pos {
    fn partial_cmp(&self, other: &usize) -> Option<std::cmp::Ordering> {
        Some(other.cmp(&self.0))
    }
}
impl std::ops::AddAssign<usize> for Pos {
    fn add_assign(&mut self, rhs: usize) {
        self.0 += rhs
    }
}

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

pub struct Env {
    pub memory_pool: std::sync::Arc<std::sync::RwLock<crate::memory::MemoryPool>>,
    pub vars: dashmap::DashMap<crate::value::VarId, crate::value::TypedPtr>,
    pub parent: Option<std::sync::Arc<std::sync::RwLock<Env>>>,
}

impl Env {
    pub fn new(
        memory_pool: std::sync::Arc<std::sync::RwLock<crate::memory::MemoryPool>>, parent: Option<std::sync::Arc<std::sync::RwLock<Env>>>
    ) -> Self {
        Self {
            memory_pool,
            vars: Default::default(),
            parent: match parent {
                Some( parent) => Some(parent),
                None => None
            }
        }
    }
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

    pub fn set_var_current(
        &self,
        id: crate::value::VarId,
        value: crate::value::Var,
    ) -> Result<(), crate::exceptions::Error> {
        // 只在当前环境 vars 中查找
        let ptr = self.vars.get(&id)
            .map(|r| r.value().clone())
            .ok_or_else(|| crate::exceptions::Error::VarNotFound(format!("Failed to set var {id} since the current env doesn't define {id}")))?;
        self.with_var_mut(ptr, |var| *var = value)
    }

    /// 通过 TypedPtr 获取值的可变引用
    pub fn with_var_mut<F, R>(&self, ptr: crate::value::TypedPtr, f: F) -> Result<R, crate::exceptions::Error>
    where
        F: FnOnce(&mut crate::value::Var) -> R,
    {
        let mut pool_guard = self.memory_pool.write()
            .map_err(|e| crate::exceptions::Error::InvalidOperation(format!("Lock poisoned: {}", e)))?;
        let var = pool_guard.get_mut(ptr.pos)
            .map_err(|_| crate::exceptions::Error::VarNotFound("position invalid".into()))?;
        Ok(f(var))
    }
    pub fn insert_var(&mut self, id: crate::value::VarId, ptr: crate::value::TypedPtr) -> Option<crate::value::TypedPtr> {
        self.vars.insert(id, ptr)
    }
}
