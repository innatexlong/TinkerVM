// parser/cmd.rs
#[repr(u8)]
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub(crate) enum Cmd {
    Add, Sub, Mul, Div, Mod,  // 0-4
    Retc, Retv, Movc, Mov,  // 5-8
    Newv, Newp, Delv, Delp, Nop,  // 9-13
    Call,  // 14
    PushVar, PopVar,  // 15-16
    Ldc,  // 17
}

pub(crate) mod cmd_u8 {
    use super::Cmd;
    // arithmetic
    pub const ADD: u8 = Cmd::Add as u8;
    pub const SUB: u8 = Cmd::Sub as u8;
    pub const MUL: u8 = Cmd::Mul as u8;
    pub const DIV: u8 = Cmd::Div as u8;
    pub const MOD: u8 = Cmd::Mod as u8;
    // control flow
    pub const RETC: u8 = Cmd::Retc as u8;
    pub const RETV: u8 = Cmd::Retv as u8;
    // memory
    pub const MOVC: u8 = Cmd::Movc as u8;
    pub const MOV: u8 = Cmd::Mov as u8;
    pub const NEWV: u8 = Cmd::Newv as u8;
    pub const NEWP: u8 = Cmd::Newp as u8;
    pub const DELV: u8 = Cmd::Delv as u8;
    pub const DELP: u8 = Cmd::Delp as u8;
    // miscellaneous
    pub const NOP: u8 = Cmd::Nop as u8;
    // functions
    pub const CALL: u8 = Cmd::Call as u8;
    // operand stack
    pub const PUSHVAR: u8 = Cmd::PushVar as u8;
    pub const POPVAR: u8 = Cmd::PopVar as u8;
    pub const LDC: u8 = Cmd::Ldc as u8;
}