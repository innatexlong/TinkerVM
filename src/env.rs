use std::collections::HashMap;

pub struct MemoryPool {
    memory: Vec<u8>
}
impl MemoryPool {
    pub fn new() -> MemoryPool {
        Self { memory: vec![] }
    }
    pub fn alloc(&mut self, pos: Pos, size: Size) -> () {}
    pub fn free(&mut self, pos: Pos, size: Size) {}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VarType {
    // TODO: Implement these types
    // Float,
    // Double,
    // U8, U16,
    U32(u32), //U64,
    U32Ptr(Pos),
    VarPtr(VarId)
}
impl std::fmt::Display for VarType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            VarType::U32(u) => write!(f, "U32(val={})", u),
            VarType::U32Ptr(p) => write!(f, "U32Ptr(id={})", p),
            VarType::VarPtr(p) => write!(f, "VarPtr(id={})", p),
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Var {
    pub var_type: VarType
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VarId(pub u32);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pos(pub u32);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Size(pub u32);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FuncPtr(pub u32);

impl std::fmt::Display for VarId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl std::fmt::Display for Pos {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl std::fmt::Display for Size {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl std::fmt::Display for FuncPtr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub struct SymbolTable {
    string_to_id: HashMap<String, VarId>,
    id_to_string: HashMap<VarId, String>,
}

impl SymbolTable {
    fn new() -> SymbolTable {
        SymbolTable{ string_to_id: HashMap::new(), id_to_string: HashMap::new() }
    }
    fn get_from_string(&mut self, string: &str) -> VarId {
        if self.string_to_id.contains_key(string) {
            self.string_to_id[string]
        } else {
            VarId(self.string_to_id.len() as u32)
        }
    }
}

pub struct Env {
    pub memory_pool: &'static MemoryPool,
    pub vars: HashMap<VarId, Var>,
    // funcs: HashMap<String, FuncPtr>, // TODO
    // parent: Env  // TODO
}

impl Env {
    pub fn new() -> Self {
        Self {
            memory_pool: &MemoryPool {},
            vars: Default::default(),
        }
    }
    #[inline]
    pub fn get_var(&self, key: VarId) -> Result<&Var, crate::exceptions::Error> {
        match self.vars.get(&key) {
            None => Err(crate::exceptions::Error::SegFault(format!("var {} not found", key))),
            Some(T) => Ok(T)
        }
    }
    #[inline]
    pub fn get_var_mut(&mut self, key: VarId) -> Result<&mut Var, crate::exceptions::Error> {
        match self.vars.get_mut(&key) {
            None => Err(crate::exceptions::Error::SegFault(format!("var {} not found", key))),
            Some(T) => Ok(T)
        }
    }
    #[inline]
    pub fn set_var(&mut self, key: VarId, var: Var) -> Option<Var> {
        self.vars.insert(key, var)
    }
}
