use std::collections::HashMap;

pub struct MemoryPool {

}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VarType {
    // Float,
    // Double,
    // U8, U16,
    U32, //U64,
    VarPtr
}

#[derive(Clone, PartialEq, Eq)]
pub struct Var {
    ptr: u32,
    size: u32,
    var_type: VarType
}

#[derive(Clone, PartialEq, Eq)]
pub struct TempVar {
    value: Vec<u8>,
    var_type: VarType
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VarId(pub u32);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pos(pub u32);
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
    memory_pool: MemoryPool,
    vars: HashMap<VarId, Var>,
    // funcs: HashMap<String, FuncPtr>, // TODO
    // parent: Env  // TODO
}

impl Env {
    fn new() -> Self {
        Self {
            memory_pool: MemoryPool {},
            vars: Default::default(),
        }
    }
    #[inline]
    fn get_var(&self, key: VarId) -> Result<&Var, crate::exceptions::Error> {
        match self.vars.get(&key) {
            None => Err(crate::exceptions::Error::SegFault(format!("var {} not found", key))),
            Some(T) => Ok(T)
        }
    }
    #[inline]
    fn get_var_mut(&mut self, key: VarId) -> Result<&mut Var, crate::exceptions::Error> {
        match self.vars.get_mut(&key) {
            None => Err(crate::exceptions::Error::SegFault(format!("var {} not found", key))),
            Some(T) => Ok(T)
        }
    }
    #[inline]
    fn set_var(&mut self, key: VarId, var: Var) -> Option<Var> {
        self.vars.insert(key, var)
    }
}
