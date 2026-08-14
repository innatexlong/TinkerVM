// parser/cmd.rs
#[repr(u8)]
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub(crate) enum Cmd {
    Add, Sub, Mul, Div, Mod,
    Movc, Mov, Retc, Retv,
    Newv, Newp, Nop
}

pub(crate) mod cmd_u8 {
    use super::Cmd;
    pub const ADD: u8 = Cmd::Add as u8;
    pub const SUB: u8 = Cmd::Sub as u8;
    pub const MUL: u8 = Cmd::Mul as u8;
    pub const DIV: u8 = Cmd::Div as u8;
    pub const MOD: u8 = Cmd::Mod as u8;
    pub const MOVC: u8 = Cmd::Movc as u8;
    pub const MOV: u8 = Cmd::Mov as u8;
    pub const RETC: u8 = Cmd::Retc as u8;
    pub const RETV: u8 = Cmd::Retv as u8;
    pub const NEWV: u8 = Cmd::Newv as u8;
    pub const NEWP: u8 = Cmd::Newp as u8;
    pub const NOP: u8 = Cmd::Nop as u8;
}